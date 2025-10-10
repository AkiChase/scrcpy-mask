pub mod share;

use std::{
    env,
    path::{Path, PathBuf},
};

use axum::http::{HeaderMap, HeaderValue};
use bevy::ecs::resource::Resource;
use reqwest::header::USER_AGENT;
use rust_i18n::t;
use semver::Version;
use serde::Deserialize;
use tokio::sync::{broadcast, oneshot};

use crate::{
    config::LocalConfig,
    mask::mask_command::MaskCommand,
    scrcpy::{control_msg::ScrcpyControlMsg, media::VideoMsg},
    utils::share::UpdateInfo,
};

pub const IDENTIFIER: &str = "com.akichase.scrcpy-mask";

pub fn relate_to_data_path<P>(segments: P) -> PathBuf
where
    P: IntoIterator,
    P::Item: AsRef<Path>,
{
    segments
        .into_iter()
        .fold(dirs::data_dir().unwrap().join(IDENTIFIER), |acc, seg| {
            acc.join(seg)
        })
}

pub fn relate_to_root_path<P>(segments: P) -> PathBuf
where
    P: IntoIterator,
    P::Item: AsRef<Path>,
{
    let root = get_base_root();
    segments.into_iter().fold(root, |acc, seg| acc.join(seg))
}

const ILLEGAL_CHARS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

pub fn is_safe_file_name(name: &str) -> bool {
    !name.contains("..")
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
        && !name.contains("..")
        && !name.chars().any(|c| ILLEGAL_CHARS.contains(&c))
        && Path::new(name).file_name().is_some()
}

fn get_base_root() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }
    #[cfg(not(debug_assertions))]
    {
        env::current_exe()
            .expect(rust_i18n::t!("utils.cannotGetCurrentExePath").as_ref())
            .parent()
            .expect(rust_i18n::t!("utils.noParentDirectory").as_ref())
            .to_path_buf()
    }
}

#[derive(Resource)]
pub struct ChannelSenderCS(pub broadcast::Sender<ScrcpyControlMsg>);

#[derive(Resource)]
pub struct ChannelReceiverV(pub crossbeam_channel::Receiver<VideoMsg>);

#[derive(Resource)]
pub struct ChannelReceiverM(
    pub crossbeam_channel::Receiver<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
);

pub async fn mask_win_move_helper(
    device_w: u32,
    device_h: u32,
    m_tx: &crossbeam_channel::Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
) -> String {
    let config = LocalConfig::get();
    let (left, top, right, bottom) = {
        if device_w >= device_h {
            // horizontal
            let left = config.horizontal_position.0;
            let top = config.horizontal_position.1;
            let mask_w = config.horizontal_mask_width;
            let mask_h = ((device_h as f32) * (mask_w as f32) / (device_w as f32)).round() as u32;
            (left, top, left + mask_w as i32, top + mask_h as i32)
        } else {
            // vertical
            let left = config.vertical_position.0;
            let top = config.vertical_position.1;
            let mask_h = config.vertical_mask_height;
            let mask_w = ((device_w as f32) * (mask_h as f32) / (device_h as f32)).round() as u32;
            (left, top, left + mask_w as i32, top + mask_h as i32)
        }
    };
    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    m_tx.send((
        MaskCommand::WinMove {
            left,
            top,
            right,
            bottom,
        },
        oneshot_tx,
    ))
    .unwrap();
    oneshot_rx.await.unwrap().unwrap()
}

const UPDATE_URL: &str = "https://api.github.com/repos/AkiChase/scrcpy-mask/releases/latest";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize)]
struct ReleaseInfo {
    tag_name: String,
    body: String,
    name: String,
    updated_at: String,
}

pub async fn check_for_update() -> Result<(), String> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("scrcpy-mask-update-checker"),
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(UPDATE_URL)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("{}: {}", t!("utils.checkForUpdateFailed"), e))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API request failed: {}", resp.status()).into());
    }

    let release: ReleaseInfo = resp
        .json()
        .await
        .map_err(|e| format!("{}: {}", t!("utils.checkForUpdateFailed"), e))?;

    let latest_version = release.tag_name.trim_start_matches('v');
    let current = Version::parse(CURRENT_VERSION)
        .map_err(|e| format!("{}: {}", t!("utils.checkForUpdateFailed"), e))?;

    let latest = Version::parse(latest_version)
        .map_err(|e| format!("{}: {}", t!("utils.checkForUpdateFailed"), e))?;

    let info = UpdateInfo {
        has_update: latest > current,
        latest_version: latest_version.to_string(),
        current_version: CURRENT_VERSION.to_string(),
        title: release.name,
        body: release.body,
        time: release.updated_at,
    };

    if info.has_update {
        log::info!(
            "{}: {} <= {}",
            t!("utils.updateAvailable"),
            info.current_version,
            info.latest_version
        );
    } else {
        log::info!(
            "{}: {} >= {}",
            t!("utils.noUpdateAvailable"),
            info.current_version,
            info.latest_version
        );
    }

    UpdateInfo::set(info).await;

    Ok(())
}
