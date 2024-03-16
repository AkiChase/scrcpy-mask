// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio::{
    runtime::Runtime,
    select,
    sync::{broadcast, watch::Sender},
};

use scrcpy_mask::{
    adb::{Adb, Device},
    client::ScrcpyClient,
    resource::ResHelper,
    socket::Server,
    window_helper::{get_window_control_list, get_window_list, WindowInfo},
};

lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
    static ref SERVER_KILLER: Arc<Mutex<Option<Sender<bool>>>> = Default::default();
}

#[tauri::command]
// get windows info list
fn get_windows() -> Vec<WindowInfo> {
    get_window_list()
}

#[tauri::command]
// get controls info list of the window
fn get_window_controls(hwnd: isize) -> Vec<WindowInfo> {
    get_window_control_list(hwnd)
}

#[tauri::command]
/// get devices info list
fn adb_devices(app: tauri::AppHandle) -> Result<Vec<Device>, String> {
    let dir = app.path_resolver().resolve_resource("resource").unwrap();
    match Adb::cmd_devices(&dir) {
        Ok(devices) => Ok(devices),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
/// get screen size of the device
fn get_screen_size(id: String, app: tauri::AppHandle) -> Result<(u16, u16), String> {
    let dir = app.path_resolver().resolve_resource("resource").unwrap();
    match ScrcpyClient::get_screen_size(&dir, &id) {
        Ok(size) => Ok(size),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
/// reverse the device port to the local port
fn reverse_server_port(
    app: tauri::AppHandle,
    id: String,
    scid: String,
    port: u16,
) -> Result<(), String> {
    let dir = app.path_resolver().resolve_resource("resource").unwrap();

    match ScrcpyClient::reverse_server_port(&dir, &id, &scid, port) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
/// push scrcpy-server file to the device
fn push_server_file(id: String, app: tauri::AppHandle) -> Result<(), String> {
    let dir = app.path_resolver().resolve_resource("resource").unwrap();
    match ScrcpyClient::push_server_file(&dir, &id) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
/// open client socket
fn open_socket_server(port: u16, app: tauri::AppHandle) -> Result<(), String> {
    // if server is running, return
    if SERVER_KILLER.lock().unwrap().is_some() {
        println!("The socket server has already started");
        return Err("The socket server has already started".into());
    }

    let app = Arc::new(app);

    let share_app = app.clone();
    let (device_reply_sender, mut device_reply_receiver) = tokio::sync::mpsc::channel::<String>(16);
    println!("device reply channel created");
    RUNTIME.spawn(async move {
        while let Some(reply) = device_reply_receiver.recv().await {
            share_app.emit_all("device-reply", reply).unwrap();
        }
    });

    // create new global watch channel sender
    let (killer_send, mut killer_receiver) = tokio::sync::watch::channel(false);
    *SERVER_KILLER.lock().unwrap() = Some(killer_send);

    let (fc_broadcast_sender, _receiver) = broadcast::channel::<String>(16);
    let fc_broadcast_sender_clone = fc_broadcast_sender.clone();

    let share_app = app.clone();
    share_app.listen_global("front-command", move |event| {
        println!("收到front-command: {}", event.payload().unwrap_or(""));
        // 广播前端命令到所有Socket处理器中
        if let Err(e) = fc_broadcast_sender_clone.send(event.payload().unwrap_or("").into()) {
            println!("front-command广播失败: {}", e);
        };
    });

    RUNTIME.spawn(async move {
        select! {
            _ = async {
                let server = Server::bind(port).await;
                loop{
                    // create channel to receive msg in TcpStream task
                    server.accept(fc_broadcast_sender.subscribe(), device_reply_sender.clone()).await;
                }
            } => {}
            _ = async {
                loop {
                    // close server when receiving msg
                    if killer_receiver.changed().await.is_ok() {
                        *SERVER_KILLER.lock().unwrap() = None;
                        println!("Close socket server listening to 127.0.0.1:{port}");
                        return
                    }
                }
            } => {}
        }
    });
    Ok(())
}

#[tauri::command]
fn close_socket_server() -> Result<(), String> {
    let server_killer = SERVER_KILLER.lock().unwrap();
    if let Some(sender) = &*server_killer {
        sender.send(true).unwrap();
        Ok(())
    } else {
        Err("The socket server is not running".into())
    }
}

#[tauri::command]
/// start scrcpy server
fn start_scrcpy_server(id: String, scid: String, app: tauri::AppHandle) -> Result<(), String> {
    let dir = app.path_resolver().resolve_resource("resource").unwrap();
    let version = ScrcpyClient::get_scrcpy_version();

    RUNTIME.spawn_blocking(move || {
        match ScrcpyClient::shell_start_server(&dir, &id, &scid, &version) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    });
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // check resource files
            ResHelper::res_init(
                &app.path_resolver()
                    .resolve_resource("resource")
                    .expect("failed to resolve resource"),
            )
            .unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_windows,
            get_window_controls,
            adb_devices,
            get_screen_size,
            reverse_server_port,
            push_server_file,
            open_socket_server,
            close_socket_server,
            start_scrcpy_server
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
