use rust_i18n::t;
use tokio::{io::AsyncReadExt, net::tcp::OwnedReadHalf};

use crate::scrcpy::constant;

const SC_CONTROL_MSG_INJECT_TEXT_MAX_LENGTH: usize = 300;
const SC_CONTROL_MSG_MAX_SIZE: usize = 1 << 18; // 256k
const SC_CONTROL_MSG_CLIPBOARD_TEXT_MAX_LENGTH: usize = SC_CONTROL_MSG_MAX_SIZE - 14;

struct Binary;
impl Binary {
    pub fn write_16be(buf: &mut [u8], val: u16) {
        buf[0] = (val >> 8) as u8;
        buf[1] = val as u8;
    }

    pub fn write_32be(buf: &mut [u8], val: u32) {
        buf[0] = (val >> 24) as u8;
        buf[1] = (val >> 16) as u8;
        buf[2] = (val >> 8) as u8;
        buf[3] = val as u8;
    }

    pub fn write_64be(buf: &mut [u8], val: u64) {
        buf[0] = (val >> 56) as u8;
        buf[1] = (val >> 48) as u8;
        buf[2] = (val >> 40) as u8;
        buf[3] = (val >> 32) as u8;
        buf[4] = (val >> 24) as u8;
        buf[5] = (val >> 16) as u8;
        buf[6] = (val >> 8) as u8;
        buf[7] = val as u8;
    }

    pub fn write_posion(buf: &mut [u8], x: i32, y: i32, w: u16, h: u16) {
        Self::write_32be(buf, x as u32);
        Self::write_32be(&mut buf[4..8], y as u32);
        Self::write_16be(&mut buf[8..10], w);
        Self::write_16be(&mut buf[10..12], h);
    }

    pub fn write_string(utf8: &str, max_len: usize, buf: &mut Vec<u8>) {
        let len = Self::str_utf8_truncation_index(utf8, max_len) as u32;
        // first 4 bytes for length
        let len_bytes = len.to_be_bytes();
        buf.extend_from_slice(&len_bytes);
        // then [len] bytes for the string
        buf.extend_from_slice(utf8.as_bytes())
    }

    // truncate utf8 string to max_len bytes
    fn str_utf8_truncation_index(utf8: &str, max_len: usize) -> usize {
        let len = utf8.len();
        if len <= max_len {
            return len;
        }
        let mut len = max_len;
        while utf8.is_char_boundary(len) {
            len -= 1;
        }
        len
    }
}

#[repr(u8)]
pub enum ScrcpyControlMsgType {
    InjectKeycode,            // 发送原始按键
    InjectText,               // 发送文本对应 keycode
    InjectTouchEvent,         // 发送触摸事件
    InjectScrollEvent,        // 发送滚动事件 (非Tap实现)
    BackOrScreenOn,           // 发送返回键
    ExpandNotificationPanel,  // 打开消息面板
    ExpandSettingsPanel,      // 打开设置面板 (在消息面板侧面)
    CollapsePanels,           // 折叠设置面板
    GetClipboard,             // 获取剪切板
    SetClipboard,             // 设置剪切板
    SetDisplayPower,          // 设置屏幕显示开关
    RotateDevice,             // 旋转设备屏幕
    UhidCreate,               // 创建虚拟设备
    UhidInput,                // 发送虚拟设备输入
    UhidDestroy,              // 销毁虚拟设备
    OpenHardKeyboardSettings, // 打开硬件键盘设置
    StartApp,                 // 启动应用
    ResetVideo,               // 重置视频流
}

#[derive(Debug, Clone)]
pub enum ScrcpyControlMsg {
    InjectKeycode {
        action: constant::KeyEventAction, // u8
        keycode: constant::Keycode,       // u32
        repeat: u32,                      // repeated times
        metastate: constant::MetaState,   // u32
    },
    InjectText {
        text: String,
    },
    InjectTouchEvent {
        action: constant::MotionEventAction, // u8
        pointer_id: u64,
        x: i32,
        y: i32,
        w: u16,
        h: u16,
        pressure: half::f16,
        action_button: constant::MotionEventButtons, // u32
        buttons: constant::MotionEventButtons,       // keep the same with action_button
    },
    InjectScrollEvent {
        x: i32,
        y: i32,
        w: u16,
        h: u16,
        hscroll: u16,
        vscroll: u16,
        buttons: u32, // the buttons pressed when scrolling, just set as 0
    },
    BackOrScreenOn {
        action: constant::KeyEventAction, // u8
    },
    GetClipboard {
        copy_key: constant::CopyKey,
    },
    SetClipboard {
        sequence: u64,
        paste: bool, // u8
        text: String,
    },
    SetDisplayPower {
        mode: bool, // u8
    },
    RotateDevice,
    ResetVideo,
}

impl From<ScrcpyControlMsg> for Vec<u8> {
    fn from(msg: ScrcpyControlMsg) -> Self {
        match msg {
            ScrcpyControlMsg::InjectKeycode {
                action,
                keycode,
                repeat,
                metastate,
            } => {
                let mut buf: Vec<u8> = vec![0; 14];
                buf[0] = ScrcpyControlMsgType::InjectKeycode as u8;
                buf[1] = action as u8;
                Binary::write_32be(&mut buf[2..6], keycode as u32);
                Binary::write_32be(&mut buf[6..10], repeat);
                Binary::write_32be(&mut buf[10..14], metastate.bits());
                buf
            }
            ScrcpyControlMsg::InjectText { text } => {
                let mut buf: Vec<u8> = vec![ScrcpyControlMsgType::InjectText as u8];
                Binary::write_string(&text, SC_CONTROL_MSG_INJECT_TEXT_MAX_LENGTH, &mut buf);
                buf
            }
            ScrcpyControlMsg::InjectTouchEvent {
                action, // u8
                pointer_id,
                x,
                y,
                w,
                h,
                pressure,
                action_button, // u32
                buttons,       // keep the same with action_button
            } => {
                let mut buf = vec![0; 32];
                buf[0] = ScrcpyControlMsgType::InjectTouchEvent as u8;
                buf[1] = action as u8;
                Binary::write_64be(&mut buf[2..10], pointer_id);
                Binary::write_posion(&mut buf[10..22], x, y, w, h);
                Binary::write_16be(&mut buf[22..24], pressure.to_bits());
                Binary::write_32be(&mut buf[24..28], action_button.bits());
                Binary::write_32be(&mut buf[28..32], buttons.bits());
                buf
            }
            ScrcpyControlMsg::InjectScrollEvent {
                x,
                y,
                w,
                h,
                hscroll,
                vscroll,
                buttons, // the buttons pressed when scrolling, just set as 0
            } => {
                let mut buf = vec![0; 21];
                buf[0] = ScrcpyControlMsgType::InjectScrollEvent as u8;
                Binary::write_posion(&mut buf[1..13], x, y, w, h);
                Binary::write_16be(&mut buf[13..15], hscroll);
                Binary::write_16be(&mut buf[15..17], vscroll);
                Binary::write_32be(&mut buf[17..21], buttons);
                buf
            }
            ScrcpyControlMsg::BackOrScreenOn { action } => {
                vec![ScrcpyControlMsgType::BackOrScreenOn as u8, action as u8]
            }
            ScrcpyControlMsg::GetClipboard { copy_key } => {
                vec![ScrcpyControlMsgType::GetClipboard as u8, copy_key as u8]
            }
            ScrcpyControlMsg::SetClipboard {
                sequence,
                paste, // u8
                text,
            } => {
                let mut buf: Vec<u8> = vec![0; 10];
                buf[0] = ScrcpyControlMsgType::SetClipboard as u8;
                Binary::write_64be(&mut buf[1..9], sequence);
                buf[9] = paste as u8;
                Binary::write_string(&text, SC_CONTROL_MSG_CLIPBOARD_TEXT_MAX_LENGTH, &mut buf);
                buf
            }
            ScrcpyControlMsg::SetDisplayPower { mode } => {
                vec![ScrcpyControlMsgType::SetDisplayPower as u8, mode as u8]
            }
            ScrcpyControlMsg::RotateDevice => {
                vec![ScrcpyControlMsgType::RotateDevice as u8]
            }
            ScrcpyControlMsg::ResetVideo => {
                vec![ScrcpyControlMsgType::ResetVideo as u8]
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ScrcpyDeviceMsg {
    Clipboard {
        length: u32,
        text: String,
    }, // 剪贴板内容
    AckClipboard {
        sequence: u64,
    }, // 设置剪切板响应
    UhidOutput {
        id: u16,
        size: u16,
        data: Vec<u8>,
    }, // 输入设备输出
    Rotation {
        rotation: u16,
        width: u32,
        height: u32,
        scid: String,
    }, // 屏幕旋转
    Unknown, // 未知消息
}

impl ScrcpyDeviceMsg {
    fn map_read_error(e: std::io::Error) -> String {
        format!("{}: {}", t!("scrcpy.failedReadControlConnection"), e)
    }

    pub async fn read_msg(read_half: &mut OwnedReadHalf, scid: String) -> Result<Self, String> {
        let message_type = read_half.read_u8().await.map_err(Self::map_read_error)?;

        match message_type {
            0 => {
                let length = read_half.read_u32().await.map_err(Self::map_read_error)?;
                let mut buf: Vec<u8> = vec![0; length as usize];
                read_half
                    .read_exact(&mut buf)
                    .await
                    .map_err(Self::map_read_error)?;
                let text = String::from_utf8(buf).map_err(|e| e.to_string())?;
                Ok(Self::Clipboard { length, text })
            }
            1 => {
                let sequence = read_half.read_u64().await.map_err(Self::map_read_error)?;
                Ok(Self::AckClipboard { sequence })
            }
            2 => {
                let id = read_half.read_u16().await.map_err(Self::map_read_error)?;
                let size = read_half.read_u16().await.map_err(Self::map_read_error)?;
                let mut data: Vec<u8> = vec![0; size as usize];
                read_half
                    .read_exact(&mut data)
                    .await
                    .map_err(Self::map_read_error)?;
                Ok(Self::UhidOutput { id, size, data })
            }
            3 => {
                let rotation = read_half.read_u16().await.map_err(Self::map_read_error)?;
                let width = read_half.read_u32().await.map_err(Self::map_read_error)?;
                let height = read_half.read_u32().await.map_err(Self::map_read_error)?;
                Ok(Self::Rotation {
                    rotation,
                    width,
                    height,
                    scid: scid.clone(),
                })
            }
            _ => Ok(Self::Unknown),
        }
    }
}
