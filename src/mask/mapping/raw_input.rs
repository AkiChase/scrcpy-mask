use std::time::Instant;

use bevy::{
    ecs::{
        event::EventReader,
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    input::{
        ButtonInput, ButtonState,
        keyboard::{KeyCode, KeyboardInput},
        mouse::MouseButton,
    },
    platform::collections::HashMap,
    state::state::NextState,
};
use bevy_ineffable::prelude::{Ineffable, InputBinding, PulseBinding};
use copypasta::{ClipboardContext, ClipboardProvider};
use rust_i18n::t;
use serde::{Deserialize, Serialize};

use crate::{
    mask::mapping::{
        MappingState,
        binding::{ButtonBinding, ValidateMappingConfig},
        config::ActiveMappingConfig,
        utils::{ControlMsgHelper, Position},
    },
    scrcpy::constant,
    utils::ChannelSenderCS,
};

pub fn raw_input_init(mut commands: Commands) {
    commands.insert_resource(RepeatCountMap::default());
    commands.insert_resource(RightMouseHoldInstant::default());
}

#[derive(Debug, Clone)]
pub struct BindMappingRawInput {
    pub note: String,
    pub position: Position,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
}

impl From<MappingRawInput> for BindMappingRawInput {
    fn from(value: MappingRawInput) -> Self {
        Self {
            note: value.note,
            position: value.position,
            bind: value.bind.clone(),
            input_binding: PulseBinding::just_released(value.bind).0, // use release to trigger
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingRawInput {
    pub note: String,
    pub position: Position,
    pub bind: ButtonBinding,
}

impl ValidateMappingConfig for MappingRawInput {}

pub fn handle_raw_input(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    mut next_state: ResMut<NextState<MappingState>>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, _) in &active_mapping.mappings {
            if action.as_ref().starts_with("RawInput") {
                if ineffable.just_pulsed(action.ineff_pulse()) {
                    next_state.set(MappingState::RawInput);
                    log::info!("[Mapping] {}", t!("mask.mapping.rawInputModeHint"));
                    return;
                }
            }
        }
    }
}

fn get_metastate(button_input: &Res<ButtonInput<KeyCode>>) -> constant::MetaState {
    let mut metastate = constant::MetaState::NONE;
    if button_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        metastate |= constant::MetaState::CTRL_ON;
    }
    if button_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        metastate |= constant::MetaState::SHIFT_ON;
    }
    if button_input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
        metastate |= constant::MetaState::ALT_ON;
    }
    // ignore SCROLL_LOCK/NUM_LOCK/CAPS_LOCK state
    metastate
}

fn get_keycode(key_code: KeyCode) -> Option<constant::Keycode> {
    Some(match key_code {
        KeyCode::Backquote => constant::Keycode::Grave,
        KeyCode::Backslash => constant::Keycode::Backslash,
        KeyCode::BracketLeft => constant::Keycode::LeftBracket,
        KeyCode::BracketRight => constant::Keycode::RightBracket,
        KeyCode::Comma => constant::Keycode::Comma,
        KeyCode::Digit0 => constant::Keycode::Keycode0,
        KeyCode::Digit1 => constant::Keycode::Keycode1,
        KeyCode::Digit2 => constant::Keycode::Keycode2,
        KeyCode::Digit3 => constant::Keycode::Keycode3,
        KeyCode::Digit4 => constant::Keycode::Keycode4,
        KeyCode::Digit5 => constant::Keycode::Keycode5,
        KeyCode::Digit6 => constant::Keycode::Keycode6,
        KeyCode::Digit7 => constant::Keycode::Keycode7,
        KeyCode::Digit8 => constant::Keycode::Keycode8,
        KeyCode::Digit9 => constant::Keycode::Keycode9,
        KeyCode::Equal => constant::Keycode::Equals,
        KeyCode::IntlBackslash => constant::Keycode::Backslash,
        KeyCode::IntlRo => constant::Keycode::Ro,
        KeyCode::IntlYen => constant::Keycode::Yen,
        KeyCode::KeyA => constant::Keycode::A,
        KeyCode::KeyB => constant::Keycode::B,
        KeyCode::KeyC => constant::Keycode::C,
        KeyCode::KeyD => constant::Keycode::D,
        KeyCode::KeyE => constant::Keycode::E,
        KeyCode::KeyF => constant::Keycode::F,
        KeyCode::KeyG => constant::Keycode::G,
        KeyCode::KeyH => constant::Keycode::H,
        KeyCode::KeyI => constant::Keycode::I,
        KeyCode::KeyJ => constant::Keycode::J,
        KeyCode::KeyK => constant::Keycode::K,
        KeyCode::KeyL => constant::Keycode::L,
        KeyCode::KeyM => constant::Keycode::M,
        KeyCode::KeyN => constant::Keycode::N,
        KeyCode::KeyO => constant::Keycode::O,
        KeyCode::KeyP => constant::Keycode::P,
        KeyCode::KeyQ => constant::Keycode::Q,
        KeyCode::KeyR => constant::Keycode::R,
        KeyCode::KeyS => constant::Keycode::S,
        KeyCode::KeyT => constant::Keycode::T,
        KeyCode::KeyU => constant::Keycode::U,
        KeyCode::KeyV => constant::Keycode::V,
        KeyCode::KeyW => constant::Keycode::W,
        KeyCode::KeyX => constant::Keycode::X,
        KeyCode::KeyY => constant::Keycode::Y,
        KeyCode::KeyZ => constant::Keycode::Z,
        KeyCode::Minus => constant::Keycode::Minus,
        KeyCode::Period => constant::Keycode::Period,
        KeyCode::Quote => constant::Keycode::Apostrophe,
        KeyCode::Semicolon => constant::Keycode::Semicolon,
        KeyCode::Slash => constant::Keycode::Slash,
        KeyCode::AltLeft => constant::Keycode::AltLeft,
        KeyCode::AltRight => constant::Keycode::AltRight,
        KeyCode::Backspace => constant::Keycode::Del,
        KeyCode::CapsLock => constant::Keycode::CapsLock,
        KeyCode::ControlLeft => constant::Keycode::CtrlLeft,
        KeyCode::ControlRight => constant::Keycode::CtrlRight,
        KeyCode::Enter => constant::Keycode::Enter,
        KeyCode::SuperLeft => constant::Keycode::MetaLeft,
        KeyCode::SuperRight => constant::Keycode::MetaRight,
        KeyCode::ShiftLeft => constant::Keycode::ShiftLeft,
        KeyCode::ShiftRight => constant::Keycode::ShiftRight,
        KeyCode::Space => constant::Keycode::Space,
        KeyCode::Tab => constant::Keycode::Tab,
        KeyCode::Delete => constant::Keycode::ForwardDel,
        KeyCode::End => constant::Keycode::MoveEnd,
        KeyCode::Help => constant::Keycode::Help,
        KeyCode::Home => constant::Keycode::MoveHome,
        KeyCode::Insert => constant::Keycode::Insert,
        KeyCode::PageDown => constant::Keycode::PageDown,
        KeyCode::PageUp => constant::Keycode::PageUp,
        KeyCode::ArrowDown => constant::Keycode::DpadDown,
        KeyCode::ArrowLeft => constant::Keycode::DpadLeft,
        KeyCode::ArrowRight => constant::Keycode::DpadRight,
        KeyCode::ArrowUp => constant::Keycode::DpadUp,
        KeyCode::NumLock => constant::Keycode::NumLock,
        KeyCode::Numpad0 => constant::Keycode::Numpad0,
        KeyCode::Numpad1 => constant::Keycode::Numpad1,
        KeyCode::Numpad2 => constant::Keycode::Numpad2,
        KeyCode::Numpad3 => constant::Keycode::Numpad3,
        KeyCode::Numpad4 => constant::Keycode::Numpad4,
        KeyCode::Numpad5 => constant::Keycode::Numpad5,
        KeyCode::Numpad6 => constant::Keycode::Numpad6,
        KeyCode::Numpad7 => constant::Keycode::Numpad7,
        KeyCode::Numpad8 => constant::Keycode::Numpad8,
        KeyCode::Numpad9 => constant::Keycode::Numpad9,
        KeyCode::NumpadAdd => constant::Keycode::NumpadAdd,
        KeyCode::NumpadComma => constant::Keycode::NumpadComma,
        KeyCode::NumpadDecimal => constant::Keycode::NumpadDot,
        KeyCode::NumpadDivide => constant::Keycode::NumpadDivide,
        KeyCode::NumpadEnter => constant::Keycode::NumpadEnter,
        KeyCode::NumpadEqual => constant::Keycode::NumpadEquals,
        KeyCode::NumpadMultiply => constant::Keycode::NumpadMultiply,
        KeyCode::NumpadParenLeft => constant::Keycode::NumpadLeftParen,
        KeyCode::NumpadParenRight => constant::Keycode::NumpadRightParen,
        KeyCode::NumpadSubtract => constant::Keycode::NumpadSubtract,
        KeyCode::Escape => constant::Keycode::Escape,
        KeyCode::Fn => constant::Keycode::Function,
        KeyCode::PrintScreen => constant::Keycode::Sysrq,
        KeyCode::ScrollLock => constant::Keycode::ScrollLock,
        KeyCode::Pause => constant::Keycode::Break,
        KeyCode::MediaPlayPause => constant::Keycode::MediaPlayPause,
        KeyCode::MediaStop => constant::Keycode::MediaStop,
        KeyCode::MediaTrackNext => constant::Keycode::MediaNext,
        KeyCode::MediaTrackPrevious => constant::Keycode::MediaPrevious,
        KeyCode::Power => constant::Keycode::Power,
        KeyCode::Sleep => constant::Keycode::Sleep,
        KeyCode::AudioVolumeDown => constant::Keycode::VolumeDown,
        KeyCode::AudioVolumeMute => constant::Keycode::Mute,
        KeyCode::AudioVolumeUp => constant::Keycode::VolumeUp,
        KeyCode::WakeUp => constant::Keycode::Wakeup,
        KeyCode::NumpadBackspace => constant::Keycode::Del,
        KeyCode::NumpadClear => constant::Keycode::Clear,
        KeyCode::NumpadClearEntry => constant::Keycode::Clear,
        KeyCode::NumpadHash => constant::Keycode::Pound,
        KeyCode::NumpadStar => constant::Keycode::Star,
        KeyCode::Copy => constant::Keycode::Copy,
        KeyCode::Cut => constant::Keycode::Cut,
        KeyCode::Paste => constant::Keycode::Paste,
        KeyCode::F1 => constant::Keycode::F1,
        KeyCode::F2 => constant::Keycode::F2,
        KeyCode::F3 => constant::Keycode::F3,
        KeyCode::F4 => constant::Keycode::F4,
        KeyCode::F5 => constant::Keycode::F5,
        KeyCode::F6 => constant::Keycode::F6,
        KeyCode::F7 => constant::Keycode::F7,
        KeyCode::F8 => constant::Keycode::F8,
        KeyCode::F9 => constant::Keycode::F9,
        KeyCode::F10 => constant::Keycode::F10,
        KeyCode::F11 => constant::Keycode::F11,
        KeyCode::F12 => constant::Keycode::F12,
        _ => return None,
    })
}

pub fn on_enter_raw_input_mode(
    mut repeat_count_map: ResMut<RepeatCountMap>,
    mut right_hold_instant: ResMut<RightMouseHoldInstant>,
    mut key_evnts: EventReader<KeyboardInput>,
) {
    key_evnts.clear();
    repeat_count_map.0.clear();
    right_hold_instant.0 = None;
}

pub fn on_exit_raw_input_mode(
    mut repeat_count_map: ResMut<RepeatCountMap>,
    mut right_hold_instant: ResMut<RightMouseHoldInstant>,
) {
    repeat_count_map.0.clear();
    right_hold_instant.0 = None;
}

#[derive(Resource, Default)]
pub struct RightMouseHoldInstant(Option<Instant>);
pub fn handle_exit_raw_input_mode(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut right_hold_instant: ResMut<RightMouseHoldInstant>,
    mut next_state: ResMut<NextState<MappingState>>,
) {
    let now = Instant::now();

    if mouse_input.just_pressed(MouseButton::Right) {
        right_hold_instant.0 = Some(now);
    } else if mouse_input.just_released(MouseButton::Right) {
        right_hold_instant.0 = None;
    }

    if let Some(start) = right_hold_instant.0 {
        if now.duration_since(start).as_secs_f32() >= 1.0 {
            right_hold_instant.0 = None;
            next_state.set(MappingState::Normal);
            log::info!("[Mapping] {}", t!("mask.mapping.exitRawInputMode"));
        }
    }
}

#[derive(Resource, Default)]
pub struct RepeatCountMap(HashMap<KeyCode, u32>);

pub fn handle_raw_input_trigger(
    mut key_evnts: EventReader<KeyboardInput>,
    mut repeat_count_map: ResMut<RepeatCountMap>,
    button_input: Res<ButtonInput<KeyCode>>,
    cs_tx_res: Res<ChannelSenderCS>,
) {
    if button_input.pressed(KeyCode::ControlLeft) || button_input.pressed(KeyCode::ControlRight) {
        if button_input.just_pressed(KeyCode::KeyV) {
            let mut ctx = ClipboardContext::new().unwrap();
            if let Ok(text) = ctx.get_contents() {
                ControlMsgHelper::set_clipboard(&cs_tx_res.0, None, text, true);
            }
            key_evnts.clear();
            return;
        } else if button_input.just_released(KeyCode::KeyV) {
            key_evnts.clear();
            return;
        }
    }

    for ev in key_evnts.read() {
        if let Some(keycode) = get_keycode(ev.key_code) {
            let metastate = get_metastate(&button_input);
            let (repeat, down) = match ev.state {
                ButtonState::Pressed => {
                    let repeat = repeat_count_map.0.entry(ev.key_code).or_insert(0);
                    let repeat_val = *repeat;
                    *repeat += 1;
                    (repeat_val, true)
                }
                ButtonState::Released => {
                    repeat_count_map.0.remove(&ev.key_code);
                    (0, false)
                }
            };
            ControlMsgHelper::send_keycode(&cs_tx_res.0, keycode, metastate, down, repeat);
        }
    }
}
