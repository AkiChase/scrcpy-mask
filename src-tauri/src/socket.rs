use tokio::{
    io::AsyncReadExt,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener,
    },
};

use crate::{
    control_msg::{self, ControlMsgType},
    scrcpy_mask_cmd::{self, ScrcpyMaskCmdType},
};

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn bind(port: u16) -> Self {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .unwrap();
        println!("已开启监听127.0.0.1:{}", port);
        Self { listener }
    }

    pub async fn accept(
        &self,
        front_msg_broadcast_receiver: tokio::sync::broadcast::Receiver<String>,
        device_reply_sender: tokio::sync::mpsc::Sender<String>,
    ) {
        let (client, _) = self.listener.accept().await.unwrap();
        println!("成功连接scrcpy-server:{:?}", client.local_addr());

        let (read_half, write_half) = client.into_split();

        // 开启线程读取设备发送的信息，并通过通道传递到与前端通信的线程，最后与前端通信的线程发送全局事件，告知前端设备发送的信息
        tokio::spawn(async move {
            Self::read_socket(read_half, device_reply_sender).await;
        });

        // 开启线程接收通道消息，其中通道消息来自前端发送的事件
        tokio::spawn(async move {
            Self::recv_front_msg(write_half, front_msg_broadcast_receiver).await;
        });
    }

    // 从客户端读取
    async fn read_socket(
        mut reader: OwnedReadHalf,
        device_reply_sender: tokio::sync::mpsc::Sender<String>,
    ) {
        // read metadata (device name)
        let mut buf: [u8; 64] = [0; 64];
        let device_name = match reader.read(&mut buf).await {
            Err(_e) => {
                eprintln!("failed to read metadata");
                return;
            }
            Ok(0) => {
                eprintln!("failed to read metadata");
                return;
            }
            Ok(n) => {
                let mut end = n;
                while buf[end - 1] == 0 {
                    end -= 1;
                }
                let device_name = std::str::from_utf8(&buf[..end]).unwrap();
                let msg = format!(
                    r#"{{
                        "msg": "MetaData",
                        "deviceName": "{device_name}"
                    }}"#
                );
                device_reply_sender.send(msg).await.unwrap();
                device_name
            }
        };

        loop {
            match reader.read_u8().await {
                Err(e) => {
                    eprintln!("Failed to read from scrcpy server, maybe it was closed. Error:{}", e);
                    break;
                }
                Ok(message_type) => {
                    let message_type = match DeviceMsgType::from_u8(message_type) {
                        Some(t) => t,
                        None => {
                            println!("Ignore unkonw message type: {}", message_type);
                            break;
                        }
                    };
                    if let Err(e) = Self::handle_device_message(
                        message_type,
                        &mut reader,
                        &device_reply_sender,
                        device_name,
                    )
                    .await
                    {
                        eprintln!("Failed to handle device message: {}", e);
                    }
                }
            }
        }
    }

    async fn handle_device_message(
        message_type: DeviceMsgType,
        reader: &mut OwnedReadHalf,
        device_reply_sender: &tokio::sync::mpsc::Sender<String>,
        device_name: &str,
    ) -> anyhow::Result<()> {
        match message_type {
            // 设备剪切板变动
            DeviceMsgType::DeviceMsgTypeClipboard => {
                let text_length = reader.read_u32().await?;
                let mut buf: Vec<u8> = vec![0; text_length as usize];
                reader.read_exact(&mut buf).await?;
                let cb = String::from_utf8(buf)?;
                let msg = format!(
                    r#"{{
                        "msg": "ClipboardChanged",
                        "deviceName": "{device_name}",
                        "clipboard": "{cb}"
                    }}"#
                );
                device_reply_sender.send(msg).await?;
            }
            // 设备剪切板设置成功的回复
            DeviceMsgType::DeviceMsgTypeAckClipboard => {
                let sequence = reader.read_u64().await?;
                let msg = format!(
                    r#"{{
                        "type": "ClipboardSetAck",
                        "deviceName": "{device_name}",
                        "sequence": "{sequence}"
                    }}"#
                );
                device_reply_sender.send(msg).await?;
            }
            // 虚拟设备输出，仅读取但不做进一步处理
            DeviceMsgType::DeviceMsgTypeUhidOutput => {
                let _id = reader.read_u16().await?;
                let size = reader.read_u16().await?;
                let mut buf: Vec<u8> = vec![0; size as usize];
                reader.read_exact(&mut buf).await?;
            }
        };
        anyhow::Ok(())
    }

    // 接收前端发送的消息，执行相关操作
    async fn recv_front_msg(
        mut write_half: OwnedWriteHalf,
        mut front_msg_broadcast_receiver: tokio::sync::broadcast::Receiver<String>,
    ) {
        loop {
            match front_msg_broadcast_receiver.recv().await {
                Ok(msg) => {
                    match serde_json::from_str::<serde_json::Value>(&msg) {
                        Err(_e) => {
                            println!("无法解析的Json数据: {}", msg);
                        }
                        Ok(payload) => {
                            if let Some(front_msg_type) = payload["msgType"].as_i64() {
                                // 发送原始控制信息
                                if front_msg_type >= 0 && front_msg_type <= 14 {
                                    let ctrl_msg_type =
                                        ControlMsgType::from_i64(front_msg_type).unwrap();
                                    control_msg::send_ctrl_msg(
                                        ctrl_msg_type,
                                        &payload["msgData"],
                                        &mut write_half,
                                    )
                                    .await;
                                    println!("控制信息发送完成！");
                                    continue;
                                } else {
                                    // 处理Scrcpy Mask命令
                                    if let Some(cmd_type) =
                                        ScrcpyMaskCmdType::from_i64(front_msg_type)
                                    {
                                        scrcpy_mask_cmd::handle_sm_cmd(
                                            cmd_type,
                                            &payload["msgData"],
                                            &mut write_half,
                                        )
                                        .await;
                                        continue;
                                    }
                                }
                            }

                            eprintln!("fc-command非法")
                        }
                    };
                }
                Err(e) => {
                    eprintln!("front-command接收失败: {:?}", e)
                }
            };
        }
    }
}

#[derive(Debug)]
enum DeviceMsgType {
    DeviceMsgTypeClipboard,
    DeviceMsgTypeAckClipboard,
    DeviceMsgTypeUhidOutput,
}

impl DeviceMsgType {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::DeviceMsgTypeClipboard),
            1 => Some(Self::DeviceMsgTypeAckClipboard),
            2 => Some(Self::DeviceMsgTypeUhidOutput),
            _ => None,
        }
    }
}
