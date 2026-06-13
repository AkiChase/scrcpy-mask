use std::collections::HashMap;

use bevy::{
    ecs::{
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    input::mouse::AccumulatedMouseMotion,
    math::Vec2,
    state::state::{NextState, State},
};
use bevy_ineffable::prelude::{ContinuousBinding, Ineffable, InputBinding, PulseBinding};
use rust_i18n::t;
use serde::{Deserialize, Serialize};

use crate::{
    mask::{
        mapping::{
            MappingState,
            binding::{ButtonBinding, ValidateMappingConfig},
            config::{ActiveMappingConfig, BindMappingConfig, BindMappingType},
            cursor::{
                ActiveCursorFpsConfig, CursorPosition, CursorState, FPS_MARGIN, FpsTouchMode,
                release_fps_touches, restore_fps_touch,
            },
            executor::{
                MappingLifecycleStart, MappingLifecycleState, make_mapping_execution_context,
                run_script_hook,
            },
            script::{BindMappingScriptHooks, MappingScriptHooks},
            script_helper::{ScriptRuntimeCommandSender, ScriptSharedState},
            utils::{ControlMsgHelper, Position, default_random_offset, random_offset_vec2},
        },
        mask_command::MaskSize,
    },
    scrcpy::constant::MotionEventAction,
    tokio_tasks::TokioTasksRuntime,
    utils::ChannelSenderCS,
};
use tokio::sync::broadcast;

pub fn fire_init(mut commands: Commands) {
    commands.insert_resource(ActiveFireMap::default());
    commands.insert_resource(FireLifecycleState::default());
}

fn default_fps_max_offset() -> f32 {
    -1.0
}

#[derive(Debug, Clone)]
pub struct BindMappingFps {
    pub id: String,
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub sensitivity_x: f32,
    pub sensitivity_y: f32,
    pub max_offset_x: f32,
    pub max_offset_y: f32,
    pub touch_mode: FpsTouchMode,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
}

impl From<MappingFps> for BindMappingFps {
    fn from(value: MappingFps) -> Self {
        Self {
            id: value.id,
            note: value.note,
            pointer_id: value.pointer_id,
            position: value.position,
            sensitivity_x: value.sensitivity_x,
            sensitivity_y: value.sensitivity_y,
            max_offset_x: value.max_offset_x,
            max_offset_y: value.max_offset_y,
            touch_mode: value.touch_mode,
            bind: value.bind.clone(),
            input_binding: PulseBinding::just_pressed(value.bind).0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingFps {
    #[serde(default = "crate::mask::mapping::config::default_mapping_id")]
    pub id: String,
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    #[serde(serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp")]
    pub sensitivity_x: f32,
    #[serde(serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp")]
    pub sensitivity_y: f32,
    #[serde(
        default = "default_fps_max_offset",
        serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp"
    )]
    pub max_offset_x: f32,
    #[serde(
        default = "default_fps_max_offset",
        serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp"
    )]
    pub max_offset_y: f32,
    #[serde(default)]
    pub touch_mode: FpsTouchMode,
    pub bind: ButtonBinding,
}

pub fn enter_fps_mode(
    cs_tx: &broadcast::Sender<crate::scrcpy::control_msg::ScrcpyControlMsg>,
    fps_config: &mut ActiveCursorFpsConfig,
    next_state: &mut NextState<CursorState>,
    mapping: &BindMappingFps,
    original_size: Vec2,
) {
    let original_pos = mapping.position.into();
    fps_config.pointer_id = mapping.pointer_id;
    fps_config.reset_touch_state();
    fps_config.original_pos = original_pos;
    fps_config.original_size = original_size;
    fps_config.ignore_fps_motion = false;
    fps_config.sensitivity = (mapping.sensitivity_x, mapping.sensitivity_y).into();
    fps_config.max_offset = Vec2::new(mapping.max_offset_x, mapping.max_offset_y);
    fps_config.touch_mode = mapping.touch_mode;

    ControlMsgHelper::send_touch(
        cs_tx,
        MotionEventAction::Down,
        mapping.pointer_id,
        original_size,
        original_pos,
    );
    next_state.set(CursorState::Fps);
}

pub fn exit_fps_mode(
    cs_tx: &broadcast::Sender<crate::scrcpy::control_msg::ScrcpyControlMsg>,
    fps_config: &mut ActiveCursorFpsConfig,
    active_fire_map: &mut ActiveFireMap,
    next_state: &mut NextState<CursorState>,
    mask_size: Vec2,
    cursor_pos: Vec2,
) -> Vec<String> {
    let released_fire_actions = release_active_fire(cs_tx, active_fire_map, mask_size);
    if released_fire_actions.is_empty() {
        release_fps_touches(cs_tx, fps_config, mask_size, cursor_pos);
    }
    fps_config.ignore_fps_motion = false;
    next_state.set(CursorState::Normal);
    released_fire_actions
}

impl ValidateMappingConfig for MappingFps {
    fn validate(&self) -> Result<(), String> {
        if self.position.x <= FPS_MARGIN as i32 || self.position.y <= FPS_MARGIN as i32 {
            return Err(t!(
                "mask.mapping.invalidPosition",
                x => self.position.x,
                y => self.position.y,
                margin => FPS_MARGIN
            )
            .to_string());
        }
        if self.max_offset_x < -1.0 || self.max_offset_y < -1.0 {
            return Err("FPS max_offset_x/max_offset_y must be -1 or greater".to_string());
        }
        if let Some(another_pointer_id) = self.touch_mode.another_pointer_id()
            && another_pointer_id == self.pointer_id
        {
            return Err(
                "FPS touch_mode another_pointer_id must differ from pointer_id".to_string(),
            );
        }
        Ok(())
    }
}

pub fn handle_fps(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    script_command_tx: Res<ScriptRuntimeCommandSender>,
    shared_state: Res<ScriptSharedState>,
    mut fps_config: ResMut<ActiveCursorFpsConfig>,
    mut active_fire_map: ResMut<ActiveFireMap>,
    mapping_state: Res<State<MappingState>>,
    state: Res<State<CursorState>>,
    mut next_state: ResMut<NextState<CursorState>>,
    cursor_pos: Res<CursorPosition>,
    mask_size: Res<MaskSize>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("Fps") {
                if ineffable.just_pulsed(action.ineff_pulse()) {
                    let original_size: Vec2 = active_mapping.original_size.into();
                    match state.get() {
                        CursorState::Normal => {
                            let mapping = mapping.as_ref_fps();
                            enter_fps_mode(
                                &cs_tx_res.0,
                                &mut fps_config,
                                &mut next_state,
                                mapping,
                                original_size,
                            );
                        }
                        CursorState::Fps => {
                            let released_fire_actions = exit_fps_mode(
                                &cs_tx_res.0,
                                &mut fps_config,
                                &mut active_fire_map,
                                &mut next_state,
                                mask_size.0,
                                cursor_pos.0,
                            );
                            spawn_fire_after_hooks_for_external_release(
                                released_fire_actions,
                                active_mapping,
                                &cs_tx_res,
                                &script_command_tx,
                                &shared_state,
                                &runtime,
                                cursor_pos.0,
                                mask_size.0,
                                mapping_state.get() == &MappingState::RawInput,
                                true,
                            );
                        }
                    };
                    return;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BindMappingFire {
    pub id: String,
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub sensitivity_x: f32,
    pub sensitivity_y: f32,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
    pub random_offset_x: f32,
    pub random_offset_y: f32,
    pub script_hooks: BindMappingScriptHooks,
}

impl From<MappingFire> for BindMappingFire {
    fn from(value: MappingFire) -> Self {
        Self {
            id: value.id,
            note: value.note,
            pointer_id: value.pointer_id,
            position: value.position,
            sensitivity_x: value.sensitivity_x,
            sensitivity_y: value.sensitivity_y,
            bind: value.bind.clone(),
            input_binding: ContinuousBinding::hold(value.bind).0,
            random_offset_x: value.random_offset_x,
            random_offset_y: value.random_offset_y,
            script_hooks: value.script_hooks.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingFire {
    #[serde(default = "crate::mask::mapping::config::default_mapping_id")]
    pub id: String,
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    #[serde(serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp")]
    pub sensitivity_x: f32,
    #[serde(serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp")]
    pub sensitivity_y: f32,
    pub bind: ButtonBinding,
    #[serde(
        default = "default_random_offset",
        serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp"
    )]
    pub random_offset_x: f32,
    #[serde(
        default = "default_random_offset",
        serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp"
    )]
    pub random_offset_y: f32,
    #[serde(default)]
    pub script_hooks: MappingScriptHooks,
}

impl ValidateMappingConfig for MappingFire {
    fn validate(&self) -> Result<(), String> {
        self.script_hooks.validate()
    }
}

#[derive(Resource, Default)]
pub struct ActiveFireMap(HashMap<String, FireItem>);

#[derive(Resource, Default)]
pub struct FireLifecycleState(MappingLifecycleState<FireReleaseContext>);

#[derive(Clone)]
struct FireReleaseContext {
    mask_size: Vec2,
    cursor_pos: Vec2,
    raw_input_flag: bool,
    fps_mode_flag: bool,
}

fn release_active_fire(
    cs_tx: &broadcast::Sender<crate::scrcpy::control_msg::ScrcpyControlMsg>,
    active_map: &mut ActiveFireMap,
    mask_size: Vec2,
) -> Vec<String> {
    let mut released_actions = Vec::with_capacity(active_map.0.len());
    for (action, fire_item) in active_map.0.drain() {
        ControlMsgHelper::send_touch(
            cs_tx,
            MotionEventAction::Up,
            fire_item.pointer_id,
            mask_size,
            fire_item.current_pos,
        );
        released_actions.push(action);
    }
    released_actions
}

pub fn spawn_fire_after_hooks_for_external_release(
    released_actions: Vec<String>,
    active_mapping: &BindMappingConfig,
    cs_tx_res: &ChannelSenderCS,
    script_command_tx: &ScriptRuntimeCommandSender,
    shared_state: &ScriptSharedState,
    runtime: &TokioTasksRuntime,
    cursor_pos: Vec2,
    mask_size: Vec2,
    raw_input_flag: bool,
    fps_mode_flag: bool,
) {
    if released_actions.is_empty() {
        return;
    }

    let original_size: Vec2 = active_mapping.original_size.into();
    for released_action in released_actions {
        let Some((_, BindMappingType::Fire(mapping))) = active_mapping
            .mappings
            .iter()
            .find(|(action, _)| action.as_ref() == released_action)
        else {
            continue;
        };

        if mapping.script_hooks.after_script_ast.empty {
            continue;
        }

        let after_script_ast = mapping.script_hooks.after_script_ast.clone();
        let exec_ctx = make_mapping_execution_context(
            cs_tx_res,
            script_command_tx,
            shared_state,
            mapping.id.clone(),
            original_size,
            cursor_pos,
            mask_size,
            raw_input_flag,
            fps_mode_flag,
        );
        runtime.spawn_background_task(move |_task_ctx| async move {
            if let Err(e) = run_script_hook(&after_script_ast, &exec_ctx).await {
                log::error!("[Fire] script hook runtime error: {:?}", e);
            }
        });
    }
}

pub fn handle_fire_trigger(
    accumulated_motion: Res<AccumulatedMouseMotion>,
    cs_tx_res: Res<ChannelSenderCS>,
    mask_size: Res<MaskSize>,
    mut active_map: ResMut<ActiveFireMap>,
) {
    if active_map.0.is_empty()
        || (accumulated_motion.delta.x == 0. && accumulated_motion.delta.y == 0.)
    {
        return;
    }

    for (_, fire_item) in active_map.0.iter_mut() {
        fire_item.current_pos += accumulated_motion.delta * fire_item.sensitivity;
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Move,
            fire_item.pointer_id,
            mask_size.0,
            fire_item.current_pos, // fire item cursor pos
        );
    }
}

struct FireItem {
    current_pos: Vec2,
    pointer_id: u64,
    sensitivity: Vec2,
}

fn fire_has_before_hook(mapping: &BindMappingFire) -> bool {
    !mapping.script_hooks.before_script_ast.empty
}

fn fire_has_after_hook(mapping: &BindMappingFire) -> bool {
    !mapping.script_hooks.after_script_ast.empty
}

fn apply_fire_begin(
    cs_tx: &ChannelSenderCS,
    fps_config: &mut ActiveCursorFpsConfig,
    active_map: &mut ActiveFireMap,
    action: String,
    mapping: &BindMappingFire,
    original_size: Vec2,
    mask_size: Vec2,
    cursor_pos: Vec2,
) {
    fps_config.ignore_fps_motion = true;
    release_fps_touches(&cs_tx.0, fps_config, mask_size, cursor_pos);

    let original_pos: Vec2 = mapping.position.into();
    let random_pos = random_offset_vec2(
        original_pos,
        Vec2::new(mapping.random_offset_x, mapping.random_offset_y),
    );
    let sensitivity: Vec2 = (mapping.sensitivity_x, mapping.sensitivity_y).into();
    let current_pos = random_pos / original_size * mask_size;

    ControlMsgHelper::send_touch(
        &cs_tx.0,
        MotionEventAction::Down,
        mapping.pointer_id,
        original_size,
        random_pos,
    );
    active_map.0.insert(
        action,
        FireItem {
            current_pos,
            pointer_id: mapping.pointer_id,
            sensitivity,
        },
    );
}

fn apply_fire_end(
    cs_tx: &ChannelSenderCS,
    fps_config: &mut ActiveCursorFpsConfig,
    active_map: &mut ActiveFireMap,
    cursor_pos: &mut CursorPosition,
    mask_size: Vec2,
    action: &str,
) -> bool {
    if let Some(fire_item) = active_map.0.remove(action) {
        ControlMsgHelper::send_touch(
            &cs_tx.0,
            MotionEventAction::Up,
            fire_item.pointer_id,
            mask_size,
            fire_item.current_pos,
        );
        restore_fps_touch(&cs_tx.0, fps_config);
        cursor_pos.0 = fps_config.original_pos / fps_config.original_size * mask_size;
        fps_config.ignore_fps_motion = false;
        true
    } else {
        false
    }
}

pub fn handle_fire(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    script_command_tx: Res<ScriptRuntimeCommandSender>,
    shared_state: Res<ScriptSharedState>,
    mapping_state: Res<State<MappingState>>,
    cursor_state: Res<State<CursorState>>,
    runtime: ResMut<TokioTasksRuntime>,
    mut fps_config: ResMut<ActiveCursorFpsConfig>,
    mut active_map: ResMut<ActiveFireMap>,
    mut cursor_pos: ResMut<CursorPosition>,
    mask_size: Res<MaskSize>,
    mut lifecycle_state: ResMut<FireLifecycleState>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("Fire") {
                let mapping = mapping.as_ref_fire();
                if ineffable.just_activated(action.ineff_continuous()) {
                    let original_size: Vec2 = active_mapping.original_size.into();
                    if fire_has_before_hook(mapping) {
                        let action = action.to_string();
                        let version = lifecycle_state.0.begin_start(&action);
                        let mapping = mapping.clone();
                        let before_script_ast = mapping.script_hooks.before_script_ast.clone();
                        let after_script_ast = mapping.script_hooks.after_script_ast.clone();
                        let exec_ctx = make_mapping_execution_context(
                            &cs_tx_res,
                            &script_command_tx,
                            &shared_state,
                            mapping.id.clone(),
                            original_size,
                            cursor_pos.0,
                            mask_size.0,
                            mapping_state.get() == &MappingState::RawInput,
                            cursor_state.get() == &CursorState::Fps,
                        );
                        let cs_tx = cs_tx_res.0.clone();
                        runtime.spawn_background_task(move |mut task_ctx| async move {
                            if let Err(e) = run_script_hook(&before_script_ast, &exec_ctx).await {
                                task_ctx
                                    .run_on_main_thread({
                                        let action = action.clone();
                                        move |main_ctx| {
                                            main_ctx
                                                .world
                                                .resource_mut::<FireLifecycleState>()
                                                .0
                                                .cancel_start(&action, version);
                                        }
                                    })
                                    .await;
                                log::error!("[Fire] script hook runtime error: {:?}", e);
                                return;
                            }

                            let pending_release = task_ctx
                                .run_on_main_thread(move |main_ctx| {
                                    let start = main_ctx
                                        .world
                                        .resource_mut::<FireLifecycleState>()
                                        .0
                                        .finish_start(&action, version);
                                    let pending_release = match start {
                                        MappingLifecycleStart::Stale => return None,
                                        MappingLifecycleStart::Ready { pending_release } => {
                                            pending_release
                                        }
                                    };
                                    if main_ctx.world.resource::<State<CursorState>>().get()
                                        != &CursorState::Fps
                                    {
                                        return None;
                                    }

                                    let current_cursor_pos =
                                        main_ctx.world.resource::<CursorPosition>().0;
                                    let current_mask_size = main_ctx.world.resource::<MaskSize>().0;
                                    let mut active_map = main_ctx
                                        .world
                                        .remove_resource::<ActiveFireMap>()
                                        .expect("ActiveFireMap resource missing");
                                    {
                                        let mut fps_config =
                                            main_ctx.world.resource_mut::<ActiveCursorFpsConfig>();
                                        apply_fire_begin(
                                            &ChannelSenderCS(cs_tx.clone()),
                                            &mut fps_config,
                                            &mut active_map,
                                            action.clone(),
                                            &mapping,
                                            original_size,
                                            current_mask_size,
                                            current_cursor_pos,
                                        );
                                    }
                                    if let Some(release) = &pending_release {
                                        if let Some(fire_item) = active_map.0.remove(&action) {
                                            ControlMsgHelper::send_touch(
                                                &cs_tx,
                                                MotionEventAction::Up,
                                                fire_item.pointer_id,
                                                release.mask_size,
                                                fire_item.current_pos,
                                            );
                                            let (fps_original_size, fps_original_pos) = {
                                                let mut fps_config = main_ctx
                                                    .world
                                                    .resource_mut::<ActiveCursorFpsConfig>(
                                                );
                                                fps_config.ignore_fps_motion = false;
                                                restore_fps_touch(&cs_tx, &mut fps_config);
                                                (fps_config.original_size, fps_config.original_pos)
                                            };
                                            main_ctx.world.resource_mut::<CursorPosition>().0 =
                                                fps_original_pos / fps_original_size
                                                    * release.mask_size;
                                        }
                                    }
                                    main_ctx.world.insert_resource(active_map);

                                    pending_release
                                })
                                .await;

                            if let Some(release) = pending_release {
                                if !after_script_ast.empty {
                                    let mut after_exec_ctx = exec_ctx.clone();
                                    after_exec_ctx.cursor_pos = release.cursor_pos;
                                    after_exec_ctx.mask_size = release.mask_size;
                                    after_exec_ctx.raw_input_flag = release.raw_input_flag;
                                    after_exec_ctx.fps_mode_flag = release.fps_mode_flag;
                                    if let Err(e) =
                                        run_script_hook(&after_script_ast, &after_exec_ctx).await
                                    {
                                        log::error!("[Fire] script hook runtime error: {:?}", e);
                                    }
                                }
                            }
                        });
                    } else {
                        apply_fire_begin(
                            &cs_tx_res,
                            &mut fps_config,
                            &mut active_map,
                            action.to_string(),
                            mapping,
                            original_size,
                            mask_size.0,
                            cursor_pos.0,
                        );
                    }
                } else if ineffable.just_deactivated(action.ineff_continuous()) {
                    let released = apply_fire_end(
                        &cs_tx_res,
                        &mut fps_config,
                        &mut active_map,
                        &mut cursor_pos,
                        mask_size.0,
                        action.as_ref(),
                    );

                    if released {
                        lifecycle_state.0.clear_pending(action.as_ref());
                    } else if fire_has_before_hook(mapping) {
                        lifecycle_state.0.record_early_release(
                            action.as_ref(),
                            FireReleaseContext {
                                mask_size: mask_size.0,
                                cursor_pos: cursor_pos.0,
                                raw_input_flag: mapping_state.get() == &MappingState::RawInput,
                                fps_mode_flag: cursor_state.get() == &CursorState::Fps,
                            },
                        );
                    }

                    if released && fire_has_after_hook(mapping) {
                        let after_script_ast = mapping.script_hooks.after_script_ast.clone();
                        let exec_ctx = make_mapping_execution_context(
                            &cs_tx_res,
                            &script_command_tx,
                            &shared_state,
                            mapping.id.clone(),
                            active_mapping.original_size.into(),
                            cursor_pos.0,
                            mask_size.0,
                            mapping_state.get() == &MappingState::RawInput,
                            cursor_state.get() == &CursorState::Fps,
                        );
                        runtime.spawn_background_task(move |_task_ctx| async move {
                            if let Err(e) = run_script_hook(&after_script_ast, &exec_ctx).await {
                                log::error!("[Fire] script hook runtime error: {:?}", e);
                            }
                        });
                    }
                }
            }
        }
    }
}
