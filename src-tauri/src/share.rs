use lazy_static::lazy_static;
use std::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ClientInfo {
    pub device_name: String,
    pub device_id: String,
    pub scid: String,
    pub width: i32,
    pub height: i32,
}

impl ClientInfo {
    pub fn new(device_name: String, device_id: String, scid: String) -> Self {
        Self {
            device_name,
            device_id,
            scid,
            width: 0,
            height: 0,
        }
    }

    pub fn set_size(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }
}

lazy_static! {
    pub static ref CLIENT_INFO: Mutex<Option<ClientInfo>> = Mutex::new(None);
}

lazy_static! {
    pub static ref ADB_PATH: Mutex<String> = Mutex::new(String::from("adb"));
}
