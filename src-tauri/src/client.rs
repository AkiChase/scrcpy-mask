use crate::{
    adb::{Adb, Device},
    resource::{ResHelper, ResourceName},
    share,
};
use std::path::PathBuf;

/**
 * the client of scrcpy
 */
#[derive(Debug)]
pub struct ScrcpyClient {
    pub device: Device,
    pub version: String,
    pub scid: String,
    pub port: u16,
}

impl ScrcpyClient {
    pub fn get_scrcpy_version() -> String {
        ResHelper::get_scrcpy_version()
    }

    pub fn adb_devices() -> Result<Vec<Device>, String> {
        let mut adb = Adb::new();
        adb.devices()
    }

    pub fn adb_kill_server() -> Result<(), String> {
        let mut adb = Adb::new();
        adb.kill_server()
    }

    /// get device screen size
    pub fn get_device_screen_size(id: &str) -> Result<(u32, u32), String> {
        Device::cmd_screen_size(id)
    }

    /// connect to wireless device
    pub fn connect_device(address: &str) -> Result<(), String> {
        let mut adb = Adb::new();
        adb.connect_device(address)
    }

    /// push server file to current device
    pub fn push_server_file(dir: &PathBuf, id: &str) -> Result<(), String> {
        let src = ResHelper::get_file_path(dir, ResourceName::ScrcpyServer);

        Device::push(
            id,
            &src.to_string_lossy(),
            "/data/local/tmp/scrcpy-server.jar",
        )?;
        log::info!("Successfully push server files: {}", src.to_str().unwrap());
        Ok(())
    }

    /// forward the local port to the device
    pub fn forward_server_port(id: &str, scid: &str, port: u16) -> Result<(), String> {
        Device::forward(
            id,
            &format!("tcp:{}", port),
            &format!("localabstract:scrcpy_{}", scid),
        )?;
        log::info!("Successfully forward port");
        Ok(())
    }

    /// reverse the device port to the local port
    pub fn reverse_server_port(id: &str, scid: &str, port: u16) -> Result<(), String> {
        Device::reverse(
            id,
            &format!("localabstract:scrcpy_{}", scid),
            &format!("tcp:{}", port),
        )?;
        log::info!("Successfully reverse port");
        Ok(())
    }

    /// spawn a new thread to start scrcpy server
    pub async fn shell_start_server(id: &str, scid: &str, version: &str) {
        log::info!("Starting scrcpy server...");

        let res = Device::shell_process(
            id,
            &[
                "CLASSPATH=/data/local/tmp/scrcpy-server.jar",
                "app_process",
                "/",
                "com.genymobile.scrcpy.Server",
                version,
                &format!("scid={}", scid),
                "tunnel_forward=true",
                "video=false",
                "audio=false",
            ],
        )
        .await;

        *share::CLIENT_INFO.lock().unwrap() = None;
        log::info!("Scrcpy server closed");

        match res {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                log::error!("{}", e);
            }
            Err(e) => {
                log::error!("JoinError: {}", e);
            }
        };
    }
}
