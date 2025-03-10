// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use scrcpy_mask::{command, resource::ResHelper, share};
use tauri::Manager;
use tauri_plugin_store::StoreExt;

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
            let store = app
                .store("store.bin")
                .map_err(|_| "failed to load store".to_string())?;

            // set adb path
            match store.get("adbPath") {
                Some(value) => {
                    *share::ADB_PATH.lock().unwrap() = value.as_str().unwrap().to_string()
                }
                None => store.set("adbPath", "adb".to_string()),
            }

            // restore window position and size
            match store.get("maskArea") {
                Some(value) => {
                    // TODO check position and size validity

                    let pos_x = value["posX"].as_i64();
                    let pos_y = value["posY"].as_i64();
                    let size_w = value["sizeW"].as_i64().unwrap_or(800);
                    let size_h = value["sizeH"].as_i64().unwrap_or(600);

                    let main_window: tauri::WebviewWindow = app.get_webview_window("main").unwrap();

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
                    let main_window: tauri::WebviewWindow = app.get_webview_window("main").unwrap();

                    main_window.center().unwrap_or(());

                    main_window
                        .set_size(tauri::Size::Logical(tauri::LogicalSize {
                            width: (800 + 70) as f64,
                            height: (600 + 30) as f64,
                        }))
                        .unwrap();
                }
            }

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
