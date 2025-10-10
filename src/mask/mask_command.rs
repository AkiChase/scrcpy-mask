use bevy::{prelude::*, window::WindowLevel};
use bevy_ineffable::prelude::IneffableCommands;
use rust_i18n::t;

use crate::{
    mask::mapping::{
        MappingState,
        config::{
            ActiveMappingConfig, MappingConfig, load_mapping_config, validate_mapping_config,
        },
        cursor::{CursorPosition, CursorState},
        script_helper::ScriptAST,
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
}

#[derive(Resource)]
pub struct MaskSize(pub Vec2);

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
) {
    for (msg, oneshot_tx) in m_rx.0.try_iter() {
        match msg {
            MaskCommand::WinMove {
                left,
                top,
                right,
                bottom,
            } => {
                // logical size and position
                let width = (right - left) as f32;
                let height = (bottom - top) as f32;

                window.resolution.set(width, height);
                window.position.set((left, top).into());

                mask_size.0 = window.resolution.size();

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
        }
    }
}
