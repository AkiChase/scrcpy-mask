mod platform_specific;

pub mod window_helper {
    #[cfg(target_os = "macos")]
    pub use crate::platform_specific::macos::window_helper::*;
    #[cfg(target_os = "windows")]
    pub use crate::platform_specific::windows::window_helper::*;
}


pub mod adb;
pub mod resource;
pub mod client;
pub mod socket;