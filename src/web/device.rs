use std::time::Duration;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use rand::Rng;
use rust_i18n::t;
use serde::Deserialize;
use serde_json::json;
use tokio::{
    sync::{broadcast, mpsc::UnboundedSender},
    time::sleep,
};

use crate::{
    config::LocalConfig,
    scrcpy::{
        adb::{Adb, Device},
        constant::Keycode,
        control_msg::ScrcpyControlMsg,
        controller::ControllerCommand,
        device_action,
        media::AudioCodec,
    },
    utils::{relate_to_root_path, share::ControlledDevice},
    web::{JsonResponse, WebServerError, ws::WebSocketNotification},
};

const SCRCPY_SERVER_VERSION: &str = "4.0";

#[derive(Debug, Clone)]
pub struct AppStateDevice {
    cs_tx: broadcast::Sender<ScrcpyControlMsg>,
    d_tx: UnboundedSender<ControllerCommand>,
    ws_tx: broadcast::Sender<WebSocketNotification>,
}

pub fn routers(
    cs_tx: broadcast::Sender<ScrcpyControlMsg>,
    d_tx: UnboundedSender<ControllerCommand>,
    ws_tx: broadcast::Sender<WebSocketNotification>,
) -> Router {
    Router::new()
        .route("/device_list", get(device_list))
        .route("/control_device", post(control_device))
        .route("/decontrol_device", post(decontrol_device))
        .route("/reconnect_device", post(reconnect_device))
        .route("/adb_connect", post(adb_connect))
        .route("/adb_pair", post(adb_pair))
        .route("/adb_restart", post(adb_restart))
        .route("/adb_screenshot", post(adb_screenshot))
        .route("/control/set_display_power", post(set_display_power))
        .route("/control/set_pointer_location", post(set_pointer_location))
        .route("/control/send_key", post(send_key))
        .with_state(AppStateDevice { cs_tx, d_tx, ws_tx })
}

async fn device_list() -> Result<JsonResponse, WebServerError> {
    let controlled_devices = ControlledDevice::get_device_list().await;
    let config = LocalConfig::get();
    let all_devices = Adb::new(config.adb_path)
        .devices()
        .map_err(|e| WebServerError::internal_error(e))?;

    Ok(JsonResponse::success(
        t!("web.device.deviceListObtained"),
        Some(json!({
            "controlled_devices": controlled_devices,
            "adb_devices": all_devices,
        })),
    ))
}

fn gen_scid() -> String {
    let mut rng = rand::rng();
    let suffix: String = (0..6)
        .map(|_| rng.random_range(1..=9).to_string())
        .collect();
    format!("10{}", suffix) // ensure 8 digits(HEX) and less than MAX_INT32
}

#[derive(Deserialize)]
struct PostDataControlDevice {
    device_id: String,
    display_id: i32,
    video: bool,
    #[serde(default)]
    audio: bool,
}

async fn _control_device(
    device_id: &str,
    display_id: i32,
    video: bool,
    audio: bool,
    d_tx: &UnboundedSender<ControllerCommand>,
    ws_tx: &broadcast::Sender<WebSocketNotification>,
) -> Result<JsonResponse, WebServerError> {
    let device_id = device_id.to_string();
    let local_config = LocalConfig::get();

    let device_list = ControlledDevice::get_device_list().await;
    // check if device is controlled
    if device_list
        .iter()
        .any(|device| device.device_id == device_id)
    {
        return Err(WebServerError::bad_request(format!(
            "{}: {}",
            t!("web.device.alreadyControlled"),
            device_id
        )));
    }
    let main = device_list.len() == 0;
    let audio = audio && main;

    // prepare for scrcpy app
    let scid = gen_scid();
    let scrcpy_path = relate_to_root_path([
        "assets",
        &format!("scrcpy-mask-server-v{}", SCRCPY_SERVER_VERSION),
    ]);
    Device::push(
        &device_id,
        scrcpy_path.to_str().unwrap(),
        "/data/local/tmp/scrcpy-server.jar",
    )
    .map_err(WebServerError::internal_error)?;
    log::info!("[WebServe] {}", t!("web.device.pushScrcpyServerSuccess"));

    let remote = format!("localabstract:scrcpy_{}", scid);
    let local = format!("tcp:{}", local_config.controller_port);
    Device::reverse(&device_id, &remote, &local).map_err(WebServerError::internal_error)?;
    log::info!(
        "[WebServe] {}",
        t!("web.device.reverseSuccess", remote => remote, local => local)
    );

    let mut args = [
        "CLASSPATH=/data/local/tmp/scrcpy-server.jar",
        "app_process",
        "/",
        "com.genymobile.scrcpy.Server",
    ]
    .iter_mut()
    .map(|arg| arg.to_string())
    .collect::<Vec<String>>();

    args.push(SCRCPY_SERVER_VERSION.to_string());
    args.push(format!("scid={}", scid));
    args.push(format!("video={}", video));
    args.push(format!("display_id={}", display_id));
    args.push(format!("audio={}", audio));
    args.push(format!("stay_awake={}", local_config.stay_awake));
    args.push(format!(
        "screen_off_timeout={}",
        local_config.screen_off_timeout
    ));
    args.push(format!(
        "power_off_on_close={}",
        local_config.power_off_on_close
    ));

    // create device
    let mut socket_id: Vec<String> = Vec::new();
    let mut commands: Vec<ControllerCommand> = Vec::new();
    if main {
        let mut meta_flag = true;
        if video {
            socket_id.push("main_video".to_string());
            commands.push(ControllerCommand::ConnectMainVideo(scid.clone(), meta_flag));
            if meta_flag {
                meta_flag = false;
            }

            // video shell args
            args.push(format!("video_codec={}", local_config.video_codec));
            args.push(format!("video_bit_rate={}", local_config.video_bit_rate));
            if local_config.video_max_size > 0 {
                args.push(format!("max_size={}", local_config.video_max_size));
            }
            if local_config.video_max_fps > 0 {
                args.push(format!("max_fps={}", local_config.video_max_fps));
            }
        }
        if audio {
            socket_id.push("main_audio".to_string());
            commands.push(ControllerCommand::ConnectMainAudio(scid.clone(), meta_flag));
            if meta_flag {
                meta_flag = false;
            }

            args.push(format!("audio_codec={}", local_config.audio_codec));
            args.push(format!("audio_source={}", local_config.audio_source));
            args.push(format!(
                "audio_dup={}",
                local_config.audio_source.is_playback() && local_config.audio_dup
            ));
            if !matches!(local_config.audio_codec, AudioCodec::Raw) {
                args.push(format!("audio_bit_rate={}", local_config.audio_bit_rate));
            }
        }
        socket_id.push("main_control".to_string());
        commands.push(ControllerCommand::ConnectMainControl(
            scid.clone(),
            meta_flag,
        ));
    } else {
        socket_id.push(format!("sub_control_{}", scid));
        commands.push(ControllerCommand::ConnectSubControl(scid.clone()));
    }

    ControlledDevice::add_device(device_id.clone(), scid.clone(), main, socket_id).await;
    // send command to controller server
    for cmd in commands {
        d_tx.send(cmd).unwrap();
    }

    // run scrcpy app
    sleep(Duration::from_millis(500)).await;
    log::info!("[WebServe] {}", t!("web.device.startingScrcpyApp"));

    let h = Device::shell_process(&device_id, args);

    let scid_copy = scid.clone();
    let ws_tx_copy = ws_tx.clone();
    tokio::spawn(async move {
        h.await.unwrap().unwrap();
        log::info!("[WebServe] {}", t!("web.device.removingDeviceAfterExit"));
        ControlledDevice::remove_device(&scid_copy).await;
        ws_tx_copy
            .send(WebSocketNotification::ScrcpyDeviceConnection {
                scid: scid_copy,
                main,
                connected: false,
            })
            .ok();
    });

    Ok(JsonResponse::success(
        t!("web.device.tryStartingScrcpy"),
        Some(json!({"scid": scid, "device_id": device_id})),
    ))
}

async fn control_device(
    State(state): State<AppStateDevice>,
    Json(payload): Json<PostDataControlDevice>,
) -> Result<JsonResponse, WebServerError> {
    let device_id = payload.device_id;
    let video = payload.video;
    let audio = payload.audio;
    let display_id = payload.display_id;

    _control_device(
        &device_id,
        display_id,
        video,
        audio,
        &state.d_tx,
        &state.ws_tx,
    )
    .await
}

#[derive(Deserialize)]
struct PostDataReconnectDevice {
    device_id: String,
    display_id: i32,
    video: bool,
    #[serde(default)]
    audio: bool,
}

async fn reconnect_device(
    State(state): State<AppStateDevice>,
    Json(payload): Json<PostDataReconnectDevice>,
) -> Result<JsonResponse, WebServerError> {
    let device_id = payload.device_id;
    let device_list = ControlledDevice::get_device_list().await;
    for device in device_list {
        if device.device_id == device_id {
            _decontrol_device(&device_id, &state.d_tx).await?;
            _control_device(
                &device_id,
                payload.display_id,
                payload.video,
                payload.audio,
                &state.d_tx,
                &state.ws_tx,
            )
            .await?;
            return Ok(JsonResponse::success(
                format!("{}: {}", t!("web.device.reconnectDevice"), device_id),
                None,
            ));
        }
    }
    Err(WebServerError::bad_request(format!(
        "{}: {}",
        t!("web.device.deviceNotFound"),
        device_id
    )))
}

#[derive(Deserialize)]
struct PostDataDeControlDevice {
    device_id: String,
}

async fn _decontrol_device(
    device_id: &str,
    d_tx: &UnboundedSender<ControllerCommand>,
) -> Result<JsonResponse, WebServerError> {
    let device_list = ControlledDevice::get_device_list().await;
    for device in device_list {
        if device.device_id == device_id {
            let scid = device.scid.clone();
            if device.main {
                d_tx.send(ControllerCommand::ShutdownMain(scid)).unwrap();
            } else {
                d_tx.send(ControllerCommand::ShutdownSub(scid)).unwrap();
            }
            ControlledDevice::remove_device(&device.scid).await;
            return Ok(JsonResponse::success(
                format!("{}: {}", t!("web.device.decontrolDevice"), device_id),
                None,
            ));
        }
    }
    Err(WebServerError::bad_request(format!(
        "{}: {}",
        t!("web.device.deviceNotFound"),
        device_id
    )))
}

async fn decontrol_device(
    State(state): State<AppStateDevice>,
    Json(payload): Json<PostDataDeControlDevice>,
) -> Result<JsonResponse, WebServerError> {
    let device_id = payload.device_id;
    _decontrol_device(&device_id, &state.d_tx).await
}

#[derive(Deserialize)]
struct PostDataAddress {
    address: String,
}

async fn adb_connect(Json(payload): Json<PostDataAddress>) -> Result<JsonResponse, WebServerError> {
    let config = LocalConfig::get();
    let address = payload.address.trim().to_string();
    match Adb::new(config.adb_path).connect_device(&address) {
        Ok(_) => Ok(JsonResponse::success(
            format!("{}", t!("web.device.adbConnect", address => address)),
            None,
        )),
        Err(e) => Err(WebServerError::bad_request(format!(
            "{}: {}",
            t!("web.device.adbConnectFailed", address => address),
            e
        ))),
    }
}

#[derive(Deserialize)]
struct PostDataAdbPair {
    address: String,
    code: String,
}

async fn adb_pair(Json(payload): Json<PostDataAdbPair>) -> Result<JsonResponse, WebServerError> {
    let config = LocalConfig::get();
    match Adb::new(config.adb_path).pair_device(&payload.address, &payload.code) {
        Ok(_) => Ok(JsonResponse::success(
            format!(
                "{}",
                t!("web.device.adbPairSuccess", address => payload.address, code => payload.code)
            ),
            None,
        )),
        Err(e) => Err(WebServerError::bad_request(format!(
            "{}: {}",
            t!("web.device.adbPairFailed", address => payload.address, code => payload.code),
            e
        ))),
    }
}

async fn adb_restart() -> Result<JsonResponse, WebServerError> {
    let controlled_devices = ControlledDevice::get_device_list().await;
    let config = LocalConfig::get();
    match Adb::new(config.adb_path).restart_server() {
        Ok(adb_devices) => Ok(JsonResponse::success(
            t!("web.device.adbRestartSuccess"),
            Some(json!({
                "controlled_devices": controlled_devices,
                "adb_devices": adb_devices,
            })),
        )),
        Err(e) => Err(WebServerError::internal_error(format!(
            "{}: {}",
            t!("web.device.adbRestartFailed"),
            e
        ))),
    }
}

#[derive(Deserialize)]
struct PostDataId {
    id: String,
}

async fn adb_screenshot(
    Json(payload): Json<PostDataId>,
) -> Result<impl IntoResponse, WebServerError> {
    let src = "/data/local/tmp/_screenshot_scrcpy_mask.png";

    let mut display_id_info = Vec::new();
    Device::shell(
        &payload.id,
        ["dumpsys", "SurfaceFlinger", "--display-id"],
        &mut display_id_info,
    )
    .map_err(|e| WebServerError::bad_request(format!("failed get display id: {}", e)))?;
    let text = String::from_utf8_lossy(&display_id_info);
    let first_line = text
        .lines()
        .next()
        .ok_or_else(|| WebServerError::bad_request("no display found"))?;
    let display_id = first_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| WebServerError::bad_request("invalid display line"))?;

    Device::shell(
        &payload.id,
        ["screencap", "-p", "-d", display_id, src],
        &mut std::io::stdout(),
    )
    .map_err(|e| {
        WebServerError::bad_request(format!(
            "{} {}: {}",
            t!("web.device.screenshotError"),
            payload.id,
            e
        ))
    })?;

    let mut image_bytes = Vec::<u8>::new();
    Device::pull(&payload.id, src.to_string(), &mut image_bytes).map_err(|e| {
        WebServerError::bad_request(format!(
            "{}: {}",
            t!("web.device.failedGetScreenshotFile"),
            e
        ))
    })?;

    Device::shell(&payload.id, ["rm", src], &mut std::io::stdout()).map_err(|e| {
        WebServerError::bad_request(format!(
            "{} {}: {}",
            t!("web.device.failedRemoveScreenshot"),
            payload.id,
            e
        ))
    })?;

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("image/png"));
    headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));

    Ok((StatusCode::OK, headers, image_bytes))
}

#[derive(Deserialize)]
struct PostDataSetDisplayPower {
    mode: bool,
}
async fn set_display_power(
    State(state): State<AppStateDevice>,
    Json(payload): Json<PostDataSetDisplayPower>,
) -> Result<JsonResponse, WebServerError> {
    if !ControlledDevice::is_any_device_controlled().await {
        return Err(WebServerError::bad_request(t!(
            "web.device.noDeviceControlled"
        )));
    }

    device_action::set_display_power(&state.cs_tx, payload.mode);
    Ok(JsonResponse::success(
        t!("web.device.setDisplayPowerSuccess"),
        None,
    ))
}

#[derive(Deserialize)]
struct PostDataSetPointerLocation {
    mode: bool,
}

async fn set_pointer_location(
    Json(payload): Json<PostDataSetPointerLocation>,
) -> Result<JsonResponse, WebServerError> {
    let device_list = ControlledDevice::get_device_list().await;
    if device_list.is_empty() {
        return Err(WebServerError::bad_request(t!(
            "web.device.noDeviceControlled"
        )));
    }

    let mode = if payload.mode { "1" } else { "0" };
    for device in device_list {
        let mut output = Vec::<u8>::new();
        Device::shell(
            &device.device_id,
            ["settings", "put", "system", "pointer_location", mode],
            &mut output,
        )
        .map_err(|e| {
            WebServerError::bad_request(format!(
                "{} {}: {}",
                t!("web.device.setPointerLocationFailed"),
                device.device_id,
                e
            ))
        })?;
    }

    Ok(JsonResponse::success(
        t!("web.device.setPointerLocationSuccess"),
        None,
    ))
}

#[derive(Deserialize)]
struct PostDataSendKey {
    keycode: Keycode,
}

async fn send_key(
    State(state): State<AppStateDevice>,
    Json(payload): Json<PostDataSendKey>,
) -> Result<JsonResponse, WebServerError> {
    if !ControlledDevice::is_any_device_controlled().await {
        return Err(WebServerError::bad_request(t!(
            "web.device.noDeviceControlled"
        )));
    }

    device_action::inject_keycode(&state.cs_tx, payload.keycode);
    Ok(JsonResponse::success(t!("web.device.sendKeySuccess"), None))
}
