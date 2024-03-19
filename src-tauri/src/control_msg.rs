use crate::binary;
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf};

pub const SC_CONTROL_MSG_INJECT_TEXT_MAX_LENGTH: usize = 300;
pub const SC_CONTROL_MSG_MAX_SIZE: usize = 1 << 18; // 256k
pub const SC_CONTROL_MSG_CLIPBOARD_TEXT_MAX_LENGTH: usize = SC_CONTROL_MSG_MAX_SIZE - 14;

pub fn gen_ctrl_msg(ctrl_msg_type: ControlMsgType, payload: &serde_json::Value) -> Vec<u8> {
    match ctrl_msg_type {
        ControlMsgType::ControlMsgTypeInjectKeycode => gen_inject_key_ctrl_msg(
            ctrl_msg_type as u8,
            payload["action"].as_u64().unwrap() as u8,
            payload["keycode"].as_u64().unwrap() as u32,
            payload["repeat"].as_u64().unwrap() as u32,
            payload["metastate"].as_u64().unwrap() as u32,
        ),
        ControlMsgType::ControlMsgTypeInjectText => {
            let mut buf: Vec<u8> = vec![ctrl_msg_type as u8];
            let text = payload["text"].as_str().unwrap();
            binary::write_string(text, SC_CONTROL_MSG_INJECT_TEXT_MAX_LENGTH, &mut buf);
            buf
        }
        ControlMsgType::ControlMsgTypeInjectTouchEvent => gen_inject_touch_ctrl_msg(
            ctrl_msg_type as u8,
            payload["action"].as_u64().unwrap() as u8,
            payload["pointerId"].as_u64().unwrap(),
            payload["position"]["x"].as_i64().unwrap() as i32,
            payload["position"]["y"].as_i64().unwrap() as i32,
            payload["position"]["w"].as_i64().unwrap() as u16,
            payload["position"]["h"].as_i64().unwrap() as u16,
            binary::float_to_u16fp(payload["pressure"].as_f64().unwrap() as f32),
            payload["actionButton"].as_u64().unwrap() as u32,
            payload["buttons"].as_u64().unwrap() as u32,
        ),
        ControlMsgType::ControlMsgTypeInjectScrollEvent => {
            let mut buf = vec![0; 21];
            buf[0] = ctrl_msg_type as u8;
            binary::write_posion(
                &mut buf[1..13],
                payload["position"]["x"].as_i64().unwrap() as i32,
                payload["position"]["y"].as_i64().unwrap() as i32,
                payload["position"]["w"].as_i64().unwrap() as u16,
                payload["position"]["h"].as_i64().unwrap() as u16,
            );
            binary::write_16be(
                &mut buf[13..15],
                binary::float_to_i16fp(payload["hscroll"].as_f64().unwrap() as f32) as u16,
            );
            binary::write_16be(
                &mut buf[15..17],
                binary::float_to_i16fp(payload["vscroll"].as_f64().unwrap() as f32) as u16,
            );
            binary::write_32be(
                &mut buf[17..21],
                payload["buttons"].as_u64().unwrap() as u32,
            );
            buf
        }
        ControlMsgType::ControlMsgTypeBackOrScreenOn => {
            vec![
                ctrl_msg_type as u8,
                payload["action"].as_u64().unwrap() as u8,
            ]
        }
        ControlMsgType::ControlMsgTypeGetClipboard => {
            vec![
                ctrl_msg_type as u8,
                payload["copyKey"].as_u64().unwrap() as u8,
            ]
        }
        ControlMsgType::ControlMsgTypeSetClipboard => {
            let mut buf: Vec<u8> = vec![0; 10];
            buf[0] = ctrl_msg_type as u8;
            binary::write_64be(&mut buf[1..9], payload["sequence"].as_u64().unwrap());
            buf[9] = payload["paste"].as_bool().unwrap_or(false) as u8;
            let text = payload["text"].as_str().unwrap();
            binary::write_string(text, SC_CONTROL_MSG_CLIPBOARD_TEXT_MAX_LENGTH, &mut buf);
            buf
        }
        ControlMsgType::ControlMsgTypeSetScreenPowerMode => {
            vec![ctrl_msg_type as u8, payload["mode"].as_u64().unwrap() as u8]
        }
        ControlMsgType::ControlMsgTypeUhidCreate => {
            let size = payload["reportDescSize"].as_u64().unwrap() as u16;
            let mut buf: Vec<u8> = vec![0; 5];
            buf[0] = ctrl_msg_type as u8;
            binary::write_16be(&mut buf[1..3], payload["id"].as_u64().unwrap() as u16);
            binary::write_16be(&mut buf[3..5], size);
            let report_desc = payload["reportDesc"].as_array().unwrap();
            let report_desc_u8: Vec<u8> = report_desc
                .iter()
                .map(|x| x.as_u64().unwrap() as u8)
                .collect();
            buf.extend_from_slice(&report_desc_u8);
            buf
        }
        ControlMsgType::ControlMsgTypeUhidInput => {
            let size = payload["size"].as_u64().unwrap() as u16;
            let mut buf: Vec<u8> = vec![0; 5];
            buf[0] = ctrl_msg_type as u8;
            binary::write_16be(&mut buf[1..3], payload["id"].as_u64().unwrap() as u16);
            binary::write_16be(&mut buf[3..5], size);
            let data = payload["data"].as_array().unwrap();
            let data_u8: Vec<u8> = data.iter().map(|x| x.as_u64().unwrap() as u8).collect();
            buf.extend_from_slice(&data_u8);
            buf
        }
        // other control message types do not have a payload
        _ => {
            vec![ctrl_msg_type as u8]
        }
    }
}

pub fn gen_inject_key_ctrl_msg(
    ctrl_msg_type: u8,
    action: u8,
    keycode: u32,
    repeat: u32,
    metastate: u32,
) -> Vec<u8> {
    let mut buf = vec![0; 14];
    buf[0] = ctrl_msg_type;
    buf[1] = action;
    binary::write_32be(&mut buf[2..6], keycode);
    binary::write_32be(&mut buf[6..10], repeat);
    binary::write_32be(&mut buf[10..14], metastate);
    buf
}

pub fn gen_inject_touch_ctrl_msg(
    ctrl_msg_type: u8,
    action: u8,
    pointer_id: u64,
    x: i32,
    y: i32,
    w: u16,
    h: u16,
    pressure: u16,
    action_button: u32,
    buttons: u32,
) -> Vec<u8> {
    let mut buf = vec![0; 32];
    buf[0] = ctrl_msg_type;
    buf[1] = action;
    binary::write_64be(&mut buf[2..10], pointer_id);
    binary::write_posion(&mut buf[10..22], x, y, w, h);
    binary::write_16be(&mut buf[22..24], pressure);
    binary::write_32be(&mut buf[24..28], action_button);
    binary::write_32be(&mut buf[28..32], buttons);
    buf
}

pub async fn send_ctrl_msg(
    ctrl_msg_type: ControlMsgType,
    payload: &serde_json::Value,
    writer: &mut OwnedWriteHalf,
) {
    let buf = gen_ctrl_msg(ctrl_msg_type, payload);
    writer.write_all(&buf).await.unwrap();
    writer.flush().await.unwrap();
}

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
