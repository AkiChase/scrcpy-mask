// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use scrcpy_mask::{command, resource::ResHelper};
use tauri::Manager;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let main_window: tauri::WebviewWindow = app.get_webview_window("main").unwrap();
            main_window.set_zoom(1.).unwrap_or(());

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
            command::adb_devices,
            command::adb_restart_server,
            command::forward_server_port,
            command::push_server_file,
            command::start_scrcpy_server,
            command::get_cur_client_info,
            command::get_device_screen_size,
            command::adb_connect,
            command::load_default_keyconfig,
            command::check_adb_available,
            command::set_adb_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
