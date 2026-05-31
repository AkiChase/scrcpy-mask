use bevy::{prelude::*, window::WindowLevel};
use bevy_ineffable::prelude::IneffableCommands;
use rust_i18n::t;

use crate::{
    config::LocalConfig,
    mask::{
        mapping::{
            MappingState,
            config::{
                ActiveMappingConfig, MappingConfig, load_mapping_config, validate_mapping_config,
            },
            cursor::{CursorPosition, CursorState},
            script_helper::ScriptAST,
        },
        ui::basic::TITLEBAR_HEIGHT,
    },
    utils::{ChannelReceiverM, ChannelSenderCS},
};

#[derive(Debug)]
pub enum MaskCommand {
    WinMove {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    },
    WinSwitchLevel {
        top: bool,
    },
    DeviceConnectionChange {
        connect: bool,
    },
    GetActiveMapping,
    ValidateMappingConfig {
        config: MappingConfig,
    },
    LoadAndActivateMappingConfig {
        file_name: String,
    },
    EvalScript {
        script: String,
    },
    ToggleTitlebar,
}

#[derive(Resource)]
pub struct MaskSize(pub Vec2);

#[derive(Resource)]
pub struct TitlebarState {
    pub visible: bool,
}

impl TitlebarState {
    pub fn offset(&self) -> f32 {
        if self.visible { TITLEBAR_HEIGHT } else { 0.0 }
    }
}

pub fn handle_mask_command(
    m_rx: Res<ChannelReceiverM>,
    cs_tx_res: Res<ChannelSenderCS>,
    cursor_pos: Res<CursorPosition>,
    mut window: Single<&mut Window>,
    mut next_mapping_state: ResMut<NextState<MappingState>>,
    mut next_cursor_state: ResMut<NextState<CursorState>>,
    mut ineffable: IneffableCommands,
    mut active_mapping: ResMut<ActiveMappingConfig>,
    mut mask_size: ResMut<MaskSize>,
    mut titlebar_state: ResMut<TitlebarState>,
) {
    for (msg, oneshot_tx) in m_rx.0.try_iter() {
        match msg {
            MaskCommand::WinMove {
                left,
                top,
                right,
                bottom,
            } => {
                let content_width = (right - left) as f32;
                let content_height = (bottom - top) as f32;

                apply_titlebar_dimensions(
                    &mut window,
                    &mut mask_size,
                    titlebar_state.visible,
                    content_width,
                    content_height,
                    left,
                    top,
                );

                let msg = t!(
                    "mask.windowMovedAndResized",
                    left => left,
                    top => top,
                    width => mask_size.0.x,
                    height => mask_size.0.y
                )
                .to_string();

                log::info!("[Mask] {}", msg);
                oneshot_tx.send(Ok(msg)).unwrap();
            }
            MaskCommand::WinSwitchLevel { top } => {
                if top {
                    window.window_level = WindowLevel::AlwaysOnTop;
                } else {
                    window.window_level = WindowLevel::Normal;
                }
                let msg = format!("[Mask] {}: {}", t!("mask.windowLevelChanged"), top);
                log::info!("{}", msg);
                oneshot_tx.send(Ok(msg)).unwrap();
            }
            MaskCommand::DeviceConnectionChange { connect } => {
                let msg = if connect {
                    next_mapping_state.set(MappingState::Normal);
                    log::info!("[Mapping] {}", t!("mask.enterNormalMappingMode"));
                    window.visible = true;
                    t!("mask.mainDeviceConnected").to_string()
                } else {
                    next_cursor_state.set(CursorState::Normal);
                    next_mapping_state.set(MappingState::Stop);
                    log::info!("[Mapping] {}", t!("mask.exitStopMappingMode"));
                    window.visible = false;
                    t!("mask.mainDeviceDisconnected").to_string()
                };
                log::info!("[Mask] {}", msg);
                oneshot_tx.send(Ok(msg)).unwrap();
            }
            MaskCommand::GetActiveMapping => {
                oneshot_tx.send(Ok(active_mapping.1.clone())).unwrap();
            }
            MaskCommand::ValidateMappingConfig { config } => {
                match validate_mapping_config(&config) {
                    Ok(_) => {
                        oneshot_tx.send(Ok(String::new())).unwrap();
                    }
                    Err(err) => {
                        oneshot_tx.send(Err(err)).unwrap();
                    }
                }
            }
            MaskCommand::LoadAndActivateMappingConfig { file_name } => {
                log::info!(
                    "[Mapping] {}: {}",
                    t!("mask.loadActivateMappingConfig"),
                    file_name
                );
                match load_mapping_config(&file_name) {
                    Ok((mapping_config, input_config)) => {
                        ineffable.set_config(&input_config);
                        active_mapping.0 = Some(mapping_config);
                        active_mapping.1 = file_name;
                        oneshot_tx.send(Ok(String::new())).unwrap();
                    }
                    Err(e) => {
                        oneshot_tx.send(Err(e)).unwrap();
                    }
                }
            }
            MaskCommand::EvalScript { script } => {
                let ast = match ScriptAST::new(&script) {
                    Err(e) => {
                        oneshot_tx.send(Err(e)).unwrap();
                        return;
                    }
                    Ok(ast) => ast,
                };

                if let Some(mapping_config) = &active_mapping.0 {
                    match ast.eval_script(
                        &cs_tx_res.0,
                        mapping_config.original_size.into(),
                        cursor_pos.0,
                        mask_size.0,
                    ) {
                        Err(e) => {
                            oneshot_tx.send(Err(e.to_string())).unwrap();
                            return;
                        }
                        Ok(_) => {
                            oneshot_tx.send(Ok(String::new())).unwrap();
                        }
                    }
                } else {
                    oneshot_tx
                        .send(Err(t!("mask.evalScriptnoMappingError").to_string()))
                        .unwrap();
                }
            }
            MaskCommand::ToggleTitlebar => {
                let new_visible = !titlebar_state.visible;
                LocalConfig::set_titlebar_visible(new_visible);
                titlebar_state.visible = new_visible;

                let bevy::window::WindowPosition::At(pos) = window.position else {
                    unreachable!("window position should always be At")
                };
                let scale_factor = window.resolution.scale_factor() as f32;
                let titlebar_physical = (TITLEBAR_HEIGHT * scale_factor) as i32;

                let content_top = if titlebar_state.visible {
                    // titlebar_state is already new_visible; we need content_top from OLD state.
                    // new_visible=true (old was hidden): content_top = pos.y
                    // new_visible=false (old was visible): content_top = pos.y + titlebar_physical
                    pos.y
                } else {
                    pos.y + titlebar_physical
                };

                let content_width = mask_size.0.x;
                let content_height = mask_size.0.y;

                apply_titlebar_dimensions(
                    &mut window,
                    &mut mask_size,
                    new_visible,
                    content_width,
                    content_height,
                    pos.x,
                    content_top,
                );

                let msg = format!("[Mask] Titlebar visible: {}", new_visible);
                oneshot_tx.send(Ok(msg)).unwrap();
            }
        }
    }
}

fn apply_titlebar_dimensions(
    window: &mut Window,
    mask_size: &mut MaskSize,
    titlebar_visible: bool,
    content_width: f32,
    content_height: f32,
    left: i32,
    top: i32,
) {
    let scale_factor = window.resolution.scale_factor() as f32;
    let titlebar_physical = (TITLEBAR_HEIGHT * scale_factor) as i32;

    let win_height = if titlebar_visible {
        content_height + TITLEBAR_HEIGHT
    } else {
        content_height
    };
    let win_top = if titlebar_visible {
        top - titlebar_physical
    } else {
        top
    };

    window.resolution.set(content_width, win_height);
    window.position.set((left, win_top).into());
    mask_size.0 = Vec2::new(content_width, content_height);
}
