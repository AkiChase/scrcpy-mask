use std::{collections::HashMap, time::Duration};

use crate::tokio_tasks::TokioTasksRuntime;
use bevy::{
    ecs::{
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    math::Vec2,
    state::state::State,
    time::{Time, Timer, TimerMode},
};
use bevy_ineffable::prelude::{ContinuousBinding, Ineffable, InputBinding};
use rust_i18n::t;
use serde::{Deserialize, Serialize};

use crate::{
    mask::{
        mapping::{
            MappingState,
            binding::{ButtonBinding, ValidateMappingConfig},
            config::ActiveMappingConfig,
            cursor::{CursorPosition, CursorState},
            script_helper::{
                ScriptAST, ScriptRuntimeCommand, ScriptRuntimeCommandReceiver,
                ScriptRuntimeCommandSender, ScriptSharedState,
            },
            utils::Position,
        },
        mask_command::MaskSize,
    },
    utils::ChannelSenderCS,
};

fn collect_script_error(errors: &mut Vec<String>, label: impl ToString, script: &str) {
    if let Err(e) = ScriptAST::validate_source(script) {
        errors.push(format!("{}:\n{}", label.to_string(), e));
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MappingScriptHooks {
    #[serde(default)]
    pub before_script: String,
    #[serde(default)]
    pub after_script: String,
}

impl MappingScriptHooks {
    pub fn validate(&self) -> Result<(), String> {
        let mut errors = Vec::new();
        collect_script_error(&mut errors, "before_script", &self.before_script);
        collect_script_error(&mut errors, "after_script", &self.after_script);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n"))
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BindMappingScriptHooks {
    pub before_script: String,
    pub after_script: String,
    pub before_script_ast: ScriptAST,
    pub after_script_ast: ScriptAST,
}

impl From<MappingScriptHooks> for BindMappingScriptHooks {
    fn from(value: MappingScriptHooks) -> Self {
        Self {
            before_script_ast: ScriptAST::new(&value.before_script).unwrap(),
            after_script_ast: ScriptAST::new(&value.after_script).unwrap(),
            before_script: value.before_script,
            after_script: value.after_script,
        }
    }
}

pub fn script_init(mut commands: Commands) {
    commands.insert_resource(ActiveScriptMap::default());
    let (runtime_command_tx, runtime_command_rx) =
        crossbeam_channel::unbounded::<ScriptRuntimeCommand>();
    commands.insert_resource(ScriptRuntimeCommandSender(runtime_command_tx));
    commands.insert_resource(ScriptRuntimeCommandReceiver(runtime_command_rx));
    commands.insert_resource(ScriptSharedState::default());
}

#[derive(Debug, Clone)]
pub struct BindMappingScript {
    pub id: String,
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
            id: value.id,
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
    #[serde(default = "crate::mask::mapping::config::default_mapping_id")]
    pub id: String,
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
        collect_script_error(
            &mut errors,
            t!("mask.mapping.pressedScriptError"),
            &self.pressed_script,
        );
        collect_script_error(
            &mut errors,
            t!("mask.mapping.releasedScriptError"),
            &self.released_script,
        );
        collect_script_error(
            &mut errors,
            t!("mask.mapping.heldScriptError"),
            &self.held_script,
        );

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
    script_command_tx: Res<ScriptRuntimeCommandSender>,
    cursor_pos_res: Res<CursorPosition>,
    mask_size_res: Res<MaskSize>,
    mapping_state: Res<State<MappingState>>,
    cursor_state: Res<State<CursorState>>,
    shared_state: Res<ScriptSharedState>,
    runtime: ResMut<TokioTasksRuntime>,
    mut active_map: ResMut<ActiveScriptMap>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("Script") {
                let mapping = mapping.as_ref_script();
                let original_size: Vec2 = active_mapping.original_size.into();
                let cs_tx = cs_tx_res.0.clone();
                let script_command_tx = script_command_tx.0.clone();
                let cursor_pos = cursor_pos_res.0.clone();
                let mask_size = mask_size_res.0;
                let raw_input_flag = mapping_state.get() == &MappingState::RawInput;
                let fps_mode_flag = cursor_state.get() == &CursorState::Fps;
                let interval = Duration::from_millis(mapping.interval as u64);

                if ineffable.just_activated(action.ineff_continuous()) {
                    if !mapping.pressed_script_ast.empty {
                        let ast = mapping.pressed_script_ast.clone();
                        let shared_state = shared_state.as_ref().clone();
                        let state_scope = mapping.id.clone();
                        runtime.spawn_background_task(move |_ctx| async move {
                            if let Err(e) = ast
                                .run_script(
                                    &cs_tx,
                                    &script_command_tx,
                                    &shared_state,
                                    &state_scope,
                                    original_size,
                                    cursor_pos,
                                    mask_size,
                                    raw_input_flag,
                                    fps_mode_flag,
                                )
                                .await
                            {
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
                                state_scope: mapping.id.clone(),
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
                        let script_command_tx = script_command_tx.clone();
                        let shared_state = shared_state.as_ref().clone();
                        let state_scope = mapping.id.clone();
                        runtime.spawn_background_task(move |_ctx| async move {
                            if let Err(e) = ast
                                .run_script(
                                    &cs_tx,
                                    &script_command_tx,
                                    &shared_state,
                                    &state_scope,
                                    original_size,
                                    cursor_pos,
                                    mask_size,
                                    raw_input_flag,
                                    fps_mode_flag,
                                )
                                .await
                            {
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
    state_scope: String,
    held_script_ast: ScriptAST,
}

#[derive(Resource, Default)]
pub struct ActiveScriptMap(HashMap<String, ScriptTimer>);

pub fn handle_script_trigger(
    time: Res<Time>,
    mut active_map: ResMut<ActiveScriptMap>,
    cs_tx_res: Res<ChannelSenderCS>,
    script_command_tx: Res<ScriptRuntimeCommandSender>,
    cursor_pos_res: Res<CursorPosition>,
    mask_size_res: Res<MaskSize>,
    mapping_state: Res<State<MappingState>>,
    cursor_state: Res<State<CursorState>>,
    shared_state: Res<ScriptSharedState>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    for (_, timer) in active_map.0.iter_mut() {
        if timer.timer.tick(time.delta()).just_finished() {
            let cs_tx = cs_tx_res.0.clone();
            let script_command_tx = script_command_tx.0.clone();
            let original_size = timer.original_size;
            let cursor_pos = cursor_pos_res.0;
            let mask_size = mask_size_res.0;
            let raw_input_flag = mapping_state.get() == &MappingState::RawInput;
            let fps_mode_flag = cursor_state.get() == &CursorState::Fps;

            let ast = timer.held_script_ast.clone();
            let shared_state = shared_state.as_ref().clone();
            let state_scope = timer.state_scope.clone();
            runtime.spawn_background_task(move |_ctx| async move {
                if let Err(e) = ast
                    .run_script(
                        &cs_tx,
                        &script_command_tx,
                        &shared_state,
                        &state_scope,
                        original_size,
                        cursor_pos,
                        mask_size,
                        raw_input_flag,
                        fps_mode_flag,
                    )
                    .await
                {
                    log::error!("{}: {}", t!("mask.mapping.heldScriptRuntimeError"), e);
                }
            });
        }
    }
}
