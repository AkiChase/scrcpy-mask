use crate::binary;
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf};

pub async fn send_ctrl_msg(
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
            binary::write_posion(
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

#[derive(Debug)]
pub enum ControlMsgType {
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
    pub fn from_i64(value: i64) -> Option<Self> {
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
