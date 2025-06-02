use crate::{
    adb::Device,
    client::ScrcpyClient,
    resource::{ResHelper, ResourceName},
    share,
    socket::connect_socket,
};
use std::{fs::read_to_string, sync::Arc};
use tauri::{Emitter, Listener, Manager};
use tauri_plugin_store::StoreExt;
use which::which;

#[tauri::command]
/// get devices info list
pub fn adb_devices() -> Result<Vec<Device>, String> {
    ScrcpyClient::adb_devices()
}

#[tauri::command]
pub async fn adb_kill_server() -> Result<(), String> {
    ScrcpyClient::adb_kill_server()
}

#[tauri::command]
/// forward local port to the device port
pub fn forward_server_port(id: String, scid: String, port: u16) -> Result<(), String> {
    ScrcpyClient::forward_server_port(&id, &scid, port)
}

#[tauri::command]
/// push scrcpy-server file to the device
pub fn push_server_file(id: String, app: tauri::AppHandle) -> Result<(), String> {
    let dir = app.path().resource_dir().unwrap().join("resource");
    ScrcpyClient::push_server_file(&dir, &id)
}

// TODO fix: device connect timeout
#[tauri::command]
/// start scrcpy server and connect to it
pub fn start_scrcpy_server(
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
        ScrcpyClient::shell_start_server(&id, &scid, &version).await;
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
        log::info!("Device reply channel created");
        tokio::spawn(async move {
            while let Some(reply) = device_reply_receiver.recv().await {
                share_app.emit("device-reply", reply).unwrap();
            }
            log::info!("Device reply channel closed");
        });

        // create channel to transmit front msg to TcpStream handler
        let (front_msg_sender, front_msg_receiver) = tokio::sync::mpsc::channel::<String>(16);
        let share_app = app.clone();
        let listen_handler = share_app.listen("front-command", move |event| {
            let sender = front_msg_sender.clone();
            // log::debug!("Received front-command: {}", event.payload());
            tokio::spawn(async move {
                if let Err(_) = sender.send(event.payload().into()).await {
                    log::error!("Forwarding front command failed, please restart the program!");
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
pub async fn get_cur_client_info() -> Result<Option<share::ClientInfo>, String> {
    let client_info = share::CLIENT_INFO.lock().unwrap();
    match &*client_info {
        Some(client) => Ok(Some(client.clone())),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn get_device_screen_size(id: String) -> Result<(u32, u32), String> {
    ScrcpyClient::get_device_screen_size(&id)
}

#[tauri::command]
pub fn adb_connect(address: String) -> Result<(), String> {
    ScrcpyClient::connect_device(&address)
}

#[tauri::command]
/// load default key mapping config file
pub async fn load_default_keyconfig(app: tauri::AppHandle) -> Result<String, String> {
    let dir = app.path().resource_dir().unwrap().join("resource");
    let file = ResHelper::get_file_path(&dir, ResourceName::DefaultKeyConfig);
    match read_to_string(file) {
        Ok(content) => Ok(content),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn check_adb_available() -> Result<String, String> {
    match which(&share::ADB_PATH.lock().unwrap().clone()) {
        Ok(p) => Ok(p.to_str().unwrap().to_string()),
        Err(_) => Err("adb not found".to_string()),
    }
}

#[tauri::command]
pub fn set_adb_path(adb_path: String, app: tauri::AppHandle) -> Result<(), String> {
    let store = app
        .store("store.bin")
        .map_err(|_| "failed to load store".to_string())?;
    store.set("adbPath", adb_path.clone());
    *share::ADB_PATH.lock().unwrap() = adb_path;
    Ok(())
}
