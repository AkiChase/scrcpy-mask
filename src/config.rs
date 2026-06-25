use std::{
    fs::{File, create_dir_all},
    io::Write,
    net::Ipv4Addr,
    path::PathBuf,
    sync::RwLock,
};

use crate::{
    DEFAULT_LANGUAGE,
    scrcpy::media::{AudioCodec, VideoCodec},
    utils::{relate_to_data_path, relate_to_root_path},
};
use once_cell::sync::Lazy;
use paste::paste;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

static CONFIG: Lazy<RwLock<LocalConfig>> = Lazy::new(|| RwLock::default());

pub const AUDIO_BIT_RATE_MIN: u32 = 16_000;

fn default_web_bind_addr() -> Ipv4Addr {
    Ipv4Addr::new(127, 0, 0, 1)
}

fn bundled_adb_path() -> PathBuf {
    let adb_name = if cfg!(target_os = "windows") {
        "adb.exe"
    } else {
        "adb"
    };
    relate_to_root_path(["assets", "platform-tools", adb_name])
}

fn default_adb_path() -> String {
    let path = bundled_adb_path();
    if path.is_file() {
        path.to_string_lossy().into_owned()
    } else {
        "adb".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LocalConfig {
    // port
    pub web_port: u16,
    #[serde(default = "default_web_bind_addr")]
    pub web_bind_addr: Ipv4Addr,
    pub controller_port: u16,
    // adb
    pub adb_path: String,
    pub adb_connect_address: String,
    // mask
    pub always_on_top: bool,
    pub titlebar_visible: bool,
    pub vertical_mask_height: u32,
    pub horizontal_mask_width: u32,
    pub vertical_position: (i32, i32),
    pub horizontal_position: (i32, i32),
    // mapping
    pub active_mapping_file: String,
    pub mapping_label_opacity: f32,
    // language
    pub language: String,
    // clipboard sync
    pub clipboard_sync: bool,
    // video config
    pub video_codec: VideoCodec,
    pub video_bit_rate: u32,
    pub video_max_size: u32,
    pub video_max_fps: u32,
    // audio config
    pub audio_codec: AudioCodec,
    pub audio_bit_rate: u32,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            adb_path: default_adb_path(),
            adb_connect_address: String::new(),
            web_port: 27799,
            web_bind_addr: default_web_bind_addr(),
            controller_port: 27798,
            always_on_top: true,
            titlebar_visible: true,
            vertical_mask_height: 720,
            horizontal_mask_width: 1280,
            vertical_position: (100, 100),
            horizontal_position: (100, 100),
            active_mapping_file: "default.json".to_string(),
            mapping_label_opacity: 0.3,
            language: DEFAULT_LANGUAGE.to_string(),
            clipboard_sync: true,
            video_codec: VideoCodec::H264,
            video_bit_rate: 8_000000, // 8M
            video_max_size: 0,        // default no limit
            video_max_fps: 0,         // default no limit
            audio_codec: AudioCodec::Opus,
            audio_bit_rate: 128_000,
        }
    }
}

macro_rules! define_setter {
    ($(($field:ident, $typ:ty)),* $(,)?) => {
        paste! {
            $(
                pub fn [<set_ $field>] (value: $typ) {
                    CONFIG.write().unwrap().$field = value;
                    Self::save().unwrap();
                }
            )*
        }
    };
}

impl LocalConfig {
    pub fn prefer_bundled_adb() {
        let path = bundled_adb_path();
        if !path.is_file() {
            return;
        }

        let mut config = CONFIG.write().unwrap();
        if config.adb_path == "adb" {
            config.adb_path = path.to_string_lossy().into_owned();
        }
    }

    pub fn save() -> Result<(), String> {
        let config_json = to_string_pretty(&Self::get())
            .map_err(|e| format!("{}: {}", t!("localConfig.serializeConfigError"), e))?;

        let path = relate_to_data_path(["config.json"]);
        if let Some(parent) = path.parent() {
            create_dir_all(parent)
                .map_err(|e| format!("{}: {}", t!("localConfig.createConfigDirError"), e))?;
        }
        let mut file = File::create(path)
            .map_err(|e| format!("{}: {}", t!("localConfig.createConfigError"), e))?;
        file.write_all(config_json.as_bytes())
            .map_err(|e| format!("{}: {}", t!("localConfig.writeConfigError"), e))?;
        Ok(())
    }

    pub fn load() -> Result<(), String> {
        let path = relate_to_data_path(["config.json"]);
        let config_string = std::fs::read_to_string(&path).map_err(|e| {
            format!(
                "{} {}: {}",
                t!("localConfig.readConfigError"),
                path.to_str().unwrap(),
                e
            )
        })?;
        let config: LocalConfig = serde_json::from_str(&config_string)
            .map_err(|e| format!("{}: {}", t!("localConfig.serializeConfigError"), e))?;
        *CONFIG.write().unwrap() = config;
        Ok(())
    }

    pub fn get() -> LocalConfig {
        CONFIG.read().unwrap().clone()
    }

    pub fn get_clipboard_sync() -> bool {
        CONFIG.read().unwrap().clipboard_sync
    }

    define_setter!(
        (web_port, u16),
        (web_bind_addr, Ipv4Addr),
        (controller_port, u16),
        (adb_path, String),
        (adb_connect_address, String),
        (always_on_top, bool),
        (titlebar_visible, bool),
        (vertical_mask_height, u32),
        (horizontal_mask_width, u32),
        (vertical_position, (i32, i32)),
        (horizontal_position, (i32, i32)),
        (active_mapping_file, String),
        (mapping_label_opacity, f32),
        (language, String),
        (clipboard_sync, bool),
        (video_codec, VideoCodec),
        (video_bit_rate, u32),
        (video_max_size, u32),
        (video_max_fps, u32),
        (audio_codec, AudioCodec),
        (audio_bit_rate, u32),
    );
}
