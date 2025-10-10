use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::scrcpy::ScrcpyDevice;

static CONTROLLED_DEVICES: Lazy<RwLock<Vec<ScrcpyDevice>>> = Lazy::new(|| RwLock::new(Vec::new()));

pub struct ControlledDevice;

impl ControlledDevice {
    pub async fn get_device_list() -> Vec<ScrcpyDevice> {
        let device_list = CONTROLLED_DEVICES.read().await;
        device_list.clone()
    }

    pub async fn get_main_device() -> Option<ScrcpyDevice> {
        let device_list = CONTROLLED_DEVICES.read().await;
        device_list.iter().find(|device| device.main).cloned()
    }

    pub async fn is_any_device_controlled() -> bool {
        let device_list = CONTROLLED_DEVICES.read().await;
        !device_list.is_empty()
    }

    pub async fn is_scid_controlled(scid: &str) -> bool {
        let device_list = CONTROLLED_DEVICES.read().await;
        device_list.iter().any(|device| device.scid == scid)
    }

    pub async fn remove_device(scid: &str) {
        let mut device_list = CONTROLLED_DEVICES.write().await;
        device_list.retain(|device| device.scid != scid);
    }

    pub async fn add_device(device_id: String, scid: String, main: bool, socket_ids: Vec<String>) {
        let mut device_list = CONTROLLED_DEVICES.write().await;
        device_list.push(ScrcpyDevice::new(device_id, scid, main, socket_ids));
    }

    pub async fn update_device_name(scid: String, name: String) {
        let mut device_list = CONTROLLED_DEVICES.write().await;
        for device in device_list.iter_mut() {
            if device.scid == scid {
                device.name = name;
                return;
            }
        }
    }

    pub async fn update_device_size(scid: String, size: (u32, u32)) {
        let mut device_list = CONTROLLED_DEVICES.write().await;
        for device in device_list.iter_mut() {
            if device.scid == scid {
                device.device_size = size;
                return;
            }
        }
    }
}

#[derive(Clone, Serialize)]
pub struct UpdateInfo {
    pub has_update: bool,
    pub latest_version: String,
    pub current_version: String,
    pub title: String,
    pub body: String,
    pub time: String,
}

impl UpdateInfo {
    pub async fn get() -> UpdateInfo {
        UPDATE_INFO.read().await.clone()
    }

    pub async fn set(info: UpdateInfo) {
        *UPDATE_INFO.write().await = info;
    }
}

static UPDATE_INFO: Lazy<RwLock<UpdateInfo>> = Lazy::new(|| {
    RwLock::new(UpdateInfo {
        has_update: false,
        latest_version: "Unknown".to_string(),
        current_version: "Unknown".to_string(),
        title: "Unknown".to_string(),
        body: "Unknown".to_string(),
        time: "".to_string(),
    })
});
