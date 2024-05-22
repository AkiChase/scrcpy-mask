use lazy_static::lazy_static;
use std::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ClientInfo {
    pub device_name: String,
    pub device_id: String,
    pub scid: String,
}

impl ClientInfo {
    pub fn new(device_name: String, device_id: String, scid: String) -> Self {
        Self {
            device_name,
            device_id,
            scid,
        }
    }
}

lazy_static! {
    pub static ref CLIENT_INFO: Mutex<Option<ClientInfo>> = Mutex::new(None);
}

lazy_static! {
    pub static ref ADB_PATH: Mutex<String> = Mutex::new(String::from("adb"));
}
