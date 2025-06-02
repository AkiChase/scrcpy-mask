use adb_client::{ADBDeviceExt, ADBServer, ADBServerDevice};
use tokio::{sync::mpsc, task::JoinHandle};

use std::{
    fs::File,
    io::Cursor,
    net::{Ipv4Addr, SocketAddrV4},
    path::Path,
};

use crate::{share, utils::ChannelWriter};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Device {
    pub id: String,
    pub status: String,
}

impl Device {
    fn new_server_device(id: &str) -> ADBServerDevice {
        ADBServerDevice::new(id.to_string(), None)
    }

    pub fn push(id: &str, src: &str, des: &str) -> Result<(), String> {
        let mut device = Device::new_server_device(id);
        let mut input = File::open(Path::new(src))
            .map_err(|e| format!("Failed to open file '{}': {}", src, e))?;
        device
            .push(&mut input, des)
            .map_err(|e| format!("Failed to push file '{}' to '{}': {}", src, des, e))?;
        Ok(())
    }

    pub fn reverse(id: &str, remote: &str, local: &str) -> Result<(), String> {
        let mut device = Device::new_server_device(id);
        device
            .reverse(remote.to_string(), local.to_string())
            .map_err(|e| format!("Failed to reverse '{}' to '{}': {}", remote, local, e))
    }

    pub fn forward(id: &str, local: &str, remote: &str) -> Result<(), String> {
        let mut device = Device::new_server_device(id);
        device
            .forward(remote.to_string(), local.to_string())
            .map_err(|e| format!("Failed to forward '{}' to '{}': {}", local, remote, e))
    }

    pub fn shell_process(id: &str, shell_args: &[&str]) -> JoinHandle<Result<(), String>> {
        let mut device = Device::new_server_device(id);
        let shell_args: Vec<String> = shell_args.iter().map(|&s| s.to_string()).collect();

        let (tx, mut rx) = mpsc::unbounded_channel();
        let h: JoinHandle<Result<(), String>> = tokio::task::spawn_blocking(move || {
            let mut writer = ChannelWriter { sender: tx };
            let shell_args: Vec<&str> = shell_args.iter().map(|s| s.as_str()).collect();
            device
                .shell_command(&shell_args, &mut writer)
                .map_err(|e| format!("Failed to run adb shell command: {}", e))
        });

        tokio::spawn(async move {
            while let Some(line) = rx.recv().await {
                log::info!("{}", line);
            }
        });

        h
    }

    pub fn cmd_screen_size(id: &str) -> Result<(u32, u32), String> {
        let mut device = Device::new_server_device(id);

        let mut output: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&mut output);
        device
            .shell_command(&["wm", "size"], &mut cursor)
            .map_err(|e| format!("Failed to run adb shell command: {}", e))?;

        let stdout = String::from_utf8_lossy(&output);
        for line in stdout.lines() {
            if let Some(rest) = line.strip_prefix("Physical size: ") {
                let mut parts = rest.trim().split('x');
                let width = parts
                    .next()
                    .ok_or("Missing width")?
                    .parse::<u32>()
                    .map_err(|e| format!("Failed to parse width: {}", e))?;

                let height = parts
                    .next()
                    .ok_or("Missing height")?
                    .parse::<u32>()
                    .map_err(|e| format!("Failed to parse height: {}", e))?;
                return Ok((width, height));
            }
        }
        Err("Failed to get screen size".to_string())
    }
}

pub struct Adb {
    pub server: ADBServer,
}

/// Module to execute adb command and fetch output.
/// But some output of command won't be output, like adb service startup information.
impl Adb {
    // pub fn cmd_base() -> Command {
    //     let adb_path = share::ADB_PATH.lock().unwrap().clone();
    //     #[cfg(target_os = "windows")]
    //     {
    //         let mut cmd = Command::new(adb_path);
    //         cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    //         return cmd;
    //     }
    //     #[cfg(not(target_os = "windows"))]
    //     Command::new(adb_path)
    // }

    pub fn new() -> Adb {
        let adb_path = share::ADB_PATH.lock().unwrap().clone();
        Self {
            server: ADBServer::new_from_path(
                SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5037),
                Some(adb_path),
            ),
        }
    }

    pub fn devices(&mut self) -> Result<Vec<Device>, String> {
        let device = self.server.devices().map_err(|e| e.to_string())?;
        Ok(device
            .iter()
            .map(|d| Device {
                id: d.identifier.clone(),
                status: d.state.to_string(),
            })
            .collect::<Vec<_>>())
    }

    pub fn kill_server(&mut self) -> Result<(), String> {
        self.server
            .kill()
            .map_err(|e| format!("Failed to kill adb server': {}", e))
    }

    pub fn connect_device(&mut self, address: &str) -> Result<(), String> {
        let socket_addr = address
            .parse::<SocketAddrV4>()
            .map_err(|e| format!("Failed to parse device address: {}", e))?;

        self.server
            .connect_device(socket_addr)
            .map_err(|e| format!("Failed to connect to device '{}': {}", address, e))
    }
}
