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
    sync::{broadcast, mpsc::UnboundedSender, oneshot},
    time::sleep,
};

use crate::{
    config::LocalConfig,
    mask::mask_command::MaskCommand,
    scrcpy::{
        adb::{Adb, Device},
        constant::{KeyEventAction, Keycode, MetaState},
        control_msg::ScrcpyControlMsg,
        controller::ControllerCommand,
    },
    utils::{relate_to_root_path, share::ControlledDevice},
    web::{JsonResponse, WebServerError},
};

#[derive(Debug, Clone)]
pub struct AppStateDevice {
    cs_tx: broadcast::Sender<ScrcpyControlMsg>,
    d_tx: UnboundedSender<ControllerCommand>,
    m_tx: crossbeam_channel::Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
}

pub fn routers(
    cs_tx: broadcast::Sender<ScrcpyControlMsg>,
    d_tx: UnboundedSender<ControllerCommand>,
    m_tx: crossbeam_channel::Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
) -> Router {
    Router::new()
        .route("/device_list", get(device_list))
        .route("/control_device", post(control_device))
        .route("/decontrol_device", post(decontrol_device))
        .route("/adb_connect", post(adb_connect))
        .route("/adb_pair", post(adb_pair))
        .route("/adb_screenshot", post(adb_screenshot))
        .route("/control/set_display_power", post(set_display_power))
        .route("/control/send_key", post(send_key))
        .route("/control/eval_script", post(eval_script))
        .with_state(AppStateDevice { cs_tx, d_tx, m_tx })
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
}

async fn control_device(
    State(state): State<AppStateDevice>,
    Json(payload): Json<PostDataControlDevice>,
) -> Result<JsonResponse, WebServerError> {
    let device_id = payload.device_id;
    let video = payload.video;
    let display_id = payload.display_id;

    let local_config = LocalConfig::get();

    let device_list = ControlledDevice::get_device_list().await;
    // check if device is controlled
    if device_list
        .iter()
        .any(|device| device.device_id == device_id)
    {
        return Err(WebServerError(
            400,
            format!("{}: {}", t!("web.device.alreadyControlled"), device_id),
        ));
    }

    // prepare for scrcpy app
    let scid = gen_scid();
    let version = "2.4";
    let scrcpy_path = relate_to_root_path(["assets", &format!("scrcpy-mask-server-v{}", version)]);
    Device::push(
        &device_id,
        scrcpy_path.to_str().unwrap(),
        "/data/local/tmp/scrcpy-server.jar",
    )
    .map_err(|e| WebServerError(500, e))?;
    log::info!("[WebServe] {}", t!("web.device.pushScrcpyServerSuccess"));

    let remote = format!("localabstract:scrcpy_{}", scid);
    let local = format!("tcp:{}", local_config.controller_port);
    Device::reverse(&device_id, &remote, &local).map_err(|e| WebServerError(500, e))?;
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

    args.push(version.to_string());
    args.push(format!("scid={}", scid));
    args.push(format!("video={}", video));
    args.push(format!("display_id={}", display_id));
    args.push("audio=false".to_string());

    // create device
    let main = device_list.len() == 0;
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
                args.push(format!("video_max_size={}", local_config.video_max_size));
            }
            if local_config.video_max_fps > 0 {
                args.push(format!("video_max_fps={}", local_config.video_max_fps));
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
        state.d_tx.send(cmd).unwrap();
    }

    // run scrcpy app
    sleep(Duration::from_millis(500)).await;
    log::info!("[WebServe] {}", t!("web.device.startingScrcpyApp"));

    let h = Device::shell_process(&device_id, args);

    let scid_copy = scid.clone();
    tokio::spawn(async move {
        h.await.unwrap().unwrap();
        log::info!("[WebServe] {}", t!("web.device.removingDeviceAfterExit"));
        ControlledDevice::remove_device(&scid_copy).await;
    });

    Ok(JsonResponse::success(
        t!("web.device.tryStartingScrcpy"),
        Some(json!({"scid": scid, "device_id": device_id})),
    ))
}

#[derive(Deserialize)]
struct PostDataDeControlDevice {
    device_id: String,
}

async fn decontrol_device(
    State(state): State<AppStateDevice>,
    Json(payload): Json<PostDataDeControlDevice>,
) -> Result<JsonResponse, WebServerError> {
    let device_id = payload.device_id;
    let device_list = ControlledDevice::get_device_list().await;
    for device in device_list {
        if device.device_id == device_id {
            let scid = device.scid.clone();
            if device.main {
                state
                    .d_tx
                    .send(ControllerCommand::ShutdownMain(scid))
                    .unwrap();
            } else {
                state
                    .d_tx
                    .send(ControllerCommand::ShutdownSub(scid))
                    .unwrap();
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

#[derive(Deserialize)]
struct PostDataAddress {
    address: String,
}

async fn adb_connect(Json(payload): Json<PostDataAddress>) -> Result<JsonResponse, WebServerError> {
    let config = LocalConfig::get();
    match Adb::new(config.adb_path).connect_device(&payload.address) {
        Ok(_) => Ok(JsonResponse::success(
            format!(
                "{}",
                t!("web.device.adbConnect", address => payload.address)
            ),
            None,
        )),
        Err(e) => Err(WebServerError::bad_request(format!(
            "{}: {}",
            t!("web.device.adbConnectFailed", address => payload.address),
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

#[derive(Deserialize)]
struct PostDataId {
    id: String,
}

async fn adb_screenshot(
    Json(payload): Json<PostDataId>,
) -> Result<impl IntoResponse, WebServerError> {
    let src = "/data/local/tmp/_screenshot_scrcpy_mask.png";

    Device::shell(
        &payload.id,
        ["screencap", "-p", src],
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

    state
        .cs_tx
        .send(ScrcpyControlMsg::SetDisplayPower { mode: payload.mode })
        .unwrap();
    Ok(JsonResponse::success(
        t!("web.device.setDisplayPowerSuccess"),
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

    state
        .cs_tx
        .send(ScrcpyControlMsg::InjectKeycode {
            action: KeyEventAction::Down,
            keycode: payload.keycode.clone(),
            repeat: 0,
            metastate: MetaState::NONE,
        })
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    state
        .cs_tx
        .send(ScrcpyControlMsg::InjectKeycode {
            action: KeyEventAction::Up,
            keycode: payload.keycode,
            repeat: 0,
            metastate: MetaState::NONE,
        })
        .unwrap();
    Ok(JsonResponse::success(t!("web.device.sendKeySuccess"), None))
}

#[derive(Deserialize)]
struct PostDataEvalScript {
    script: String,
}

async fn eval_script(
    State(state): State<AppStateDevice>,
    Json(payload): Json<PostDataEvalScript>,
) -> Result<JsonResponse, WebServerError> {
    if !ControlledDevice::is_any_device_controlled().await {
        return Err(WebServerError::bad_request(t!(
            "web.device.noDeviceControlled"
        )));
    }

    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    state
        .m_tx
        .send((
            MaskCommand::EvalScript {
                script: payload.script,
            },
            oneshot_tx,
        ))
        .unwrap();
    match oneshot_rx.await.unwrap() {
        Ok(_) => Ok(JsonResponse::success(
            t!("web.device.evalScriptSuccess"),
            None,
        )),
        Err(e) => Err(WebServerError::bad_request(format!(
            "{}:\n{}",
            t!("web.device.evalScriptError"),
            e
        ))),
    }
}
