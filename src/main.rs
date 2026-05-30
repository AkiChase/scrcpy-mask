use std::{fs::File, net::SocketAddrV4, sync::OnceLock};

use bevy::{
    log::{BoxedLayer, LogPlugin, tracing_subscriber::Layer},
    prelude::*,
    window::{PresentMode, WindowLevel},
};
use bevy_tokio_tasks::TokioTasksRuntime;
use scrcpy_mask::{
    config::LocalConfig,
    mask::{MaskPlugins, mask_command::MaskCommand},
    scrcpy::{
        control_msg::ScrcpyControlMsg,
        controller::{self, ControllerCommand},
        media::VideoMsg,
    },
    utils::{
        ChannelReceiverM, ChannelReceiverV, ChannelSenderCS, check_for_update, relate_to_data_path,
    },
    web::{self, ws::WebSocketNotification},
};
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing_appender::non_blocking::WorkerGuard;

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

fn log_custom_layer(_app: &mut App) -> Option<BoxedLayer> {
    let file = File::create(relate_to_data_path(["app.log"])).unwrap_or_else(|e| {
        panic!("Failed to create log file: {}", e);
    });
    let (non_blocking, guard) = tracing_appender::non_blocking(file);
    let _ = LOG_GUARD.set(guard);
    Some(
        bevy::log::tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_file(false)
            .with_line_number(true)
            .with_ansi(false)
            .boxed(),
    )
}

fn main() {
    let default_language = "en-US";
    rust_i18n::set_locale(default_language);

    if let Err(e) = LocalConfig::load() {
        println!("LocalConfig load failed. {}", e);
    }

    let mut local_config = LocalConfig::get();
    // update language
    let language = local_config.language;
    match language.as_str() {
        "zh-CN" | "en-US" => rust_i18n::set_locale(&language),
        _ => {
            rust_i18n::set_locale(default_language);
            LocalConfig::set_language(default_language.to_string());
            local_config = LocalConfig::get();
        }
    }
    // update config file
    LocalConfig::save().unwrap();

    ffmpeg_next::init().unwrap();

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                custom_layer: log_custom_layer,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    has_shadow: false,
                    transparent: true, // for windows: https://github.com/bevyengine/bevy/issues/7544
                    decorations: false,
                    present_mode: PresentMode::AutoVsync,
                    resizable: false,
                    visible: false,
                    window_level: if local_config.always_on_top {
                        WindowLevel::AlwaysOnTop
                    } else {
                        WindowLevel::Normal
                    },
                    #[cfg(target_os = "macos")]
                    composite_alpha_mode: bevy::window::CompositeAlphaMode::PostMultiplied,
                    #[cfg(target_os = "linux")]
                    composite_alpha_mode: bevy::window::CompositeAlphaMode::PreMultiplied,
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(bevy_tokio_tasks::TokioTasksPlugin::default())
    .add_plugins(MaskPlugins)
    .add_systems(Startup, (start_servers, check_for_update_system));

    #[cfg(target_os = "macos")]
    {
        app.insert_resource(bevy::ecs::schedule::MainThreadExecutor::default())
            .add_systems(Startup, macos_menu);
    }

    app.run();
}

#[cfg(target_os = "macos")]
fn macos_menu(executor: Res<bevy::ecs::schedule::MainThreadExecutor>) {
    use muda::{Menu, Submenu};
    // remove default menu
    executor
        .0
        .spawn(async move {
            let menu = Menu::new();
            let submenu = Submenu::new("scrcpy-mask", true);
            menu.append(&submenu).unwrap();
            menu.init_for_nsapp();
        })
        .detach();
}

fn start_servers(mut commands: Commands) {
    let config = LocalConfig::get();
    let web_addr: SocketAddrV4 = format!("127.0.0.1:{}", config.web_port).parse().unwrap();
    let controller_addr: SocketAddrV4 = format!("127.0.0.1:{}", config.controller_port)
        .parse()
        .unwrap();

    let (cs_tx, _) = broadcast::channel::<ScrcpyControlMsg>(1000);
    let (ws_tx, _) = broadcast::channel::<WebSocketNotification>(1000);
    let (v_tx, v_rx) = crossbeam_channel::unbounded::<VideoMsg>();
    let (m_tx, m_rx) =
        crossbeam_channel::unbounded::<(MaskCommand, oneshot::Sender<Result<String, String>>)>();
    let (d_tx, d_rx) = mpsc::unbounded_channel::<ControllerCommand>();

    commands.insert_resource(ChannelSenderCS(cs_tx.clone()));
    commands.insert_resource(ChannelReceiverV(v_rx));
    commands.insert_resource(ChannelReceiverM(m_rx));
    web::Server::start(web_addr, cs_tx.clone(), d_tx, m_tx.clone(), ws_tx.clone());
    controller::Controller::start(controller_addr, cs_tx, v_tx, d_rx, m_tx, ws_tx);
}

fn check_for_update_system(runtime: ResMut<TokioTasksRuntime>) {
    runtime.spawn_background_task(move |_ctx| async move {
        if let Err(e) = check_for_update().await {
            log::error!("{}", e);
        }
    });
}
