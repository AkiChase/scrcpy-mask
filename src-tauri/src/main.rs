// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use scrcpy_mask::{
    adb::{Adb, Device},
    client::ScrcpyClient,
    resource::ResHelper,
    socket::connect_socket,
};
use std::sync::Arc;
use tauri::Manager;

#[tauri::command]
/// get devices info list
fn adb_devices(app: tauri::AppHandle) -> Result<Vec<Device>, String> {
    let dir = app.path().resource_dir().unwrap().join("resource");
    match Adb::cmd_devices(&dir) {
        Ok(devices) => Ok(devices),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
/// forward local port to the device port
fn forward_server_port(
    app: tauri::AppHandle,
    id: String,
    scid: String,
    port: u16,
) -> Result<(), String> {
    let dir = app.path().resource_dir().unwrap().join("resource");

    match ScrcpyClient::forward_server_port(&dir, &id, &scid, port) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
/// push scrcpy-server file to the device
fn push_server_file(id: String, app: tauri::AppHandle) -> Result<(), String> {
    let dir = app.path().resource_dir().unwrap().join("resource");
    match ScrcpyClient::push_server_file(&dir, &id) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
/// start scrcpy server and connect to it
fn start_scrcpy_server(
    id: String,
    scid: String,
    address: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let dir = app.path().resource_dir().unwrap().join("resource");
    let version = ScrcpyClient::get_scrcpy_version();

    // start scrcpy server
    tokio::spawn(async move {
        ScrcpyClient::shell_start_server(&dir, &id, &scid, &version).unwrap();
    });

    // connect to scrcpy server
    tokio::spawn(async move {
        // wait 1 second for scrcpy-server to start
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let app = Arc::new(app);

        // create channel to transmit device reply to front
        let share_app = app.clone();
        let (device_reply_sender, mut device_reply_receiver) =
            tokio::sync::mpsc::channel::<String>(16);
        println!("device reply channel created");
        tokio::spawn(async move {
            while let Some(reply) = device_reply_receiver.recv().await {
                share_app.emit("device-reply", reply).unwrap();
            }
            println!("device reply channel closed");
        });

        // create channel to transmit front msg to TcpStream handler
        let (front_msg_sender, front_msg_receiver) = tokio::sync::mpsc::channel::<String>(16);
        let share_app = app.clone();
        let listen_handler = share_app.listen("front-command", move |event| {
            let sender = front_msg_sender.clone();
            // println!("收到front-command: {}", event.payload());
            tokio::spawn(async move {
                if let Err(e) = sender.send(event.payload().into()).await {
                    println!("front-command转发失败: {}", e);
                };
            });
        });

        // connect
        let share_app = app.clone();
        tokio::spawn(connect_socket(
            address,
            front_msg_receiver,
            device_reply_sender,
            listen_handler,
            share_app,
        ));
    });

    Ok(())
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // let main_window = app.get_webview_window("main").unwrap();
            // main_window
            //     .set_size(tauri::Size::Logical(tauri::LogicalSize {
            //         width: 1350.,
            //         height: 750.,
            //     }))
            //     .unwrap();

            // check resource files
            ResHelper::res_init(
                &app.path()
                    .resource_dir()
                    .expect("failed to find resource")
                    .join("resource"),
            )
            .unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            adb_devices,
            forward_server_port,
            push_server_file,
            start_scrcpy_server
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
