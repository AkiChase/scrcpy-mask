use std::{
    io::BufRead,
    process::{Child, Command, Stdio},
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use anyhow::{Context, Ok, Result};

use crate::share;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Device {
    pub id: String,
    pub status: String,
}

impl Device {
    /// execute "adb push" to push file from src to des
    pub fn cmd_push(id: &str, src: &str, des: &str) -> Result<String> {
        let mut adb_command = Adb::cmd_base();
        let res = adb_command
            .args(&["-s", id, "push", src, des])
            .output()
            .with_context(|| format!("Failed to execute 'adb push {} {}'", src, des))?;
        Ok(String::from_utf8(res.stdout).unwrap())
    }

    /// execute "adb reverse" to reverse the device port to local port
    pub fn cmd_reverse(id: &str, remote: &str, local: &str) -> Result<()> {
        let mut adb_command = Adb::cmd_base();
        adb_command
            .args(&["-s", id, "reverse", remote, local])
            .output()
            .with_context(|| format!("Failed to execute 'adb reverse {} {}'", remote, local))?;
        Ok(())
    }

    /// execute "adb forward" to forward the local port to the device
    pub fn cmd_forward(id: &str, local: &str, remote: &str) -> Result<()> {
        let mut adb_command = Adb::cmd_base();
        adb_command
            .args(&["-s", id, "forward", local, remote])
            .output()
            .with_context(|| format!("Failed to execute 'adb forward {} {}'", local, remote))?;
        Ok(())
    }

    /// execute "adb shell" to execute shell command on the device
    pub fn cmd_shell(id: &str, shell_args: &[&str]) -> Result<Child> {
        let mut adb_command = Adb::cmd_base();
        let mut args = vec!["-s", id, "shell"];
        args.extend_from_slice(shell_args);
        Ok(adb_command
            .args(args)
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to execute 'adb shell'")?)
    }

    /// execute "adb shell wm size" to get screen size
    pub fn cmd_screen_size(id: &str) -> Result<(u32, u32)> {
        let mut adb_command = Adb::cmd_base();
        let output = adb_command
            .args(&["-s", id, "shell", "wm", "size"])
            .output()
            .context("Failed to execute 'adb shell wm size'")?;

        for line in output.stdout.lines() {
            if let std::result::Result::Ok(line) = line {
                if line.starts_with("Physical size: ") {
                    let size_str = line.trim_start_matches("Physical size: ").split('x');
                    let width = size_str.clone().next().unwrap().parse::<u32>().unwrap();
                    let height = size_str.clone().last().unwrap().parse::<u32>().unwrap();
                    return Ok((width, height));
                }
            }
        }
        Err(anyhow::anyhow!("Failed to get screen size"))
    }
}

pub struct Adb;

/// Module to execute adb command and fetch output.
/// But some output of command won't be output, like adb service startup information.
impl Adb {
    pub fn cmd_base() -> Command {
        let adb_path = share::ADB_PATH.lock().unwrap().clone();
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new(adb_path);
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            return cmd;
        }
        #[cfg(not(target_os = "windows"))]
        Command::new(adb_path)
    }

    /// execute "adb devices" and return devices list
    pub fn cmd_devices() -> Result<Vec<Device>> {
        let mut adb_command = Adb::cmd_base();
        let output = adb_command
            .args(&["devices"])
            .output()
            .context("Failed to execute 'adb devices'")?;

        let mut devices_vec: Vec<Device> = Vec::new();
        let mut lines = output.stdout.lines();
        // skip first line
        lines.next();

        // parse string to Device
        for line in lines {
            if let std::result::Result::Ok(s) = line {
                let device_info: Vec<&str> = s.split('\t').collect();
                if device_info.len() == 2 {
                    devices_vec.push(Device {
                        id: device_info[0].to_string(),
                        status: device_info[1].to_string(),
                    });
                }
            }
        }
        Ok(devices_vec)
    }

    /// execute "adb kill-server"
    pub fn cmd_kill_server() -> Result<()> {
        let mut adb_command = Adb::cmd_base();
        adb_command
            .args(&["kill-server"])
            .output()
            .context("Failed to execute 'adb kill-server'")?;
        Ok(())
    }

    /// execute "adb reverse --remove-all"
    pub fn cmd_reverse_remove() -> Result<()> {
        let mut adb_command = Adb::cmd_base();
        adb_command
            .args(&["reverse", " --remove-all"])
            .output()
            .context("Failed to execute 'adb reverse --remove-all'")?;
        Ok(())
    }

    /// execute "adb forward --remove-all"
    pub fn cmd_forward_remove() -> Result<()> {
        let mut adb_command = Adb::cmd_base();
        adb_command
            .args(&["forward", " --remove-all"])
            .output()
            .context("Failed to execute 'adb forward --remove-all'")?;
        Ok(())
    }

    /// execute "adb start-server"
    pub fn cmd_start_server() -> Result<()> {
        let mut adb_command = Adb::cmd_base();
        adb_command
            .args(&["start-server"])
            .output()
            .context("Failed to execute 'adb start-server'")?;
        Ok(())
    }

    pub fn cmd_connect(address: &str) -> Result<String> {
        let mut adb_command = Adb::cmd_base();
        let output = adb_command
            .args(&["connect", address])
            .output()
            .context(format!("Failed to execute 'adb connect {}'", address))?;

        let res = String::from_utf8(output.stdout)?;
        Ok(res)
    }
}
