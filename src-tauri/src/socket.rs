use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener,
    },
};

use crate::binary;

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
        fc_broadcast_receiver: tokio::sync::broadcast::Receiver<String>,
        device_reply_sender: tokio::sync::mpsc::Sender<String>,
    ) {
        let (client, _) = self.listener.accept().await.unwrap();
        println!("成功连接scrcpy-server:{:?}", client.local_addr());

        tokio::spawn(async move {
            let (read_half, write_half) = client.into_split();

            // 开启线程读取设备发送的信息，并通过通道传递到与前端通信的线程，最后与前端通信的线程发送全局事件，告知前端设备发送的信息
            tokio::spawn(async move {
                Self::read_socket(read_half, device_reply_sender).await;
            });

            // 开启线程接收通道消息，其中通道消息来自前端发送的事件
            tokio::spawn(async move {
                Self::recv_front_command(write_half, fc_broadcast_receiver).await;
            });
        });
    }

    // 从客户端读取
    async fn read_socket(
        mut reader: OwnedReadHalf,
        device_reply_sender: tokio::sync::mpsc::Sender<String>,
    ) {
        // read metadata (device name)
        let mut buf: [u8; 64] = [0; 64];
        match reader.read(&mut buf).await {
            Err(_e) => eprintln!("failed to read metadata"),
            Ok(n) => {
                let device_name = std::str::from_utf8(&buf[..n]).unwrap();
                println!("device name: {}", device_name);
            }
        };

        loop {
            match reader.read_u8().await {
                Err(e) => {
                    eprintln!("read from client error:{}", e);
                    break;
                }
                Ok(message_type) => {
                    let message_type = match DeviceMsgType::from_u8(message_type) {
                        Some(t) => t,
                        None => {
                            println!("Unkonw message type: {}", message_type);
                            break;
                        }
                    };
                    if let Err(e) =
                        Self::handle_device_message(message_type, &mut reader, &device_reply_sender)
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
    ) -> anyhow::Result<()> {
        match message_type {
            // 设备剪切板变动
            DeviceMsgType::DeviceMsgTypeClipboard => {
                let text_length = reader.read_u32().await?;
                let mut buf: Vec<u8> = vec![0; text_length as usize];
                reader.read_exact(&mut buf).await?;
                device_reply_sender
                    .send(format!(
                        "收到DeviceMsgTypeClipboard设备消息, 剪切板内容:{}",
                        String::from_utf8(buf)?
                    ))
                    .await?;
            }
            // 设备剪切板设置成功的回复
            DeviceMsgType::DeviceMsgTypeAckClipboard => {
                let sequence = reader.read_u64().await?;
                device_reply_sender
                    .send(format!(
                        "收到DeviceMsgTypeAckClipboard设备消息, sequence:{}",
                        sequence
                    ))
                    .await?;
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
    async fn recv_front_command(
        mut write_half: OwnedWriteHalf,
        mut fc_broadcast_receiver: tokio::sync::broadcast::Receiver<String>,
    ) {
        loop {
            match fc_broadcast_receiver.recv().await {
                Ok(msg) => {
                    match serde_json::from_str::<serde_json::Value>(&msg) {
                        Err(_e) => {
                            println!("无法解析的Json数据: {}", msg);
                        }
                        Ok(payload) => {
                            if let Some(fc_type) = payload["fcType"].as_i64() {
                                if fc_type >= 0 && fc_type <= 14 {
                                    let ctrl_msg_type = ControlMsgType::from_i64(fc_type).unwrap();
                                    Self::send_ctrl_msg(ctrl_msg_type, &payload["msgData"], &mut write_half)
                                        .await;
                                    println!("控制信息发送完成！");
                                    continue;
                                } else {
                                    if let Some(pfc_type) = PureFrontCommandType::from_i64(fc_type)
                                    {
                                        Self::exec_pfc(pfc_type).await;
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

    async fn exec_pfc(pfc_type: PureFrontCommandType) {
        match pfc_type {
            _ => {}
        }
        println!("收到纯前端命令:{:?}", pfc_type);
    }

    async fn send_ctrl_msg(
        ctrl_msg_type: ControlMsgType,
        payload: &serde_json::Value,
        writer: &mut OwnedWriteHalf,
    ) {
        match ctrl_msg_type {
            ControlMsgType::ControlMsgTypeInjectTouchEvent => {
                let mut buf: [u8; 32] = [0; 32];
                buf[0] = ControlMsgType::ControlMsgTypeInjectTouchEvent as u8;
                buf[1] = payload["action"].as_u64().unwrap() as u8;
                binary::write_64be(&mut buf[2..10], payload["pointerId"].as_u64().unwrap());
                Self::write_posion(
                    &mut buf[10..22],
                    payload["position"]["x"].as_i64().unwrap() as i32,
                    payload["position"]["y"].as_i64().unwrap() as i32,
                    payload["position"]["w"].as_i64().unwrap() as u16,
                    payload["position"]["h"].as_i64().unwrap() as u16,
                );
                binary::write_16be(
                    &mut buf[22..24],
                    binary::float_to_u16fp(payload["pressure"].as_f64().unwrap() as f32),
                );
                binary::write_32be(
                    &mut buf[24..28],
                    payload["actionButton"].as_u64().unwrap() as u32,
                );
                binary::write_32be(
                    &mut buf[28..32],
                    payload["buttons"].as_u64().unwrap() as u32,
                );

                writer.write_all(&buf).await.unwrap();
                writer.flush().await.unwrap();
            }
            _ => {}
        };
        println!("收到前端命令，发送控制消息:{:?}", ctrl_msg_type);
    }

    fn write_posion(buf: &mut [u8], x: i32, y: i32, w: u16, h: u16) {
        binary::write_32be(buf, x as u32);
        binary::write_32be(&mut buf[4..8], y as u32);
        binary::write_16be(&mut buf[8..10], w);
        binary::write_16be(&mut buf[10..12], h);
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

#[derive(Debug)]
enum ControlMsgType {
    ControlMsgTypeInjectKeycode,            //发送原始按键
    ControlMsgTypeInjectText, //发送文本，不知道是否能输入中文（估计只是把文本转为keycode的输入效果）
    ControlMsgTypeInjectTouchEvent, //发送触摸事件
    ControlMsgTypeInjectScrollEvent, //发送滚动事件（类似接入鼠标后滚动滚轮的效果，不是通过触摸实现的）
    ControlMsgTypeBackOrScreenOn,    //应该就是发送返回键
    ControlMsgTypeExpandNotificationPanel, //打开消息面板
    ControlMsgTypeExpandSettingsPanel, //打开设置面板（就是消息面板右侧的）
    ControlMsgTypeCollapsePanels,    //折叠上述面板
    ControlMsgTypeGetClipboard,      //获取剪切板
    ControlMsgTypeSetClipboard,      //设置剪切板
    ControlMsgTypeSetScreenPowerMode, //设置屏幕电源模式，是关闭设备屏幕的（SC_SCREEN_POWER_MODE_OFF 和 SC_SCREEN_POWER_MODE_NORMAL ）
    ControlMsgTypeRotateDevice,       //旋转设备屏幕
    ControlMsgTypeUhidCreate,         //创建虚拟设备？从而模拟真实的键盘、鼠标用的，目前没用
    ControlMsgTypeUhidInput,          //同上转发键盘、鼠标的输入，目前没用
    ControlMsgTypeOpenHardKeyboardSettings, //打开设备的硬件键盘设置，目前没用
}

impl ControlMsgType {
    fn from_i64(value: i64) -> Option<Self> {
        match value {
            0 => Some(Self::ControlMsgTypeInjectKeycode),
            1 => Some(Self::ControlMsgTypeInjectText),
            2 => Some(Self::ControlMsgTypeInjectTouchEvent),
            3 => Some(Self::ControlMsgTypeInjectScrollEvent),
            4 => Some(Self::ControlMsgTypeBackOrScreenOn),
            5 => Some(Self::ControlMsgTypeExpandNotificationPanel),
            6 => Some(Self::ControlMsgTypeExpandSettingsPanel),
            7 => Some(Self::ControlMsgTypeCollapsePanels),
            8 => Some(Self::ControlMsgTypeGetClipboard),
            9 => Some(Self::ControlMsgTypeSetClipboard),
            10 => Some(Self::ControlMsgTypeSetScreenPowerMode),
            11 => Some(Self::ControlMsgTypeRotateDevice),
            12 => Some(Self::ControlMsgTypeUhidCreate),
            13 => Some(Self::ControlMsgTypeUhidInput),
            14 => Some(Self::ControlMsgTypeOpenHardKeyboardSettings),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum PureFrontCommandType {
    PasteText,
}

impl PureFrontCommandType {
    fn from_i64(value: i64) -> Option<Self> {
        match value {
            15 => Some(Self::PasteText),
            _ => None,
        }
    }
}
