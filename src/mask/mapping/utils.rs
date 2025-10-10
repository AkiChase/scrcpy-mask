use std::ops::MulAssign;

use bevy::math::Vec2;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::scrcpy::{
    constant::{self, MotionEventAction, MotionEventButtons},
    control_msg::ScrcpyControlMsg,
};

pub const MIN_MOVE_STEP_LENGTH: f32 = 25.; // px
pub const MIN_MOVE_STEP_INTERVAL: u64 = 25; // ms

#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl From<Size> for Vec2 {
    fn from(size: Size) -> Self {
        Vec2::new(size.width as f32, size.height as f32)
    }
}

impl From<(u32, u32)> for Size {
    fn from((width, height): (u32, u32)) -> Self {
        Size { width, height }
    }
}

impl From<Vec2> for Size {
    fn from(vec: Vec2) -> Self {
        Size {
            width: vec.x as u32,
            height: vec.y as u32,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl From<(i32, i32)> for Position {
    fn from((x, y): (i32, i32)) -> Self {
        Position { x, y }
    }
}

impl From<Position> for Vec2 {
    fn from(pos: Position) -> Self {
        Vec2::new(pos.x as f32, pos.y as f32)
    }
}

impl MulAssign<Vec2> for Position {
    fn mul_assign(&mut self, rhs: Vec2) {
        self.x = ((self.x as f32) * rhs.x).round() as i32;
        self.y = ((self.y as f32) * rhs.y).round() as i32;
    }
}

pub struct ControlMsgHelper;

impl ControlMsgHelper {
    pub fn send_touch(
        cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
        action: MotionEventAction,
        pointer_id: u64,
        size: Vec2,
        pos: Vec2,
    ) {
        cs_tx
            .send(ScrcpyControlMsg::InjectTouchEvent {
                action,
                pointer_id,
                x: pos.x as i32,
                y: pos.y as i32,
                w: size.x as u16,
                h: size.y as u16,
                pressure: half::f16::from_f32_const(1.0),
                action_button: MotionEventButtons::PRIMARY,
                buttons: MotionEventButtons::PRIMARY,
            })
            .unwrap();
    }

    pub fn send_keycode(
        cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
        keycode: constant::Keycode,
        metastate: constant::MetaState,
        down: bool,
        repeat: u32,
    ) {
        let action = if down {
            constant::KeyEventAction::Down
        } else {
            constant::KeyEventAction::Up
        };
        cs_tx
            .send(ScrcpyControlMsg::InjectKeycode {
                action,
                keycode,
                repeat,
                metastate,
            })
            .unwrap();
    }

    pub fn set_clipboard(
        cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
        sequence: Option<u64>,
        text: String,
        paste: bool,
    ) {
        let sequence = sequence.unwrap_or_else(|| rand::random());
        cs_tx
            .send(ScrcpyControlMsg::SetClipboard {
                sequence,
                paste,
                text,
            })
            .unwrap();
    }
}

pub fn ease_sigmoid_like(t: f32) -> f32 {
    1.0 / (1.0 + (-12.0 * (t - 0.5)).exp())
}
