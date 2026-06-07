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
use bevy_ineffable::prelude::{ContinuousBinding, Ineffable, InputBinding, PulseBinding};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::{
    mask::mapping::{
        MappingState,
        binding::{ButtonBinding, ValidateMappingConfig},
        config::ActiveMappingConfig,
        cursor::{CursorPosition, CursorState},
        executor::{
            MappingExecutionError, MappingLifecycleStart, MappingLifecycleState,
            make_mapping_execution_context, run_script_hook, run_with_hooks,
        },
        script::{BindMappingScriptHooks, MappingScriptHooks},
        script_helper::{ScriptRuntimeCommandSender, ScriptSharedState},
        utils::{ControlMsgHelper, Position, default_random_offset, random_offset_vec2},
    },
    mask::mask_command::MaskSize,
    scrcpy::constant::MotionEventAction,
    utils::ChannelSenderCS,
};

pub fn tap_init(mut commands: Commands) {
    commands.insert_resource(ActiveRepeatTapMap::default());
    commands.insert_resource(ActiveSingleTapMap::default());
    commands.insert_resource(SingleTapLifecycleState::default());
    commands.insert_resource(RepeatTapLifecycleState::default());
}

#[derive(Debug, Clone)]
pub struct BindMappingSingleTap {
    pub id: String,
    pub position: Position,
    pub note: String,
    pub pointer_id: u64,
    pub duration: u64,
    pub sync: bool,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
    pub random_offset_x: f32,
    pub random_offset_y: f32,
    pub script_hooks: BindMappingScriptHooks,
}

impl From<MappingSingleTap> for BindMappingSingleTap {
    fn from(value: MappingSingleTap) -> Self {
        Self {
            id: value.id,
            position: value.position,
            note: value.note,
            pointer_id: value.pointer_id,
            duration: value.duration,
            sync: value.sync,
            bind: value.bind.clone(),
            random_offset_x: value.random_offset_x,
            random_offset_y: value.random_offset_y,
            script_hooks: value.script_hooks.into(),
            input_binding: ContinuousBinding::hold(value.bind).0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingSingleTap {
    #[serde(default = "crate::mask::mapping::config::default_mapping_id")]
    pub id: String,
    pub position: Position,
    pub note: String,
    pub pointer_id: u64,
    pub duration: u64,
    pub sync: bool,
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

impl ValidateMappingConfig for MappingSingleTap {
    fn validate(&self) -> Result<(), String> {
        self.script_hooks.validate()
    }
}

#[derive(Resource, Default)]
pub struct ActiveSingleTapMap(HashMap<String, Vec2>);

#[derive(Resource, Default)]
pub struct SingleTapLifecycleState(MappingLifecycleState<SingleTapReleaseContext>);

#[derive(Clone)]
struct SingleTapReleaseContext {
    cursor_pos: Vec2,
    mask_size: Vec2,
    raw_input_flag: bool,
    fps_mode_flag: bool,
}

fn single_tap_has_before_hook(mapping: &BindMappingSingleTap) -> bool {
    !mapping.script_hooks.before_script_ast.empty
}

fn single_tap_has_after_hook(mapping: &BindMappingSingleTap) -> bool {
    !mapping.script_hooks.after_script_ast.empty
}

fn apply_single_tap_down(
    cs_tx: &ChannelSenderCS,
    active_single_tap: &mut ActiveSingleTapMap,
    action: String,
    mapping: &BindMappingSingleTap,
    original_size: Vec2,
) {
    let random_pos = random_offset_vec2(
        mapping.position.into(),
        Vec2::new(mapping.random_offset_x, mapping.random_offset_y),
    );
    ControlMsgHelper::send_touch(
        &cs_tx.0,
        MotionEventAction::Down,
        mapping.pointer_id,
        original_size,
        random_pos,
    );
    active_single_tap.0.insert(action, random_pos);
}

fn apply_single_tap_up(
    cs_tx: &ChannelSenderCS,
    active_single_tap: &mut ActiveSingleTapMap,
    action: &str,
    mapping: &BindMappingSingleTap,
    original_size: Vec2,
) -> bool {
    if let Some(random_pos) = active_single_tap.0.remove(action) {
        ControlMsgHelper::send_touch(
            &cs_tx.0,
            MotionEventAction::Up,
            mapping.pointer_id,
            original_size,
            random_pos,
        );
        true
    } else {
        false
    }
}

pub fn handle_single_tap(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    mut active_single_tap: ResMut<ActiveSingleTapMap>,
    cs_tx_res: Res<ChannelSenderCS>,
    script_command_tx: Res<ScriptRuntimeCommandSender>,
    shared_state: Res<ScriptSharedState>,
    mask_size: Res<MaskSize>,
    cursor_pos: Res<CursorPosition>,
    mapping_state: Res<State<MappingState>>,
    cursor_state: Res<State<CursorState>>,
    runtime: ResMut<TokioTasksRuntime>,
    mut lifecycle_state: ResMut<SingleTapLifecycleState>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("SingleTap") {
                let original_size: Vec2 = active_mapping.original_size.into();
                let mapping = mapping.as_ref_singletap();
                if ineffable.just_activated(action.ineff_continuous()) {
                    if mapping.sync {
                        if single_tap_has_before_hook(mapping) {
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
                                if let Err(e) = run_script_hook(&before_script_ast, &exec_ctx).await
                                {
                                    task_ctx
                                        .run_on_main_thread({
                                            let action = action.clone();
                                            move |main_ctx| {
                                                main_ctx
                                                    .world
                                                    .resource_mut::<SingleTapLifecycleState>()
                                                    .0
                                                    .cancel_start(&action, version);
                                            }
                                        })
                                        .await;
                                    log::error!("[SingleTap] script hook runtime error: {:?}", e);
                                    return;
                                }

                                let pending_release = task_ctx
                                    .run_on_main_thread(move |main_ctx| {
                                        let start = main_ctx
                                            .world
                                            .resource_mut::<SingleTapLifecycleState>()
                                            .0
                                            .finish_start(&action, version);
                                        let pending_release = match start {
                                            MappingLifecycleStart::Stale => return None,
                                            MappingLifecycleStart::Ready { pending_release } => {
                                                pending_release
                                            }
                                        };

                                        let mut active_single_tap =
                                            main_ctx.world.resource_mut::<ActiveSingleTapMap>();
                                        apply_single_tap_down(
                                            &ChannelSenderCS(cs_tx.clone()),
                                            &mut active_single_tap,
                                            action.clone(),
                                            &mapping,
                                            original_size,
                                        );

                                        if pending_release.is_some() {
                                            apply_single_tap_up(
                                                &ChannelSenderCS(cs_tx),
                                                &mut active_single_tap,
                                                &action,
                                                &mapping,
                                                original_size,
                                            );
                                        }

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
                                            run_script_hook(&after_script_ast, &after_exec_ctx)
                                                .await
                                        {
                                            log::error!(
                                                "[SingleTap] script hook runtime error: {:?}",
                                                e
                                            );
                                        }
                                    }
                                }
                            });
                        } else {
                            apply_single_tap_down(
                                &cs_tx_res,
                                &mut active_single_tap,
                                action.to_string(),
                                mapping,
                                original_size,
                            );
                        }
                    } else {
                        let pointer_id = mapping.pointer_id;
                        let random_pos = random_offset_vec2(
                            mapping.position.into(),
                            Vec2::new(mapping.random_offset_x, mapping.random_offset_y),
                        );
                        let duration = Duration::from_millis(mapping.duration as u64);
                        let hooks = mapping.script_hooks.clone();
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
                        runtime.spawn_background_task(move |_ctx| async move {
                            let result = run_with_hooks(hooks, exec_ctx, move |ctx| async move {
                                ControlMsgHelper::send_touch(
                                    &ctx.cs_tx,
                                    MotionEventAction::Down,
                                    pointer_id,
                                    ctx.original_size,
                                    random_pos,
                                );
                                sleep(duration).await;
                                ControlMsgHelper::send_touch(
                                    &ctx.cs_tx,
                                    MotionEventAction::Up,
                                    pointer_id,
                                    ctx.original_size,
                                    random_pos,
                                );
                                Ok::<(), MappingExecutionError>(())
                            })
                            .await;
                            if let Err(e) = result {
                                log::error!("[SingleTap] mapping execution error: {:?}", e);
                            }
                        });
                    }
                } else if mapping.sync && ineffable.just_deactivated(action.ineff_continuous()) {
                    let released = apply_single_tap_up(
                        &cs_tx_res,
                        &mut active_single_tap,
                        action.as_ref(),
                        mapping,
                        original_size,
                    );

                    if released {
                        lifecycle_state.0.clear_pending(action.as_ref());
                    } else if single_tap_has_before_hook(mapping) {
                        lifecycle_state.0.record_early_release(
                            action.as_ref(),
                            SingleTapReleaseContext {
                                cursor_pos: cursor_pos.0,
                                mask_size: mask_size.0,
                                raw_input_flag: mapping_state.get() == &MappingState::RawInput,
                                fps_mode_flag: cursor_state.get() == &CursorState::Fps,
                            },
                        );
                    }

                    if released && single_tap_has_after_hook(mapping) {
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
                        runtime.spawn_background_task(move |_ctx| async move {
                            if let Err(e) = run_script_hook(&after_script_ast, &exec_ctx).await {
                                log::error!("[SingleTap] script hook runtime error: {:?}", e);
                            }
                        });
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BindMappingRepeatTap {
    pub id: String,
    pub position: Position,
    pub note: String,
    pub pointer_id: u64,
    pub duration: u64,
    pub interval: u32,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
    pub random_offset_x: f32,
    pub random_offset_y: f32,
    pub script_hooks: BindMappingScriptHooks,
}

impl From<MappingRepeatTap> for BindMappingRepeatTap {
    fn from(value: MappingRepeatTap) -> Self {
        Self {
            id: value.id,
            position: value.position,
            note: value.note,
            pointer_id: value.pointer_id,
            duration: value.duration,
            interval: value.interval,
            bind: value.bind.clone(),
            input_binding: ContinuousBinding::hold(value.bind).0,
            random_offset_x: value.random_offset_x,
            random_offset_y: value.random_offset_y,
            script_hooks: value.script_hooks.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingRepeatTap {
    #[serde(default = "crate::mask::mapping::config::default_mapping_id")]
    pub id: String,
    pub position: Position,
    pub note: String,
    pub pointer_id: u64,
    pub duration: u64,
    pub interval: u32,
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

impl ValidateMappingConfig for MappingRepeatTap {
    fn validate(&self) -> Result<(), String> {
        self.script_hooks.validate()
    }
}

#[derive(Resource, Default)]
pub struct ActiveRepeatTapMap(HashMap<String, RepeatTapTimer>);

#[derive(Resource, Default)]
pub struct RepeatTapLifecycleState(MappingLifecycleState<RepeatTapReleaseContext>);

#[derive(Clone)]
struct RepeatTapReleaseContext {
    cursor_pos: Vec2,
    mask_size: Vec2,
    raw_input_flag: bool,
    fps_mode_flag: bool,
}

struct RepeatTapTimer {
    timer: Timer,
    pointer_id: u64,
    original_pos: Vec2,
    original_size: Vec2,
    duration: Duration,
    random_offset: Vec2,
}

fn repeat_tap_has_before_hook(mapping: &BindMappingRepeatTap) -> bool {
    !mapping.script_hooks.before_script_ast.empty
}

fn repeat_tap_has_after_hook(mapping: &BindMappingRepeatTap) -> bool {
    !mapping.script_hooks.after_script_ast.empty
}

fn spawn_repeat_tap_once(
    runtime: &TokioTasksRuntime,
    cs_tx: &ChannelSenderCS,
    pointer_id: u64,
    original_size: Vec2,
    original_pos: Vec2,
    random_offset: Vec2,
    duration: Duration,
) {
    let cs_tx = cs_tx.0.clone();
    let random_pos = random_offset_vec2(original_pos, random_offset);
    ControlMsgHelper::send_touch(
        &cs_tx,
        MotionEventAction::Down,
        pointer_id,
        original_size,
        random_pos,
    );
    runtime.spawn_background_task(move |_ctx| async move {
        sleep(duration).await;
        ControlMsgHelper::send_touch(
            &cs_tx,
            MotionEventAction::Up,
            pointer_id,
            original_size,
            random_pos,
        );
    });
}

fn make_repeat_tap_timer(mapping: &BindMappingRepeatTap, original_size: Vec2) -> RepeatTapTimer {
    RepeatTapTimer {
        timer: {
            let interval = Duration::from_millis(mapping.interval as u64);
            let mut timer = Timer::new(interval, TimerMode::Repeating);
            timer.tick(interval);
            timer
        },
        pointer_id: mapping.pointer_id,
        original_pos: mapping.position.into(),
        original_size,
        duration: Duration::from_millis(mapping.duration as u64),
        random_offset: Vec2::new(mapping.random_offset_x, mapping.random_offset_y),
    }
}

pub fn handle_repeat_tap_trigger(
    time: Res<Time>,
    mut active_map: ResMut<ActiveRepeatTapMap>,
    cs_tx_res: Res<ChannelSenderCS>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    for (_, timer) in active_map.0.iter_mut() {
        if timer.timer.tick(time.delta()).just_finished() {
            spawn_repeat_tap_once(
                &runtime,
                &cs_tx_res,
                timer.pointer_id,
                timer.original_size,
                timer.original_pos,
                timer.random_offset,
                timer.duration,
            );
        }
    }
}

pub fn handle_repeat_tap(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    mut active_map: ResMut<ActiveRepeatTapMap>,
    cs_tx_res: Res<ChannelSenderCS>,
    script_command_tx: Res<ScriptRuntimeCommandSender>,
    shared_state: Res<ScriptSharedState>,
    mask_size: Res<MaskSize>,
    cursor_pos: Res<CursorPosition>,
    mapping_state: Res<State<MappingState>>,
    cursor_state: Res<State<CursorState>>,
    runtime: ResMut<TokioTasksRuntime>,
    mut lifecycle_state: ResMut<RepeatTapLifecycleState>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("RepeatTap") {
                let mapping = mapping.as_ref_repeattap();
                if ineffable.just_activated(action.ineff_continuous()) {
                    let original_size: Vec2 = active_mapping.original_size.into();
                    if repeat_tap_has_before_hook(mapping) {
                        let action = action.to_string();
                        let version = lifecycle_state.0.begin_start(&action);
                        let mapping = mapping.clone();
                        let before_script_ast = mapping.script_hooks.before_script_ast.clone();
                        let after_script_ast = mapping.script_hooks.after_script_ast.clone();
                        let pointer_id = mapping.pointer_id;
                        let original_pos: Vec2 = mapping.position.into();
                        let random_offset =
                            Vec2::new(mapping.random_offset_x, mapping.random_offset_y);
                        let duration = Duration::from_millis(mapping.duration as u64);
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
                        runtime.spawn_background_task(move |mut task_ctx| async move {
                            if let Err(e) = run_script_hook(&before_script_ast, &exec_ctx).await {
                                task_ctx
                                    .run_on_main_thread({
                                        let action = action.clone();
                                        move |main_ctx| {
                                            main_ctx
                                                .world
                                                .resource_mut::<RepeatTapLifecycleState>()
                                                .0
                                                .cancel_start(&action, version);
                                        }
                                    })
                                    .await;
                                log::error!("[RepeatTap] script hook runtime error: {:?}", e);
                                return;
                            }

                            let pending_release = task_ctx
                                .run_on_main_thread(move |main_ctx| {
                                    let start = main_ctx
                                        .world
                                        .resource_mut::<RepeatTapLifecycleState>()
                                        .0
                                        .finish_start(&action, version);
                                    let pending_release = match start {
                                        MappingLifecycleStart::Stale => return None,
                                        MappingLifecycleStart::Ready { pending_release } => {
                                            pending_release
                                        }
                                    };

                                    if pending_release.is_none() {
                                        let mut active_map =
                                            main_ctx.world.resource_mut::<ActiveRepeatTapMap>();
                                        active_map.0.insert(
                                            action.clone(),
                                            make_repeat_tap_timer(&mapping, original_size),
                                        );
                                    }

                                    pending_release
                                })
                                .await;

                            if let Some(release) = pending_release {
                                let random_pos = random_offset_vec2(original_pos, random_offset);
                                ControlMsgHelper::send_touch(
                                    &exec_ctx.cs_tx,
                                    MotionEventAction::Down,
                                    pointer_id,
                                    original_size,
                                    random_pos,
                                );
                                sleep(duration).await;
                                ControlMsgHelper::send_touch(
                                    &exec_ctx.cs_tx,
                                    MotionEventAction::Up,
                                    pointer_id,
                                    original_size,
                                    random_pos,
                                );

                                if !after_script_ast.empty {
                                    let mut after_exec_ctx = exec_ctx.clone();
                                    after_exec_ctx.cursor_pos = release.cursor_pos;
                                    after_exec_ctx.mask_size = release.mask_size;
                                    after_exec_ctx.raw_input_flag = release.raw_input_flag;
                                    after_exec_ctx.fps_mode_flag = release.fps_mode_flag;
                                    if let Err(e) =
                                        run_script_hook(&after_script_ast, &after_exec_ctx).await
                                    {
                                        log::error!(
                                            "[RepeatTap] script hook runtime error: {:?}",
                                            e
                                        );
                                    }
                                }
                            }
                        });
                    } else {
                        active_map.0.insert(
                            action.to_string(),
                            make_repeat_tap_timer(mapping, original_size),
                        );
                    }
                } else if ineffable.just_deactivated(action.ineff_continuous()) {
                    let released = active_map.0.remove(action.as_ref()).is_some();

                    if released {
                        lifecycle_state.0.clear_pending(action.as_ref());
                    } else if repeat_tap_has_before_hook(mapping) {
                        lifecycle_state.0.record_early_release(
                            action.as_ref(),
                            RepeatTapReleaseContext {
                                cursor_pos: cursor_pos.0,
                                mask_size: mask_size.0,
                                raw_input_flag: mapping_state.get() == &MappingState::RawInput,
                                fps_mode_flag: cursor_state.get() == &CursorState::Fps,
                            },
                        );
                    }

                    if released && repeat_tap_has_after_hook(mapping) {
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
                        runtime.spawn_background_task(move |_ctx| async move {
                            if let Err(e) = run_script_hook(&after_script_ast, &exec_ctx).await {
                                log::error!("[RepeatTap] script hook runtime error: {:?}", e);
                            }
                        });
                    }
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingMultipleTapItem {
    pub position: Position,
    pub duration: u64,
    pub wait: u64,
}

#[derive(Debug, Clone)]
pub struct BindMappingMultipleTap {
    pub id: String,
    pub note: String,
    pub pointer_id: u64,
    pub items: Vec<MappingMultipleTapItem>,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
    pub random_offset_x: f32,
    pub random_offset_y: f32,
    pub script_hooks: BindMappingScriptHooks,
}

impl From<MappingMultipleTap> for BindMappingMultipleTap {
    fn from(value: MappingMultipleTap) -> Self {
        Self {
            id: value.id,
            note: value.note,
            pointer_id: value.pointer_id,
            items: value.items,
            bind: value.bind.clone(),
            input_binding: PulseBinding::just_pressed(value.bind).0,
            random_offset_x: value.random_offset_x,
            random_offset_y: value.random_offset_y,
            script_hooks: value.script_hooks.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingMultipleTap {
    #[serde(default = "crate::mask::mapping::config::default_mapping_id")]
    pub id: String,
    pub note: String,
    pub pointer_id: u64,
    pub items: Vec<MappingMultipleTapItem>,
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

impl ValidateMappingConfig for MappingMultipleTap {
    fn validate(&self) -> Result<(), String> {
        if self.items.is_empty() {
            return Err("MultipleTap's operation item list is empty".to_string());
        }
        self.script_hooks.validate()
    }
}

pub fn handle_multiple_tap(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    script_command_tx: Res<ScriptRuntimeCommandSender>,
    shared_state: Res<ScriptSharedState>,
    mask_size: Res<MaskSize>,
    cursor_pos: Res<CursorPosition>,
    mapping_state: Res<State<MappingState>>,
    cursor_state: Res<State<CursorState>>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("MultipleTap") {
                let mapping = mapping.as_ref_multipletap();
                if ineffable.just_pulsed(action.ineff_pulse()) {
                    let original_size: Vec2 = active_mapping.original_size.into();
                    let pointer_id = mapping.pointer_id;
                    let items = mapping.items.clone();
                    let random_offset = Vec2::new(mapping.random_offset_x, mapping.random_offset_y);
                    let hooks = mapping.script_hooks.clone();
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
                    runtime.spawn_background_task(move |_ctx| async move {
                        let result = run_with_hooks(hooks, exec_ctx, move |ctx| async move {
                            for item in items {
                                let random_pos =
                                    random_offset_vec2(item.position.into(), random_offset);
                                sleep(Duration::from_millis(item.wait)).await;
                                ControlMsgHelper::send_touch(
                                    &ctx.cs_tx,
                                    MotionEventAction::Down,
                                    pointer_id,
                                    ctx.original_size,
                                    random_pos,
                                );
                                sleep(Duration::from_millis(item.duration)).await;
                                ControlMsgHelper::send_touch(
                                    &ctx.cs_tx,
                                    MotionEventAction::Up,
                                    pointer_id,
                                    ctx.original_size,
                                    random_pos,
                                );
                            }
                            Ok::<(), MappingExecutionError>(())
                        })
                        .await;
                        if let Err(e) = result {
                            log::error!("[MultipleTap] mapping execution error: {:?}", e);
                        }
                    });
                }
            }
        }
    }
}
