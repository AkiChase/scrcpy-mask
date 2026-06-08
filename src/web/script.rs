use axum::{Json, Router, extract::State, routing::post};
use rust_i18n::t;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::oneshot;

use crate::{
    mask::{mapping::script_helper::ScriptAST, mask_command::MaskCommand},
    utils::share::ControlledDevice,
    web::{JsonResponse, WebServerError},
};

#[derive(Debug, Clone)]
pub struct AppStateScript {
    m_tx: crossbeam_channel::Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
}

pub fn routers(
    m_tx: crossbeam_channel::Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
) -> Router {
    Router::new()
        .route("/validate", post(validate_script))
        .route("/run", post(run_script))
        .with_state(AppStateScript { m_tx })
}

#[derive(Deserialize)]
struct PostDataScript {
    script: String,
}

async fn validate_script(
    Json(payload): Json<PostDataScript>,
) -> Result<JsonResponse, WebServerError> {
    let diagnostics = ScriptAST::validate_diagnostics(&payload.script);
    let data = json!({
        "valid": diagnostics.is_empty(),
        "diagnostics": diagnostics,
    });

    Ok(JsonResponse::success(
        t!("web.script.validateScriptSuccess"),
        Some(data),
    ))
}

async fn run_script(
    State(state): State<AppStateScript>,
    Json(payload): Json<PostDataScript>,
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
            MaskCommand::RunScript {
                script: payload.script,
            },
            oneshot_tx,
        ))
        .unwrap();

    match oneshot_rx.await.unwrap() {
        Ok(_) => Ok(JsonResponse::success(
            t!("web.script.runScriptSuccess"),
            None,
        )),
        Err(e) => Err(WebServerError::bad_request(format!(
            "{}:\n{}",
            t!("web.script.runScriptError"),
            e
        ))),
    }
}
