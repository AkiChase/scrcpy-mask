use std::{collections::HashMap, time::Duration};

use bevy::{
    ecs::{
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    math::Vec2,
    time::{Time, Timer, TimerMode},
};
use bevy_ineffable::prelude::{ContinuousBinding, Ineffable, InputBinding};
use bevy_tokio_tasks::TokioTasksRuntime;
use rust_i18n::t;
use serde::{Deserialize, Serialize};

use crate::{
    mask::mapping::{
        binding::{ButtonBinding, ValidateMappingConfig},
        config::ActiveMappingConfig,
        cursor::CursorPosition,
        script_helper::ScriptAST,
        utils::Position,
    },
    utils::ChannelSenderCS,
};

pub fn script_init(mut commands: Commands) {
    commands.insert_resource(ActiveScriptMap::default());
}

#[derive(Debug, Clone)]
pub struct BindMappingScript {
    pub position: Position,
    pub note: String,
    pub pressed_script: String,
    pub released_script: String,
    pub held_script: String,
    pub pressed_script_ast: ScriptAST,
    pub released_script_ast: ScriptAST,
    pub held_script_ast: ScriptAST,
    pub interval: u64,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
}

impl From<MappingScript> for BindMappingScript {
    fn from(value: MappingScript) -> Self {
        Self {
            position: value.position,
            note: value.note,
            pressed_script_ast: ScriptAST::new(&value.pressed_script).unwrap(),
            released_script_ast: ScriptAST::new(&value.released_script).unwrap(),
            held_script_ast: ScriptAST::new(&value.held_script).unwrap(),
            pressed_script: value.pressed_script,
            released_script: value.released_script,
            held_script: value.held_script,
            interval: value.interval,
            bind: value.bind.clone(),
            input_binding: ContinuousBinding::hold(value.bind).0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingScript {
    pub position: Position,
    pub note: String,
    pub pressed_script: String,
    pub released_script: String,
    pub held_script: String,
    pub interval: u64,
    pub bind: ButtonBinding,
}

impl ValidateMappingConfig for MappingScript {
    fn validate(&self) -> Result<(), String> {
        let mut errors = Vec::new();
        if let Err(e) = ScriptAST::new(&self.pressed_script) {
            errors.push(format!("{}:\n{}", t!("mask.mapping.pressedScriptError"), e));
        }
        if let Err(e) = ScriptAST::new(&self.released_script) {
            errors.push(format!(
                "{}:\n{}",
                t!("mask.mapping.releasedScriptError"),
                e
            ));
        }
        if let Err(e) = ScriptAST::new(&self.held_script) {
            errors.push(format!("{}:\n{}", t!("mask.mapping.heldScriptError"), e));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n"))
        }
    }
}

pub fn handle_script(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    cursor_pos_res: Res<CursorPosition>,
    runtime: ResMut<TokioTasksRuntime>,
    mut active_map: ResMut<ActiveScriptMap>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("Script") {
                let mapping = mapping.as_ref_script();
                let original_size: Vec2 = active_mapping.original_size.into();
                let cs_tx = cs_tx_res.0.clone();
                let cursor_pos = cursor_pos_res.0.clone();
                let interval = Duration::from_millis(mapping.interval as u64);

                if ineffable.just_activated(action.ineff_continuous()) {
                    if !mapping.pressed_script_ast.empty {
                        let ast = mapping.pressed_script_ast.clone();
                        runtime.spawn_background_task(move |_ctx| async move {
                            if let Err(e) = ast.eval_script(&cs_tx, original_size, cursor_pos) {
                                log::error!(
                                    "{}: {}",
                                    t!("mask.mapping.pressedScriptRuntimeError"),
                                    e
                                );
                            }
                        });
                    }

                    if !mapping.held_script_ast.empty {
                        let mut timer = Timer::new(interval, TimerMode::Repeating);
                        timer.tick(interval);
                        active_map.0.insert(
                            action.to_string(),
                            ScriptTimer {
                                timer,
                                original_size: original_size,
                                held_script_ast: mapping.held_script_ast.clone(),
                            },
                        );
                    }
                } else if ineffable.just_deactivated(action.ineff_continuous()) {
                    if !mapping.held_script_ast.empty {
                        active_map.0.remove(action.as_ref());
                    }

                    if !mapping.released_script_ast.empty {
                        let ast = mapping.released_script_ast.clone();
                        runtime.spawn_background_task(move |_ctx| async move {
                            if let Err(e) = ast.eval_script(&cs_tx, original_size, cursor_pos) {
                                log::error!(
                                    "{}: {}",
                                    t!("mask.mapping.releasedScriptRuntimeError"),
                                    e
                                );
                            }
                        });
                    }
                }
            }
        }
    }
}

struct ScriptTimer {
    timer: Timer,
    original_size: Vec2,
    held_script_ast: ScriptAST,
}

#[derive(Resource, Default)]
pub struct ActiveScriptMap(HashMap<String, ScriptTimer>);

pub fn handle_script_trigger(
    time: Res<Time>,
    mut active_map: ResMut<ActiveScriptMap>,
    cs_tx_res: Res<ChannelSenderCS>,
    cursor_pos_res: Res<CursorPosition>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    for (_, timer) in active_map.0.iter_mut() {
        if timer.timer.tick(time.delta()).just_finished() {
            let cs_tx = cs_tx_res.0.clone();
            let original_size = timer.original_size;
            let cursor_pos = cursor_pos_res.0;

            let ast = timer.held_script_ast.clone();
            runtime.spawn_background_task(move |_ctx| async move {
                if let Err(e) = ast.eval_script(&cs_tx, original_size, cursor_pos) {
                    log::error!("{}: {}", t!("mask.mapping.heldScriptRuntimeError"), e);
                }
            });
        }
    }
}
