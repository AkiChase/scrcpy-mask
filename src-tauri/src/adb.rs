use crate::resource::{ResHelper, ResourceName};
use std::{
    io::BufRead,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use anyhow::{Context, Ok, Result};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Device {
    pub id: String,
    pub status: String,
}

impl Device {
    /// execute "adb push" to push file from src to des
    pub fn cmd_push(res_dir: &PathBuf, id: &str, src: &str, des: &str) -> Result<String> {
        let mut adb_command = Adb::cmd_base(res_dir);
        let res = adb_command
            .args(&["-s", id, "push", src, des])
            .output()
            .with_context(|| format!("Failed to execute 'adb push {} {}'", src, des))?;
        Ok(String::from_utf8(res.stdout).unwrap())
    }

    /// execute "adb reverse" to reverse the device port to local port
    pub fn cmd_reverse(res_dir: &PathBuf, id: &str, remote: &str, local: &str) -> Result<()> {
        let mut adb_command = Adb::cmd_base(res_dir);
        adb_command
            .args(&["-s", id, "reverse", remote, local])
            .output()
            .with_context(|| format!("Failed to execute 'adb reverse {} {}'", remote, local))?;
        Ok(())
    }

    /// execute "adb forward" to forward the local port to the device
    pub fn cmd_forward(res_dir: &PathBuf, id: &str, local: &str, remote: &str) -> Result<()> {
        let mut adb_command = Adb::cmd_base(res_dir);
        adb_command
            .args(&["-s", id, "forward", local, remote])
            .output()
            .with_context(|| format!("Failed to execute 'adb forward {} {}'", local, remote))?;
        Ok(())
    }

    /// execute "adb shell" to execute shell command on the device
    pub fn cmd_shell(res_dir: &PathBuf, id: &str, shell_args: &[&str]) -> Result<Child> {
        let mut adb_command = Adb::cmd_base(res_dir);
        let mut args = vec!["-s", id, "shell"];
        args.extend_from_slice(shell_args);
        Ok(adb_command
            .args(args)
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to execute 'adb shell'")?)
    }
}

pub struct Adb;

/// Module to execute adb command and fetch output.
/// But some output of command won't be output, like adb service startup information.
impl Adb {
    fn cmd_base(res_dir: &PathBuf) -> Command {
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new(ResHelper::get_file_path(res_dir, ResourceName::Adb));
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            cmd
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new(ResHelper::get_file_path(res_dir, ResourceName::Adb))
        }
    }

    /// execute "adb devices" and return devices list
    pub fn cmd_devices(res_dir: &PathBuf) -> Result<Vec<Device>> {
        let mut adb_command = Adb::cmd_base(res_dir);
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
    pub fn cmd_kill_server(res_dir: &PathBuf) -> Result<()> {
        let mut adb_command = Adb::cmd_base(res_dir);
        adb_command
            .args(&["kill-server"])
            .output()
            .context("Failed to execute 'adb kill-server'")?;
        Ok(())
    }

    /// execute "adb reverse --remove-all"
    pub fn cmd_reverse_remove(res_dir: &PathBuf) -> Result<()> {
        let mut adb_command = Adb::cmd_base(res_dir);
        adb_command
            .args(&["reverse", " --remove-all"])
            .output()
            .context("Failed to execute 'adb reverse --remove-all'")?;
        Ok(())
    }

    /// execute "adb forward --remove-all"
    pub fn cmd_forward_remove(res_dir: &PathBuf) -> Result<()> {
        let mut adb_command = Adb::cmd_base(res_dir);
        adb_command
            .args(&["forward", " --remove-all"])
            .output()
            .context("Failed to execute 'adb forward --remove-all'")?;
        Ok(())
    }

    /// execute "adb start-server"
    pub fn cmd_start_server(res_dir: &PathBuf) -> Result<()> {
        let mut adb_command = Adb::cmd_base(res_dir);
        adb_command
            .args(&["start-server"])
            .output()
            .context("Failed to execute 'adb start-server'")?;
        Ok(())
    }
}
