use serde::Serialize;

pub mod adb;
pub mod connection;
pub mod constant;
pub mod control_msg;
pub mod controller;
pub mod media;

#[derive(Clone, Serialize, Debug)]
pub struct ScrcpyDevice {
    pub device_id: String,
    pub scid: String,
    pub socket_ids: Vec<String>,
    pub name: String,
    pub main: bool,
    pub device_size: (u32, u32),
}

impl ScrcpyDevice {
    pub fn new(device_id: String, scid: String, main: bool, socket_ids: Vec<String>) -> Self {
        Self {
            device_id,
            scid,
            socket_ids,
            name: "Unknow".to_string(),
            main,
            device_size: (0, 0),
        }
    }
}
