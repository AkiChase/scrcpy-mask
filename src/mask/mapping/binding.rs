use std::str::FromStr;

use bevy::input::{
    gamepad::{GamepadAxis, GamepadButton},
    keyboard::{KeyCode, NativeKeyCode},
    mouse::MouseButton,
};
use bevy_ineffable::{
    bindings::{AnalogInput, BinaryInput, ChordLike, Threshold},
    prelude::{DualAxisBinding, InputBinding, SingleAxisBinding},
};
use rust_i18n::t;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum MergedButton {
    Mouse(MouseButton),
    ScrollDown,
    ScrollUp,
    Keyboard(KeyCode),
    GamePad(GamepadButton),
}

impl From<MouseButton> for MergedButton {
    fn from(value: MouseButton) -> Self {
        Self::Mouse(value)
    }
}

impl From<KeyCode> for MergedButton {
    fn from(value: KeyCode) -> Self {
        Self::Keyboard(value)
    }
}

impl From<GamepadButton> for MergedButton {
    fn from(value: GamepadButton) -> Self {
        Self::GamePad(value)
    }
}

impl From<MergedButton> for BinaryInput {
    fn from(input: MergedButton) -> Self {
        match input {
            MergedButton::ScrollDown => {
                BinaryInput::Axis(AnalogInput::ScrollWheelY, Threshold::preset_down())
            }
            MergedButton::ScrollUp => {
                BinaryInput::Axis(AnalogInput::ScrollWheelY, Threshold::preset_up())
            }
            MergedButton::Mouse(mouse_button) => BinaryInput::MouseButton(mouse_button),
            MergedButton::Keyboard(key_code) => BinaryInput::Key(key_code),
            MergedButton::GamePad(gamepad_button) => BinaryInput::Gamepad(gamepad_button),
        }
    }
}

macro_rules! match_gamepad_to_string {
    ($button:expr; $($variant:ident),* $(,)?) => {
        match $button {
            $(GamepadButton::$variant => format!("G-{}", stringify!($variant)),)*
            GamepadButton::Other(other) => format!("G-Other-{}", other),
        }
    };
}

macro_rules! match_keycode_to_string {
    ($key_code:expr; $($variant:ident),* $(,)?) => {
        match $key_code {
            $(KeyCode::$variant => stringify!($variant).to_string(),)*
            KeyCode::Unidentified(native_key_code) => match native_key_code {
                NativeKeyCode::Unidentified => "Unidentified".to_string(),
                NativeKeyCode::Android(code) => format!("Android-{}", code),
                NativeKeyCode::MacOS(code) => format!("MacOS-{}", code),
                NativeKeyCode::Windows(code) => format!("Windows-{}", code),
                NativeKeyCode::Xkb(code) => format!("Xkb-{}", code),
            },
        }
    };
}

impl ToString for MergedButton {
    fn to_string(&self) -> String {
        match self {
            MergedButton::Mouse(mouse_button) => match mouse_button {
                MouseButton::Left => "M-Left".to_string(),
                MouseButton::Right => "M-Right".to_string(),
                MouseButton::Middle => "M-Middle".to_string(),
                MouseButton::Back => "M-Back".to_string(),
                MouseButton::Forward => "M-Forward".to_string(),
                MouseButton::Other(other) => format!("M-Other-{}", other),
            },
            MergedButton::ScrollDown => "ScrollDown".to_string(),
            MergedButton::ScrollUp => "ScrollUp".to_string(),
            MergedButton::GamePad(gamepad_button) => match_gamepad_to_string!(gamepad_button;
                South, East, North, West, C, Z,
                LeftTrigger, LeftTrigger2, RightTrigger, RightTrigger2,
                Select, Start, Mode,
                LeftThumb, RightThumb,
                DPadUp, DPadDown, DPadLeft, DPadRight
            ),
            MergedButton::Keyboard(key_code) => match_keycode_to_string!(key_code;
                Backquote, Backslash, BracketLeft, BracketRight, Comma,
                Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9,
                Equal, IntlBackslash, IntlRo, IntlYen,
                KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ,
                KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT,
                KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
                Minus, Period, Quote, Semicolon, Slash,
                AltLeft, AltRight, Backspace, CapsLock, ContextMenu,
                ControlLeft, ControlRight, Enter, SuperLeft, SuperRight,
                ShiftLeft, ShiftRight, Space, Tab,
                Convert, KanaMode, Lang1, Lang2, Lang3, Lang4, Lang5, NonConvert,
                Delete, End, Help, Home, Insert, PageDown, PageUp,
                ArrowDown, ArrowLeft, ArrowRight, ArrowUp,
                NumLock, Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
                Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
                NumpadAdd, NumpadBackspace, NumpadClear, NumpadClearEntry, NumpadComma,
                NumpadDecimal, NumpadDivide, NumpadEnter, NumpadEqual, NumpadHash,
                NumpadMemoryAdd, NumpadMemoryClear, NumpadMemoryRecall,
                NumpadMemoryStore, NumpadMemorySubtract, NumpadMultiply,
                NumpadParenLeft, NumpadParenRight, NumpadStar, NumpadSubtract,
                Escape, Fn, FnLock, PrintScreen, ScrollLock, Pause,
                BrowserBack, BrowserFavorites, BrowserForward, BrowserHome, BrowserRefresh,
                BrowserSearch, BrowserStop, Eject, LaunchApp1, LaunchApp2, LaunchMail,
                MediaPlayPause, MediaSelect, MediaStop, MediaTrackNext, MediaTrackPrevious,
                Power, Sleep, AudioVolumeDown, AudioVolumeMute, AudioVolumeUp, WakeUp,
                Meta, Hyper, Turbo, Abort, Resume, Suspend, Again, Copy, Cut, Find, Open,
                Paste, Props, Select, Undo, Hiragana, Katakana,
                F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
                F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24, F25,
                F26, F27, F28, F29, F30, F31, F32, F33, F34, F35
            ),
        }
    }
}

macro_rules! match_gamepad_from_str {
    ($s:expr, $stripped:expr; $($variant:ident),* $(,)?) => {
        match $stripped {
            $(stringify!($variant) => Ok(MergedButton::GamePad(GamepadButton::$variant)),)*
            ss if ss.starts_with("Other-") => ss[6..]
                .parse()
                .map(|i| MergedButton::GamePad(GamepadButton::Other(i)))
                .map_err(|e| format!("{}: {}", t!("mask.mapping.invalidGamepadButtonCode"),e)),
            _ => Err(format!("{}: {}", t!("mask.mapping.unknownGamepadButton")
,$s)),
        }
    };
}

macro_rules! match_keycode_from_str {
    ($s:expr; $($variant:ident),* $(,)?) => {
        match $s {
            $(stringify!($variant) => Ok(MergedButton::Keyboard(KeyCode::$variant)),)*
            "Unidentified" => Ok(MergedButton::Keyboard(KeyCode::Unidentified(NativeKeyCode::Unidentified))),
            "ScrollDown" => Ok(MergedButton::ScrollDown),
            "ScrollUp" => Ok(MergedButton::ScrollUp),
            s if s.starts_with("Android-") => s[8..].parse()
                .map(|code| MergedButton::Keyboard(KeyCode::Unidentified(NativeKeyCode::Android(code))))
                .map_err(|e| format!("{}: {}", t!("mask.mapping.invalidCode", name=>"Android"), e)),
            s if s.starts_with("MacOS-") => s[6..].parse()
                .map(|code| MergedButton::Keyboard(KeyCode::Unidentified(NativeKeyCode::MacOS(code))))
                .map_err(|e| format!("{}: {}", t!("mask.mapping.invalidCode", name=>"MacOS"), e)),

            s if s.starts_with("Windows-") => s[8..].parse()
                .map(|code| MergedButton::Keyboard(KeyCode::Unidentified(NativeKeyCode::Windows(code))))
                .map_err(|e| format!("{}: {}", t!("mask.mapping.invalidCode", name=>"Windows"), e)),
            s if s.starts_with("Xkb-") => s[4..].parse()
                .map(|code| MergedButton::Keyboard(KeyCode::Unidentified(NativeKeyCode::Xkb(code))))
                .map_err(|e| format!("{}: {}", t!("mask.mapping.invalidCode", name=>"Xkb"), e)),
            _ => Err(format!("{}: {}", t!("mask.mapping.unknownKeyCode"),$s)),
        }
    };
}

impl FromStr for MergedButton {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = s.strip_prefix("M-") {
            match stripped {
                "Left" => Ok(MergedButton::Mouse(MouseButton::Left)),
                "Right" => Ok(MergedButton::Mouse(MouseButton::Right)),
                "Middle" => Ok(MergedButton::Mouse(MouseButton::Middle)),
                "Back" => Ok(MergedButton::Mouse(MouseButton::Back)),
                "Forward" => Ok(MergedButton::Mouse(MouseButton::Forward)),
                ss if ss.starts_with("Other-") => ss[6..]
                    .parse()
                    .map(|i| MergedButton::Mouse(MouseButton::Other(i)))
                    .map_err(|e| {
                        format!(
                            "{} MouseButton::Other, {}",
                            t!("mask.mapping.failedToParseTo"),
                            e
                        )
                    }),
                _ => Err(format!("{}: {}", t!("mask.mapping.unknownMouseButton"), s)),
            }
        } else if let Some(stripped) = s.strip_prefix("G-") {
            match_gamepad_from_str!(s, stripped;
                South, East, North, West, C, Z,
                LeftTrigger, LeftTrigger2, RightTrigger, RightTrigger2,
                Select, Start, Mode,
                LeftThumb, RightThumb,
                DPadUp, DPadDown, DPadLeft, DPadRight
            )
        } else {
            match_keycode_from_str!(s;
                Backquote, Backslash, BracketLeft, BracketRight, Comma,
                Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9,
                Equal, IntlBackslash, IntlRo, IntlYen,
                KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ,
                KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT,
                KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
                Minus, Period, Quote, Semicolon, Slash,
                AltLeft, AltRight, Backspace, CapsLock, ContextMenu,
                ControlLeft, ControlRight, Enter, SuperLeft, SuperRight,
                ShiftLeft, ShiftRight, Space, Tab,
                Convert, KanaMode, Lang1, Lang2, Lang3, Lang4, Lang5, NonConvert,
                Delete, End, Help, Home, Insert, PageDown, PageUp,
                ArrowDown, ArrowLeft, ArrowRight, ArrowUp,
                NumLock, Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
                Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
                NumpadAdd, NumpadBackspace, NumpadClear, NumpadClearEntry, NumpadComma,
                NumpadDecimal, NumpadDivide, NumpadEnter, NumpadEqual, NumpadHash,
                NumpadMemoryAdd, NumpadMemoryClear, NumpadMemoryRecall,
                NumpadMemoryStore, NumpadMemorySubtract, NumpadMultiply,
                NumpadParenLeft, NumpadParenRight, NumpadStar, NumpadSubtract,
                Escape, Fn, FnLock, PrintScreen, ScrollLock, Pause,
                BrowserBack, BrowserFavorites, BrowserForward, BrowserHome, BrowserRefresh,
                BrowserSearch, BrowserStop, Eject, LaunchApp1, LaunchApp2, LaunchMail,
                MediaPlayPause, MediaSelect, MediaStop, MediaTrackNext, MediaTrackPrevious,
                Power, Sleep, AudioVolumeDown, AudioVolumeMute, AudioVolumeUp, WakeUp,
                Meta, Hyper, Turbo, Abort, Resume, Suspend, Again, Copy, Cut, Find, Open,
                Paste, Props, Select, Undo, Hiragana, Katakana,
                F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
                F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24, F25,
                F26, F27, F28, F29, F30, F31, F32, F33, F34, F35
            )
        }
    }
}

impl Serialize for MergedButton {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for MergedButton {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MergedButtonVisitor;

        impl<'de> serde::de::Visitor<'de> for MergedButtonVisitor {
            type Value = MergedButton;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing a MergedButton")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                MergedButton::from_str(v).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(MergedButtonVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ButtonBinding(Vec<MergedButton>);

impl ButtonBinding {
    pub fn new(buttons: Vec<MergedButton>) -> Self {
        ButtonBinding(buttons)
    }
}

impl ToString for ButtonBinding {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join("+")
    }
}

impl From<ButtonBinding> for Vec<BinaryInput> {
    fn from(value: ButtonBinding) -> Self {
        value.0.into_iter().map(|input| input.into()).collect()
    }
}

impl From<ButtonBinding> for ChordLike {
    fn from(value: ButtonBinding) -> Self {
        match value.0.len() {
            0 => ChordLike::Single(value.0[0].clone().into()),
            _ => ChordLike::Multiple(value.0.iter().map(|input| input.clone().into()).collect()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum DirectionBinding {
    Button {
        up: ButtonBinding,
        down: ButtonBinding,
        left: ButtonBinding,
        right: ButtonBinding,
    },
    JoyStick {
        x: GamepadAxis,
        y: GamepadAxis,
    },
}

impl From<DirectionBinding> for InputBinding {
    fn from(value: DirectionBinding) -> Self {
        match value {
            DirectionBinding::Button {
                up,
                down,
                left,
                right,
            } => {
                DualAxisBinding::builder()
                    .set_x(
                        SingleAxisBinding::hold()
                            .set_negative(left)
                            .set_positive(right)
                            .build(),
                    )
                    .set_y(
                        SingleAxisBinding::hold()
                            .set_negative(up)
                            .set_positive(down)
                            .build(),
                    )
                    .build()
                    .0
            }
            DirectionBinding::JoyStick { x, y } => {
                DualAxisBinding::builder()
                    .set_x(
                        SingleAxisBinding::analog(AnalogInput::GamePad(x))
                            .set_sensitivity(1.0)
                            .build(),
                    )
                    .set_y(
                        SingleAxisBinding::analog(AnalogInput::GamePad(y))
                            .set_sensitivity(1.0)
                            .build(),
                    )
                    .build()
                    .0
            }
        }
    }
}

impl DirectionBinding {
    fn _gamepad_axis_name(axis: &GamepadAxis) -> String {
        match axis {
            GamepadAxis::LeftStickX => "LeftStickX".to_string(),
            GamepadAxis::LeftStickY => "LeftStickY".to_string(),
            GamepadAxis::RightStickX => "RightStickX".to_string(),
            GamepadAxis::RightStickY => "RightStickY".to_string(),
            GamepadAxis::LeftZ => "LeftZ".to_string(),
            GamepadAxis::RightZ => "RightZ".to_string(),
            GamepadAxis::Other(code) => format!("Other-{}", code),
        }
    }

    pub fn to_string_vec(&self) -> Vec<String> {
        match self {
            DirectionBinding::Button {
                up,
                down,
                left,
                right,
            } => vec![up, down, left, right]
                .iter()
                .map(|b| b.to_string())
                .collect(),
            DirectionBinding::JoyStick { x, y } => {
                let (x, y) = (Self::_gamepad_axis_name(x), Self::_gamepad_axis_name(y));
                if x[0..x.len() - 1] != y[0..y.len() - 1] {
                    vec![format!("{}+{}", x, y)]
                } else {
                    vec![x[0..x.len() - 1].to_string()]
                }
            }
        }
    }
}

pub trait ValidateMappingConfig {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}
