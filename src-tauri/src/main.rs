// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use scrcpy_mask::{
    adb::{Adb, Device},
    client::ScrcpyClient,
    resource::{ResHelper, ResourceName},
    share,
    socket::connect_socket,
};
use std::{fs::read_to_string, sync::Arc};
use tauri::Manager;

#[tauri::command]
/// get devices info list
fn adb_devices() -> Result<Vec<Device>, String> {
    match Adb::cmd_devices() {
        Ok(devices) => Ok(devices),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
/// forward local port to the device port
fn forward_server_port(id: String, scid: String, port: u16) -> Result<(), String> {
    match ScrcpyClient::forward_server_port(&id, &scid, port) {
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
    let mut client_info = share::CLIENT_INFO.lock().unwrap();
    if let Some(_) = &*client_info {
        return Err("client already exists".to_string());
    }

    *client_info = Some(share::ClientInfo::new(
        "unknow".to_string(),
        id.clone(),
        scid.clone(),
    ));

    let version = ScrcpyClient::get_scrcpy_version();

    // start scrcpy server
    tokio::spawn(async move {
        ScrcpyClient::shell_start_server(&id, &scid, &version).unwrap();
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
                if let Err(_) = sender.send(event.payload().into()).await {
                    println!("front-command forwarding failure, please restart the program !");
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

#[tauri::command]
fn get_cur_client_info() -> Result<Option<share::ClientInfo>, String> {
    let client_info = share::CLIENT_INFO.lock().unwrap();
    match &*client_info {
        Some(client) => Ok(Some(client.clone())),
        None => Ok(None),
    }
}

#[tauri::command]
/// get device screen size
fn get_device_screen_size(id: String) -> Result<(u32, u32), String> {
    match ScrcpyClient::get_device_screen_size(&id) {
        Ok(size) => Ok(size),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
/// connect to wireless device
fn adb_connect(address: String) -> Result<String, String> {
    match Adb::cmd_connect(&address) {
        Ok(res) => Ok(res),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
/// load default key mapping config file
fn load_default_keyconfig(app: tauri::AppHandle) -> Result<String, String> {
    let dir = app.path().resource_dir().unwrap().join("resource");
    let file = ResHelper::get_file_path(&dir, ResourceName::DefaultKeyConfig);
    match read_to_string(file) {
        Ok(content) => Ok(content),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn check_adb_available() -> Result<(), String> {
    match Adb::cmd_base().output() {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn set_adb_path(adb_path: String, app: tauri::AppHandle) -> Result<(), String> {
    let app_h = app.app_handle().clone();
    let stores = app_h.state::<tauri_plugin_store::StoreCollection<tauri::Wry>>();
    let path = std::path::PathBuf::from("store.bin");
    let store_res: Result<(), tauri_plugin_store::Error> =
        tauri_plugin_store::with_store(app, stores, path, |store| {
            store.insert(
                "adbPath".to_string(),
                serde_json::Value::String(adb_path.clone()),
            )?;
            *share::ADB_PATH.lock().unwrap() = adb_path;
            Ok(())
        });

    match store_res {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let stores = app
                .app_handle()
                .state::<tauri_plugin_store::StoreCollection<tauri::Wry>>();
            let path: std::path::PathBuf = std::path::PathBuf::from("store.bin");
            tauri_plugin_store::with_store(app.app_handle().clone(), stores, path, |store| {
                // load adb path
                match store.get("adbPath") {
                    Some(value) => {
                        *share::ADB_PATH.lock().unwrap() = value.as_str().unwrap().to_string()
                    }
                    None => store
                        .insert(
                            "adbPath".to_string(),
                            serde_json::Value::String("adb".to_string()),
                        )
                        .unwrap(),
                };

                // restore window position and size
                match store.get("maskArea") {
                    Some(value) => {
                        let pos_x = value["posX"].as_i64();
                        let pos_y = value["posY"].as_i64();
                        let size_w = value["sizeW"].as_i64().unwrap_or(800);
                        let size_h = value["sizeH"].as_i64().unwrap_or(600);

                        let main_window: tauri::WebviewWindow =
                            app.get_webview_window("main").unwrap();

                        main_window.set_zoom(1.).unwrap_or(());

                        if pos_x.is_none() || pos_y.is_none() {
                            main_window.center().unwrap_or(());
                        } else {
                            main_window
                                .set_position(tauri::Position::Logical(tauri::LogicalPosition {
                                    x: (pos_x.unwrap() - 70) as f64,
                                    y: (pos_y.unwrap() - 30) as f64,
                                }))
                                .unwrap();
                        }

                        main_window
                            .set_size(tauri::Size::Logical(tauri::LogicalSize {
                                width: (size_w + 70) as f64,
                                height: (size_h + 30) as f64,
                            }))
                            .unwrap();
                    }
                    None => {
                        let main_window: tauri::WebviewWindow =
                            app.get_webview_window("main").unwrap();

                        main_window.center().unwrap_or(());

                        main_window
                            .set_size(tauri::Size::Logical(tauri::LogicalSize {
                                width: (800 + 70) as f64,
                                height: (600 + 30) as f64,
                            }))
                            .unwrap();
                    }
                }

                Ok(())
            })
            .unwrap();

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
            start_scrcpy_server,
            get_cur_client_info,
            get_device_screen_size,
            adb_connect,
            load_default_keyconfig,
            check_adb_available,
            set_adb_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
