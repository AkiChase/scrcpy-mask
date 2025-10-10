use std::fs;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use bevy::math::Vec2;
use rust_i18n::t;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::oneshot;

use crate::{
    config::LocalConfig,
    mask::{
        mapping::config::{MappingConfig, MappingType, save_mapping_config},
        mask_command::MaskCommand,
    },
    utils::{is_safe_file_name, relate_to_data_path},
    web::{JsonResponse, WebServerError},
};

#[derive(Debug, Clone)]
pub struct AppStatMapping {
    m_tx: crossbeam_channel::Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
}

pub fn routers(
    m_tx: crossbeam_channel::Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
) -> Router {
    Router::new()
        .route("/change_active_mapping", post(change_active_mapping))
        .route("/create_mapping", post(create_mapping))
        .route("/rename_mapping", post(rename_mapping))
        .route("/duplicate_mapping", post(duplicate_mapping))
        .route("/delete_mapping", post(delete_mapping))
        .route("/update_mapping", post(update_mapping))
        .route("/read_mapping", post(read_mapping))
        .route("/get_mapping_list", get(get_mapping_list))
        .route("/migrate_mapping", post(migrate_mapping))
        .with_state(AppStatMapping { m_tx })
}

#[derive(Deserialize)]
struct PostDataChangeActiveMapping {
    file: String,
}

async fn change_active_mapping(
    State(state): State<AppStatMapping>,
    Json(mut payload): Json<PostDataChangeActiveMapping>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    state
        .m_tx
        .send((
            MaskCommand::LoadAndActivateMappingConfig {
                file_name: payload.file.clone(),
            },
            oneshot_tx,
        ))
        .unwrap();
    match oneshot_rx.await.unwrap() {
        Ok(_) => {
            LocalConfig::set_active_mapping_file(payload.file.clone());
            log::info!(
                "[WebServer] {}: {}",
                t!("web.mapping.setActiveMapping"),
                payload.file
            );

            Ok(JsonResponse::success(
                format!(
                    "{}: {}",
                    t!("web.mapping.setActiveMappingSuccess"),
                    payload.file
                ),
                None,
            ))
        }
        Err(e) => Err(WebServerError::bad_request(format!(
            "{}: {}. {}",
            t!("web.mapping.failedToLoadMappingConfig"),
            payload.file,
            e
        ))),
    }
}

#[derive(Deserialize)]
struct PostDataNewMapping {
    file: String,
    config: MappingConfig,
}

async fn validate_config(
    m_tx: &crossbeam_channel::Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
    config: &MappingConfig,
) -> Result<(), String> {
    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    m_tx.send((
        MaskCommand::ValidateMappingConfig {
            config: config.clone(),
        },
        oneshot_tx,
    ))
    .unwrap();
    match oneshot_rx.await.unwrap() {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

async fn create_mapping(
    State(state): State<AppStatMapping>,
    Json(mut payload): Json<PostDataNewMapping>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    let bad_request =
        |msg| -> Result<JsonResponse, WebServerError> { Err(WebServerError::bad_request(msg)) };

    if !is_safe_file_name(payload.file.as_ref()) {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.nameNotSafe"),
            payload.file
        ));
    }

    let config_path = relate_to_data_path(["mapping", &payload.file]);
    if config_path.exists() {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.mappingConfigExists"),
            payload.file
        ));
    }

    validate_config(&state.m_tx, &payload.config)
        .await
        .map_err(|e| WebServerError::bad_request(e))?;

    // save to file
    save_mapping_config(&payload.config, &config_path)
        .map_err(|e| WebServerError::bad_request(e))?;

    log::info!(
        "[WebServer] {}: {}",
        t!("web.mapping.createMappingConfig"),
        payload.file
    );
    Ok(JsonResponse::success(
        format!(
            "{}: {}",
            t!("web.mapping.createMappingConfig"),
            payload.file
        ),
        None,
    ))
}

#[derive(Deserialize)]
struct PostDataMappingFile {
    file: String,
}

async fn delete_mapping(
    State(state): State<AppStatMapping>,
    Json(mut payload): Json<PostDataMappingFile>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    let bad_request =
        |msg| -> Result<JsonResponse, WebServerError> { Err(WebServerError::bad_request(msg)) };

    if !is_safe_file_name(payload.file.as_ref()) {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.nameNotSafe"),
            payload.file
        ));
    }

    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    state
        .m_tx
        .send((MaskCommand::GetActiveMapping, oneshot_tx))
        .unwrap();
    let file = oneshot_rx.await.unwrap().unwrap();
    if file == payload.file {
        return bad_request(t!("web.mapping.cannotDeleteActiveMapping").to_string());
    }
    let file_path = relate_to_data_path(["mapping", &payload.file]);
    if !file_path.exists() {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.mappingConfigNotExists"),
            payload.file
        ));
    }
    fs::remove_file(file_path).map_err(|e| {
        WebServerError::bad_request(format!(
            "{} {}: {}",
            t!("web.mapping.deleteMappingConfigError"),
            payload.file,
            e
        ))
    })?;

    log::info!(
        "[WebServer] {}: {}",
        t!("web.mapping.deleteMappingConfig"),
        payload.file
    );
    Ok(JsonResponse::success(
        format!(
            "{}: {}",
            t!("web.mapping.deleteMappingConfig"),
            payload.file
        ),
        None,
    ))
}

#[derive(Deserialize)]
struct PostDataRenameMappingFile {
    file: String,
    new_file: String,
}

async fn rename_mapping(
    State(state): State<AppStatMapping>,
    Json(mut payload): Json<PostDataRenameMappingFile>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    if !payload.new_file.ends_with(".json") {
        payload.new_file.push_str(".json");
    }

    let bad_request =
        |msg| -> Result<JsonResponse, WebServerError> { Err(WebServerError::bad_request(msg)) };

    if !is_safe_file_name(payload.file.as_ref()) {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.nameNotSafe"),
            payload.file
        ));
    }
    if !is_safe_file_name(payload.new_file.as_ref()) {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.nameNotSafe"),
            payload.new_file
        ));
    }

    // rename file
    let old_path = relate_to_data_path(["mapping", &payload.file]);
    if !old_path.exists() {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.mappingConfigNotFound"),
            old_path.to_str().unwrap()
        ));
    }
    let new_path = relate_to_data_path(["mapping", &payload.new_file]);
    if new_path.exists() {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.mappingConfigExists"),
            new_path.to_str().unwrap()
        ));
    }
    fs::rename(old_path, new_path).map_err(|e| WebServerError::internal_error(e.to_string()))?;

    // get active mapping file
    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    state
        .m_tx
        .send((MaskCommand::GetActiveMapping, oneshot_tx))
        .unwrap();
    let file = oneshot_rx.await.unwrap().unwrap();
    if file == payload.file {
        // if active, set new active mapping
        let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
        state
            .m_tx
            .send((
                MaskCommand::LoadAndActivateMappingConfig {
                    file_name: payload.new_file.clone(),
                },
                oneshot_tx,
            ))
            .unwrap();
        match oneshot_rx.await.unwrap() {
            Ok(_) => {
                LocalConfig::set_active_mapping_file(payload.new_file.clone());
                let msg = t!(
                    "web.mapping.renameActivateMappingSuccess",
                    oldFile => payload.file,
                    newFile => payload.new_file
                );
                log::info!("[WebServer] {}", msg);
                return Ok(JsonResponse::success(msg, None));
            }
            Err(e) => {
                return Err(WebServerError::bad_request(format!(
                    "{}: {}. {}",
                    t!("web.mapping.failedToLoadMappingConfig"),
                    payload.file,
                    e
                )));
            }
        }
    }

    let msg = t!(
        "web.mapping.renameMappingConfigSuccess",
        file => payload.file,
        newFile => payload.new_file
    );
    log::info!("[WebServer] {}", msg);
    Ok(JsonResponse::success(msg, None))
}

#[derive(Deserialize)]
struct PostDataDuplicateMappingFile {
    file: String,
    new_file: String,
}

async fn duplicate_mapping(
    Json(mut payload): Json<PostDataDuplicateMappingFile>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    if !payload.new_file.ends_with(".json") {
        payload.new_file.push_str(".json");
    }

    let bad_request =
        |msg| -> Result<JsonResponse, WebServerError> { Err(WebServerError::bad_request(msg)) };

    if !is_safe_file_name(payload.file.as_ref()) {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.nameNotSafe"),
            payload.file
        ));
    }
    if !is_safe_file_name(payload.new_file.as_ref()) {
        return bad_request(format!(
            "New {}: {}",
            t!("web.mapping.nameNotSafe"),
            payload.new_file
        ));
    }

    let old_path = relate_to_data_path(["mapping", &payload.file]);
    if !old_path.exists() {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.mappingConfigExists"),
            old_path.to_str().unwrap()
        ));
    }
    let new_path = relate_to_data_path(["mapping", &payload.new_file]);
    if new_path.exists() {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.mappingConfigExists"),
            new_path.to_str().unwrap()
        ));
    }
    fs::copy(old_path, new_path).map_err(|e| WebServerError::internal_error(e.to_string()))?;
    log::info!(
        "[WebServer] {}",
        t!(
            "web.mapping.copyMappingConfig",
            file => payload.file,
            newFile => payload.new_file
        )
    );
    Ok(JsonResponse::success(
        t!(
            "web.mapping.copyMappingConfig",
            file => payload.file,
            newFile => payload.new_file
        ),
        None,
    ))
}

async fn update_mapping(
    State(state): State<AppStatMapping>,
    Json(mut payload): Json<PostDataNewMapping>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    let bad_request =
        |msg| -> Result<JsonResponse, WebServerError> { Err(WebServerError::bad_request(msg)) };

    if !is_safe_file_name(payload.file.as_ref()) {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.nameNotSafe"),
            payload.file
        ));
    }

    validate_config(&state.m_tx, &payload.config)
        .await
        .map_err(|e| WebServerError::bad_request(e))?;

    // save to file
    let config_path = relate_to_data_path(["mapping", &payload.file]);
    save_mapping_config(&payload.config, &config_path)
        .map_err(|e| WebServerError::bad_request(e))?;

    // get active mapping file
    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    state
        .m_tx
        .send((MaskCommand::GetActiveMapping, oneshot_tx))
        .unwrap();
    let file = oneshot_rx.await.unwrap().unwrap();
    if file == payload.file {
        // if active, refresh active mapping
        let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
        state
            .m_tx
            .send((
                MaskCommand::LoadAndActivateMappingConfig {
                    file_name: payload.file.clone(),
                },
                oneshot_tx,
            ))
            .unwrap();
        match oneshot_rx.await.unwrap() {
            Ok(_) => {
                LocalConfig::set_active_mapping_file(payload.file.clone());
                let msg = format!(
                    "{}: {}",
                    t!("web.mapping.updateAndActivateMappingConfig"),
                    payload.file
                );

                log::info!("[WebServer] {}", msg);
                Ok(JsonResponse::success(msg, None))
            }
            Err(e) => Err(WebServerError::bad_request(format!(
                "{} {}. {}",
                t!("web.mapping.failedToLoadUpdatedMappingConfig"),
                payload.file,
                e
            ))),
        }
    } else {
        let msg = format!("{} {}", t!("web.mapping.updateMappingConfig"), payload.file);
        log::info!("[WebServer] {}", msg);
        Ok(JsonResponse::success(msg, None))
    }
}

async fn get_mapping_list(
    State(state): State<AppStatMapping>,
) -> Result<JsonResponse, WebServerError> {
    let dir_path = relate_to_data_path(["mapping"]);
    let entries = fs::read_dir(dir_path).map_err(|e| {
        WebServerError::bad_request(format!(
            "{}: {}",
            t!("web.mapping.unableReadMappingConfigDir"),
            e
        ))
    })?;

    let mut mapping_files: Vec<String> = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        if path.is_file() {
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(file_name) = path.file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        mapping_files.push(name_str.to_string());
                    }
                }
            }
        }
    }

    // get active mapping file
    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    state
        .m_tx
        .send((MaskCommand::GetActiveMapping, oneshot_tx))
        .unwrap();
    let file = oneshot_rx.await.unwrap().unwrap();

    Ok(JsonResponse::success(
        t!("web.mapping.readMappingListSuccess"),
        Some(json!({
            "mapping_list": mapping_files,
            "active_mapping": file,
        })),
    ))
}

async fn read_mapping(
    State(state): State<AppStatMapping>,
    Json(mut payload): Json<PostDataMappingFile>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    let bad_request =
        |msg| -> Result<JsonResponse, WebServerError> { Err(WebServerError::bad_request(msg)) };

    if !is_safe_file_name(payload.file.as_ref()) {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.nameNotSafe"),
            payload.file
        ));
    }

    // load from file
    let path = relate_to_data_path(["mapping", &payload.file]);
    if !path.exists() {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.mappingConfigExists"),
            payload.file
        ));
    }
    let config_string = std::fs::read_to_string(path).map_err(|e| {
        WebServerError::bad_request(format!(
            "{} {}: {}",
            t!("web.mapping.cannotReadMappingConfig"),
            payload.file,
            e
        ))
    })?;
    let mapping_config: MappingConfig = serde_json::from_str(&config_string).map_err(|e| {
        WebServerError::bad_request(format!(
            "{} {}: {}",
            t!("web.mapping.cannotDeserializeConfig"),
            payload.file,
            e
        ))
    })?;

    validate_config(&state.m_tx, &mapping_config)
        .await
        .map_err(|e| {
            WebServerError::bad_request(format!(
                "{} {}: {}",
                t!("web.mapping.invalidMappingConfig"),
                payload.file,
                e
            ))
        })?;

    Ok(JsonResponse::success(
        format!("{} {}", t!("web.mapping.mappingReadSuccess"), payload.file),
        Some(json!({
            "mapping_config": mapping_config,
        })),
    ))
}

#[derive(Deserialize)]
struct PostDataMigrateMappingFile {
    file: String,
    new_file: String,
    width: u32,
    height: u32,
}

async fn migrate_mapping(
    Json(mut payload): Json<PostDataMigrateMappingFile>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    if !payload.new_file.ends_with(".json") {
        payload.new_file.push_str(".json");
    }

    let bad_request =
        |msg| -> Result<JsonResponse, WebServerError> { Err(WebServerError::bad_request(msg)) };

    if !is_safe_file_name(payload.file.as_ref()) {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.nameNotSafe"),
            payload.file
        ));
    }

    let old_path = relate_to_data_path(["mapping", &payload.file]);
    if !old_path.exists() {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.mappingConfigNotExists"),
            payload.file
        ));
    }

    let new_path = relate_to_data_path(["mapping", &payload.new_file]);
    if new_path.exists() {
        return bad_request(format!(
            "{}: {}",
            t!("web.mapping.mappingConfigExists"),
            payload.new_file
        ));
    }

    let config_string = std::fs::read_to_string(old_path).map_err(|e| {
        WebServerError::bad_request(format!(
            "{} {}: {}",
            t!("web.mapping.cannotReadMappingConfig"),
            payload.file,
            e
        ))
    })?;
    let mut mapping_config: MappingConfig = serde_json::from_str(&config_string).map_err(|e| {
        WebServerError::bad_request(format!(
            "{} {}: {}",
            t!("web.mapping.cannotDeserializeConfig"),
            payload.file,
            e
        ))
    })?;

    if payload.width == 0 || payload.height == 0 {
        return bad_request(format!(
            "{}: {}, {}",
            t!("web.mapping.invalidSize"),
            payload.width,
            payload.height
        ));
    }

    let scale = Vec2::new(
        payload.width as f32 / mapping_config.original_size.width as f32,
        payload.height as f32 / mapping_config.original_size.height as f32,
    );

    mapping_config.original_size.width = payload.width;
    mapping_config.original_size.height = payload.height;

    mapping_config
        .mappings
        .iter_mut()
        .for_each(|mapping| match mapping {
            MappingType::SingleTap(m) => {
                m.position *= scale;
            }
            MappingType::RepeatTap(m) => {
                m.position *= scale;
            }
            MappingType::MultipleTap(m) => {
                m.items.iter_mut().for_each(|item| {
                    item.position *= scale;
                });
            }
            MappingType::Swipe(m) => {
                m.positions.iter_mut().for_each(|p| {
                    *p *= scale;
                });
            }
            MappingType::DirectionPad(m) => {
                m.position *= scale;
                m.max_offset_x *= scale.x;
                m.max_offset_y *= scale.y;
            }
            MappingType::MouseCastSpell(m) => {
                m.position *= scale;
                m.cast_radius *= scale.y;
                m.center *= scale;
                m.drag_radius *= scale.y;
            }
            MappingType::PadCastSpell(m) => {
                m.drag_radius *= scale.y;
                m.position *= scale;
            }
            MappingType::CancelCast(m) => {
                m.position *= scale;
            }
            MappingType::Observation(m) => {
                m.position *= scale;
            }
            MappingType::Fps(m) => {
                m.position *= scale;
            }
            MappingType::Fire(m) => {
                m.position *= scale;
            }
            MappingType::RawInput(m) => {
                m.position *= scale;
            }
            MappingType::Script(m) => {
                m.position *= scale;
            }
        });

    // save to file
    save_mapping_config(&mapping_config, &new_path).map_err(|e| WebServerError::bad_request(e))?;

    let msg = t!(
        "web.mapping.migrateMappingConfig",
        file => payload.file,
        newFile => payload.new_file
    )
    .to_string();

    log::info!("[WebServer] {}", msg);
    Ok(JsonResponse::success(msg, None))
}
