use tokio::sync::broadcast;

use crate::scrcpy::{
    constant::{KeyEventAction, Keycode, MetaState},
    control_msg::ScrcpyControlMsg,
};

/// Send a key Down + Up sequence immediately.
pub fn inject_keycode(cs_tx: &broadcast::Sender<ScrcpyControlMsg>, keycode: Keycode) {
    let _ = cs_tx.send(ScrcpyControlMsg::InjectKeycode {
        action: KeyEventAction::Down,
        keycode: keycode.clone(),
        repeat: 0,
        metastate: MetaState::NONE,
    });
    let _ = cs_tx.send(ScrcpyControlMsg::InjectKeycode {
        action: KeyEventAction::Up,
        keycode,
        repeat: 0,
        metastate: MetaState::NONE,
    });
}

/// Turn the device display on (mode: true) or off (mode: false).
pub fn set_display_power(cs_tx: &broadcast::Sender<ScrcpyControlMsg>, mode: bool) {
    let _ = cs_tx.send(ScrcpyControlMsg::SetDisplayPower { mode });
}
