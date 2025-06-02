use std::sync::Arc;

use anyhow::Context;
use serde_json::json;
use tauri::Listener;
use tokio::{
    io::AsyncReadExt,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

use crate::{
    control_msg::{self, ControlMsgType},
    scrcpy_mask_cmd::{self, ScrcpyMaskCmdType},
    share,
};

pub async fn connect_socket(
    address: String,
    front_msg_receiver: tokio::sync::mpsc::Receiver<String>,
    device_reply_sender: tokio::sync::mpsc::Sender<String>,
    listen_handler: u32,
    app: Arc<tauri::AppHandle>,
) -> anyhow::Result<()> {
    let client = TcpStream::connect(address)
        .await
        .context("Socket connect failed")?;

    log::info!(
        "Connected to scrcpy server: {}",
        client.local_addr().unwrap()
    );

    let (read_half, write_half) = client.into_split();

    // 开启线程读取设备发送的信息，并通过通道传递到与前端通信的线程，最后与前端通信的线程发送全局事件，告知前端设备发送的信息
    tokio::spawn(async move {
        read_socket(read_half, device_reply_sender).await;
    });

    // 开启线程接收通道消息，其中通道消息来自前端发送的事件
    tokio::spawn(async move {
        recv_front_msg(write_half, front_msg_receiver, listen_handler, app).await;
    });
    anyhow::Ok(())
}

// read from scrcpy server
async fn read_socket(
    mut reader: OwnedReadHalf,
    device_reply_sender: tokio::sync::mpsc::Sender<String>,
) {
    // read dummy byte
    let mut buf: [u8; 1] = [0; 1];
    if let Err(_e) = reader.read_exact(&mut buf).await {
        log::error!("Failed to read dummy byte");
        return;
    }

    // read metadata (device name)
    let mut buf: [u8; 64] = [0; 64];
    match reader.read(&mut buf).await {
        Err(_e) => {
            log::error!("Failed to read metadata");
            return;
        }
        Ok(0) => {
            log::error!("Failed to read metadata");
            return;
        }
        Ok(n) => {
            let mut end = n;
            while buf[end - 1] == 0 {
                end -= 1;
            }
            let device_name = std::str::from_utf8(&buf[..end]).unwrap();
            // update device name for share
            share::CLIENT_INFO
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .device_name = device_name.to_string();

            let msg = json!({
                "type": "MetaData",
                "deviceName": device_name,
            })
            .to_string();
            device_reply_sender.send(msg).await.unwrap();
        }
    };

    loop {
        match reader.read_u8().await {
            Err(e) => {
                log::error!(
                    "Failed to read from scrcpy server, maybe it was closed: {}",
                    e
                );
                log::info!("Drop TcpStream reader");
                drop(reader);
                return;
            }
            Ok(message_type) => {
                let message_type = match DeviceMsgType::from_u8(message_type) {
                    Some(t) => t,
                    None => {
                        log::warn!("Ignore unkonw message type: {}", message_type);
                        continue;
                    }
                };
                if let Err(e) =
                    handle_device_message(message_type, &mut reader, &device_reply_sender).await
                {
                    log::warn!("Failed to handle device message: {}", e);
                }
            }
        }
    }
}

async fn handle_device_message(
    message_type: DeviceMsgType,
    reader: &mut OwnedReadHalf,
    device_reply_sender: &tokio::sync::mpsc::Sender<String>,
) -> anyhow::Result<()> {
    match message_type {
        // Clipboard changed
        DeviceMsgType::DeviceMsgTypeClipboard => {
            let text_length = reader.read_u32().await?;
            let mut buf: Vec<u8> = vec![0; text_length as usize];
            reader.read_exact(&mut buf).await?;
            let cb = String::from_utf8(buf)?;
            let msg = json!({
                "type": "ClipboardChanged",
                "clipboard": cb
            })
            .to_string();
            device_reply_sender.send(msg).await?;
        }
        // Clipboard set ACK
        DeviceMsgType::DeviceMsgTypeAckClipboard => {
            let sequence = reader.read_u64().await?;
            let msg = json!({
                "type": "ClipboardSetAck",
                "sequence": sequence
            })
            .to_string();
            device_reply_sender.send(msg).await?;
        }
        // Virtual device output(read only but not further processing)
        DeviceMsgType::DeviceMsgTypeUhidOutput => {
            let _id = reader.read_u16().await?;
            let size = reader.read_u16().await?;
            let mut buf: Vec<u8> = vec![0; size as usize];
            reader.read_exact(&mut buf).await?;
        }
        // Device rotation
        DeviceMsgType::DeviceMsgTypeRotation => {
            let rotation = reader.read_u16().await?;
            let width = reader.read_i32().await?;
            let height = reader.read_i32().await?;
            let msg = json!({
                "type": "DeviceRotation",
                "rotation": rotation,
                "width": width,
                "height": height
            })
            .to_string();
            share::CLIENT_INFO
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .set_size(width, height);
            device_reply_sender.send(msg).await?;
        }
    };
    anyhow::Ok(())
}

// Receive messages sent by the front-end and perform related operations
async fn recv_front_msg(
    mut write_half: OwnedWriteHalf,
    mut front_msg_receiver: tokio::sync::mpsc::Receiver<String>,
    listen_handler: u32,
    app: Arc<tauri::AppHandle>,
) {
    while let Some(msg) = front_msg_receiver.recv().await {
        match serde_json::from_str::<serde_json::Value>(&msg) {
            Err(_e) => {
                log::warn!("Failed to parse front msg as json: {}", msg);
            }
            Ok(payload) => {
                if let Some(front_msg_type) = payload["msgType"].as_i64() {
                    if front_msg_type >= 0 && front_msg_type <= 14 {
                        // Processing Control commands
                        let ctrl_msg_type = ControlMsgType::from_i64(front_msg_type).unwrap();
                        control_msg::send_ctrl_msg(
                            ctrl_msg_type,
                            &payload["msgData"],
                            &mut write_half,
                        )
                        .await;
                        continue;
                    } else {
                        // Processing Scrcpy Mask commands
                        if let Some(cmd_type) = ScrcpyMaskCmdType::from_i64(front_msg_type) {
                            if let ScrcpyMaskCmdType::Shutdown = cmd_type {
                                *share::CLIENT_INFO.lock().unwrap() = None;

                                drop(write_half);
                                log::info!("Drop TcpStream writer");
                                app.unlisten(listen_handler);
                                log::info!("Front msg channel closed");
                                return;
                            }

                            scrcpy_mask_cmd::handle_sm_cmd(
                                cmd_type,
                                &payload["msgData"],
                                &mut write_half,
                            )
                            .await
                        }
                    }
                } else {
                    log::warn!("Invalid font command!: {:?}", payload);
                }
            }
        };
    }

    log::info!("Font msg channel closed");
}

#[derive(Debug)]
enum DeviceMsgType {
    DeviceMsgTypeClipboard,
    DeviceMsgTypeAckClipboard,
    DeviceMsgTypeUhidOutput,
    DeviceMsgTypeRotation,
}

impl DeviceMsgType {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::DeviceMsgTypeClipboard),
            1 => Some(Self::DeviceMsgTypeAckClipboard),
            2 => Some(Self::DeviceMsgTypeUhidOutput),
            3 => Some(Self::DeviceMsgTypeRotation),
            _ => None,
        }
    }
}
