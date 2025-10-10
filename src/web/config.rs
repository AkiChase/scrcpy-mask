use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use rust_i18n::t;
use serde::Deserialize;
use tokio::sync::oneshot;

use crate::{
    config::LocalConfig,
    mask::mask_command::MaskCommand,
    scrcpy::{adb::Adb, media::VideoCodec},
    utils::{
        IDENTIFIER, check_for_update, mask_win_move_helper,
        share::{ControlledDevice, UpdateInfo},
    },
    web::{JsonResponse, WebServerError},
};

#[derive(Debug, Clone)]
pub struct AppStatConfig {
    m_tx: crossbeam_channel::Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
}

pub fn routers(
    m_tx: crossbeam_channel::Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
) -> Router {
    Router::new()
        .route("/get_config", get(get_config))
        .route("/update_config", post(update_config))
        .route("/open_data_path", get(open_data_path))
        .route("/get_update_info", get(get_update_info))
        .route("/check_update", get(check_update))
        .with_state(AppStatConfig { m_tx })
}

async fn get_config() -> Result<JsonResponse, WebServerError> {
    let config = LocalConfig::get();
    Ok(JsonResponse::success(
        t!("web.config.getLocalConfigSuccess"),
        Some(serde_json::to_value(&config).unwrap()),
    ))
}

async fn open_data_path() -> Result<JsonResponse, WebServerError> {
    let path = dirs::data_dir().unwrap().join(IDENTIFIER);
    opener::open(path).map_err(|e| {
        WebServerError::bad_request(format!("{}: {}", t!("web.config.openDataPathFailed"), e))
    })?;

    return Ok(JsonResponse::success(
        t!("web.config.openDataPathSuccess"),
        None,
    ));
}

async fn get_update_info() -> Result<JsonResponse, WebServerError> {
    let info = UpdateInfo::get().await;
    return Ok(JsonResponse::success(
        t!("web.config.getUpdateInfoSuccess"),
        Some(serde_json::to_value(&info).unwrap()),
    ));
}

async fn check_update() -> Result<JsonResponse, WebServerError> {
    match check_for_update().await {
        Err(e) => {
            return Err(WebServerError::bad_request(e.to_string()));
        }
        Ok(()) => {
            let info = UpdateInfo::get().await;
            return Ok(JsonResponse::success(
                t!("web.config.getUpdateInfoSuccess"),
                Some(serde_json::to_value(&info).unwrap()),
            ));
        }
    };
}

#[derive(Deserialize)]
struct PostDataUpdateConfig {
    key: String,
    value: serde_json::Value,
}

async fn update_config(
    State(state): State<AppStatConfig>,
    Json(payload): Json<PostDataUpdateConfig>,
) -> Result<JsonResponse, WebServerError> {
    // sync with src/config.rs
    match payload.key.as_str() {
        "language" => {
            if let Some(value) = payload.value.as_str() {
                if !matches!(value, "zh-CN" | "en-US") {
                    return Err(WebServerError::bad_request(format!(
                        "{}: {}",
                        t!("web.config.invalidLanguage"),
                        value
                    )));
                }
                rust_i18n::set_locale(value);
                LocalConfig::set_language(value.to_string());
                return Ok(JsonResponse::success(
                    format!("{}: {}", t!("web.config.setLanguageSuccess"), value),
                    None,
                ));
            } else {
                return Err(WebServerError::bad_request(t!(
                    "web.config.languageMustBeString"
                )));
            }
        }
        "web_port" => {
            if let Some(value) = payload.value.as_u64() {
                LocalConfig::set_web_port(value as u16);
                return Ok(JsonResponse::success(
                    format!("{}: {}", t!("web.config.restartToApplyWebPort"), value),
                    None,
                ));
            } else {
                return Err(WebServerError::bad_request(t!(
                    "web.config.webPortMustBeU16"
                )));
            }
        }
        "adb_path" => {
            if let Some(value) = payload.value.as_str() {
                match Adb::check_adb_path(value) {
                    Ok(_) => {
                        LocalConfig::set_adb_path(value.to_string());
                        return Ok(JsonResponse::success(
                            format!("{}: {}", t!("web.config.adbPathSetSuccess"), value),
                            None,
                        ));
                    }
                    Err(e) => {
                        return Err(WebServerError::bad_request(format!(
                            "{}: {}",
                            t!("web.config.adbPathSetFailed"),
                            e
                        )));
                    }
                }
            } else {
                return Err(WebServerError::bad_request(t!(
                    "web.config.adbPathMustBeString"
                )));
            }
        }
        "controller_port" => {
            if let Some(value) = payload.value.as_u64() {
                LocalConfig::set_controller_port(value as u16);
                return Ok(JsonResponse::success(
                    format!(
                        "{}: {}",
                        t!("web.config.restartToApplyControllerPort"),
                        value
                    ),
                    None,
                ));
            } else {
                return Err(WebServerError::bad_request(t!(
                    "web.config.controllerPortMustBeU16"
                )));
            }
        }
        "always_on_top" => {
            if let Some(value) = payload.value.as_bool() {
                LocalConfig::set_always_on_top(value);
                let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
                state
                    .m_tx
                    .send((MaskCommand::WinSwitchLevel { top: value }, oneshot_tx))
                    .unwrap();
                let msg = oneshot_rx.await.unwrap().unwrap();
                return Ok(JsonResponse::success(msg, None));
            } else {
                return Err(WebServerError::bad_request(t!(
                    "web.config.alwaysOnTopMustBeBool"
                )));
            }
        }
        "vertical_mask_height" => {
            if let Some(value) = payload.value.as_u64() {
                LocalConfig::set_vertical_mask_height(value as u32);
                if let Some(main_device) = ControlledDevice::get_main_device().await {
                    let (device_w, device_h) = main_device.device_size;
                    let msg = mask_win_move_helper(device_w, device_h, &state.m_tx).await;
                    return Ok(JsonResponse::success(
                        format!("{}. {}", t!("web.config.setVerticalMaskHeightSuccess"), msg),
                        None,
                    ));
                }
                return Ok(JsonResponse::success(
                    format!("{}", t!("web.config.setVerticalMaskHeightSuccess")),
                    None,
                ));
            }
            return Err(WebServerError::bad_request(t!(
                "web.config.verticalMaskHeightMustBeu32"
            )));
        }
        "horizontal_mask_width" => {
            if let Some(value) = payload.value.as_u64() {
                LocalConfig::set_horizontal_mask_width(value as u32);
                if let Some(main_device) = ControlledDevice::get_main_device().await {
                    let (device_w, device_h) = main_device.device_size;
                    let msg = mask_win_move_helper(device_w, device_h, &state.m_tx).await;
                    return Ok(JsonResponse::success(
                        format!(
                            "{}. {}",
                            t!("web.config.setHorizontalMaskWidthSuccess"),
                            msg
                        ),
                        None,
                    ));
                }
                return Ok(JsonResponse::success(
                    t!("web.config.setHorizontalMaskWidthSuccess"),
                    None,
                ));
            }
            return Err(WebServerError::bad_request(t!(
                "web.config.horizontalMaskWidthMustBeu32"
            )));
        }
        "vertical_position" => {
            if let Some(value) = payload.value.as_array() {
                if value.len() == 2 {
                    if let (Some(x), Some(y)) = (value[0].as_i64(), value[1].as_i64()) {
                        LocalConfig::set_vertical_position((x as i32, y as i32));
                        if let Some(main_device) = ControlledDevice::get_main_device().await {
                            let (device_w, device_h) = main_device.device_size;
                            let msg = mask_win_move_helper(device_w, device_h, &state.m_tx).await;
                            return Ok(JsonResponse::success(
                                format!("{}. {}", t!("web.config.setVerticalPositionSuccess"), msg),
                                None,
                            ));
                        }
                        return Ok(JsonResponse::success(
                            format!("{}", t!("web.config.setVerticalPositionSuccess")),
                            None,
                        ));
                    }
                }
            }
            return Err(WebServerError::bad_request(t!(
                "web.config.verticalPositionTypeError"
            )));
        }
        "horizontal_position" => {
            if let Some(value) = payload.value.as_array() {
                if value.len() == 2 {
                    if let (Some(x), Some(y)) = (value[0].as_i64(), value[1].as_i64()) {
                        LocalConfig::set_horizontal_position((x as i32, y as i32));
                        if let Some(main_device) = ControlledDevice::get_main_device().await {
                            let (device_w, device_h) = main_device.device_size;
                            let msg = mask_win_move_helper(device_w, device_h, &state.m_tx).await;
                            return Ok(JsonResponse::success(
                                format!(
                                    "{}. {}",
                                    t!("web.config.setHorizontalPositionSuccess"),
                                    msg
                                ),
                                None,
                            ));
                        }
                    }
                }
            }
            return Err(WebServerError::bad_request(t!(
                "web.config.horizontalPositionTypeError"
            )));
        }
        "active_mapping_file" => {
            return Err(WebServerError::bad_request(format!(
                "{}",
                t!("web.config.pleaseRequestForOperation", api => "/api/mapping/change_active_mapping")
            )));
        }
        "mapping_label_opacity" => {
            if let Some(value) = payload.value.as_f64() {
                if value <= 1.0 && value >= 0.0 {
                    LocalConfig::set_mapping_label_opacity(value as f32);
                    return Ok(JsonResponse::success(
                        format!(
                            "{}: {}",
                            t!("web.config.setMappingLabelOpacitySuccess"),
                            value
                        ),
                        None,
                    ));
                }
            }
            return Err(WebServerError::bad_request(t!(
                "web.config.mappingLabelOpacityRange"
            )));
        }
        "clipboard_sync" => {
            if let Some(value) = payload.value.as_bool() {
                LocalConfig::set_clipboard_sync(value);
                return Ok(JsonResponse::success(
                    format!("{}: {}", t!("web.config.setClipboardSyncSuccess"), value),
                    None,
                ));
            }
            return Err(WebServerError::bad_request(t!(
                "web.config.clipboardSyncTypeError"
            )));
        }
        "video_codec" => {
            if let Some(value) = payload.value.as_str() {
                let codec = match value {
                    "H264" => VideoCodec::H264,
                    "H265" => VideoCodec::H265,
                    "AV1" => VideoCodec::AV1,
                    _ => {
                        return Err(WebServerError::bad_request(format!(
                            "{}: {}",
                            t!("web.config.invalidVideoCodec"),
                            value
                        )));
                    }
                };
                LocalConfig::set_video_codec(codec);
                return Ok(JsonResponse::success(
                    format!("{}: {}", t!("web.config.setVideoCodecSuccess"), value),
                    None,
                ));
            }
            return Err(WebServerError::bad_request(t!(
                "web.config.videoCodecTypeError"
            )));
        }
        "video_bit_rate" => {
            if let Some(value) = payload.value.as_u64() {
                LocalConfig::set_video_bit_rate(value as u32);
                return Ok(JsonResponse::success(
                    format!("{}: {}", t!("web.config.setVideoBitRateSuccess"), value),
                    None,
                ));
            }
            return Err(WebServerError::bad_request(t!(
                "web.config.videoBitRateTypeError"
            )));
        }
        "video_max_size" => {
            if let Some(value) = payload.value.as_u64() {
                LocalConfig::set_video_max_size(value as u32);
                return Ok(JsonResponse::success(
                    format!("{}: {}", t!("web.config.setVideoMaxSizeSuccess"), value),
                    None,
                ));
            }
            return Err(WebServerError::bad_request(t!(
                "web.config.videoMaxSizeTypeError"
            )));
        }
        "video_max_fps" => {
            if let Some(value) = payload.value.as_u64() {
                LocalConfig::set_video_max_fps(value as u32);
                return Ok(JsonResponse::success(
                    format!("{}: {}", t!("web.config.setVideoMaxFpsSuccess"), value),
                    None,
                ));
            }
            return Err(WebServerError::bad_request(t!(
                "web.config.videoMaxFpsTypeError"
            )));
        }
        _ => Err(WebServerError::bad_request(format!(
            "{}: {}",
            t!("web.config.invalidMappingKey"),
            payload.key
        ))),
    }
}
