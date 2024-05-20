use anyhow::{Ok, Result};
use std::{io::BufRead, path::PathBuf};

use crate::{
    adb::{Adb, Device},
    resource::{ResHelper, ResourceName},
    share,
};

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

    pub fn adb_devices() -> Result<Vec<Device>> {
        Adb::cmd_devices()
    }

    pub fn adb_restart_server() -> Result<()> {
        Adb::cmd_kill_server()?;
        Adb::cmd_start_server()?;
        Ok(())
    }

    pub fn adb_reverse_remove() -> Result<()> {
        Adb::cmd_reverse_remove()
    }

    pub fn adb_forward_remove() -> Result<()> {
        Adb::cmd_forward_remove()
    }

    // get the screen size of the device
    pub fn get_device_screen_size(id: &str) -> Result<(u32, u32)> {
        Device::cmd_screen_size(id)
    }

    /// push server file to current device
    pub fn push_server_file(dir: &PathBuf, id: &str) -> Result<()> {
        let info = Device::cmd_push(
            id,
            &ResHelper::get_file_path(dir, ResourceName::ScrcpyServer).to_string_lossy(),
            "/data/local/tmp/scrcpy-server.jar",
        )?;

        println!("{}\nSuccessfully push server files", info);
        Ok(())
    }

    /// forward the local port to the device
    pub fn forward_server_port(id: &str, scid: &str, port: u16) -> Result<()> {
        Device::cmd_forward(
            id,
            &format!("tcp:{}", port),
            &format!("localabstract:scrcpy_{}", scid),
        )?;
        println!("Successfully forward port");
        Ok(())
    }

    /// reverse the device port to the local port
    pub fn reverse_server_port(id: &str, scid: &str, port: u16) -> Result<()> {
        Device::cmd_reverse(
            id,
            &format!("localabstract:scrcpy_{}", scid),
            &format!("tcp:{}", port),
        )?;
        println!("Successfully reverse port");
        Ok(())
    }

    /// spawn a new thread to start scrcpy server
    pub fn shell_start_server(id: &str, scid: &str, version: &str) -> Result<()> {
        let mut child = Device::cmd_shell(
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
        )?;

        println!("Starting scrcpy server...");
        let out = child.stdout.take().unwrap();
        let mut out = std::io::BufReader::new(out);
        let mut s = String::new();

        while let core::result::Result::Ok(_) = out.read_line(&mut s) {
            // break at the end of program
            if let core::result::Result::Ok(Some(_)) = child.try_wait() {
                break;
            }
            print!("{}", s);
            // clear string to store new line only
            s.clear();
        }

        *share::CLIENT_INFO.lock().unwrap() = None;
        println!("Scrcpy server closed");
        Ok(())
    }
}
