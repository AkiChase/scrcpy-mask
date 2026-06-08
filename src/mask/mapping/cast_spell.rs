use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Instant,
};

use crate::tokio_tasks::TokioTasksRuntime;
use bevy::{
    ecs::{
        resource::Resource,
        system::{Commands, Local, Res, ResMut, Single},
    },
    math::Vec2,
    state::state::State,
    window::Window,
};
use bevy_ineffable::prelude::{ContinuousBinding, Ineffable, InputBinding, PulseBinding};
use serde::{Deserialize, Serialize};

use crate::{
    mask::{
        mapping::{
            MappingState,
            binding::{ButtonBinding, DirectionBinding, ValidateMappingConfig},
            config::{ActiveMappingConfig, BindMappingConfig, BindMappingType, MappingAction},
            cursor::{CursorPosition, CursorState, NormalCursorCapture},
            direction_pad::{BlockDirectionPad, DirectionPadMap},
            executor::{
                MappingExecutionContext, MappingExecutionError, MappingLifecycleStart,
                MappingLifecycleState, make_mapping_execution_context, run_script_hook,
                run_with_hooks,
            },
            script::{BindMappingScriptHooks, MappingScriptHooks},
            script_helper::ScriptAST,
            script_helper::{ScriptRuntimeCommand, ScriptRuntimeCommandSender, ScriptSharedState},
            utils::{
                ControlMsgHelper, DEFAULT_SWIPE_DURATION, Position, SingleSwipeStrategy,
                anchor_random_offset, build_single_segment_swipe_intermediate_points,
                default_random_offset, handle_direction_jitter, handle_direction_move_randomized,
                random_offset_vec2, spawn_initial_swipe,
            },
        },
        mask_command::MaskSize,
    },
    scrcpy::constant::MotionEventAction,
    utils::ChannelSenderCS,
};
use tokio::sync::broadcast;
use tokio::sync::oneshot;

pub fn cast_spell_init(mut commands: Commands) {
    commands.insert_resource(ActiveCastSpell::default());
    commands.insert_resource(MouseCastSpellLifecycleState::default());
    commands.insert_resource(PadCastSpellLifecycleState::default());
}

#[derive(Resource, Default)]
pub struct ActiveCastSpell(Option<ActiveCastSpellItem>);

const CAST_SPELL_DELAY: u64 = 50;

struct ActiveCastSpellItem {
    key: String,
    state_scope: String,
    after_script_ast: ScriptAST,
    pointer_id: u64,
    current_pos: Vec2,
    original_size: Vec2,
    cast_pos: Vec2,
    drag_radius: f32,
    initial_swipe_done: Arc<AtomicBool>,
    // for mouse cast spell
    mouse_flag: bool,
    center_pos: Vec2,
    cast_radius: f32,
    horizontal_scale_factor: f32,
    vertical_scale_factor: f32,
    cast_no_direction: bool,
    // for pad cast spell
    pad_action: Option<MappingAction>,
    last_state: Vec2,
    block_direction_pad: bool,
    // randomization
    enable_randomization: bool,
    random_anchor: Vec2,
    random_offset: Vec2,
    current_jitter: Vec2,
    next_jitter_at: Instant,
    move_gen: Arc<AtomicU64>,
}

impl ActiveCastSpellItem {
    fn new_mouse_item(
        key: String,
        state_scope: String,
        after_script_ast: ScriptAST,
        pointer_id: u64,
        current_pos: Vec2,
        original_size: Vec2,
        cast_pos: Vec2,
        drag_radius: f32,
        initial_swipe_done: Arc<AtomicBool>,
        center_pos: Vec2,
        cast_radius: f32,
        horizontal_scale_factor: f32,
        vertical_scale_factor: f32,
        cast_no_direction: bool,
    ) -> Self {
        Self {
            mouse_flag: true,
            key,
            state_scope,
            after_script_ast,
            pointer_id,
            current_pos,
            original_size,
            cast_pos,
            drag_radius,
            initial_swipe_done,
            center_pos,
            cast_radius,
            horizontal_scale_factor,
            vertical_scale_factor,
            cast_no_direction,
            pad_action: None,
            last_state: Vec2::ZERO,
            block_direction_pad: false,
            enable_randomization: false,
            random_anchor: Vec2::ZERO,
            random_offset: Vec2::ZERO,
            current_jitter: Vec2::ZERO,
            next_jitter_at: Instant::now(),
            move_gen: Arc::new(AtomicU64::new(0)),
        }
    }

    fn new_pad_item(
        key: String,
        state_scope: String,
        after_script_ast: ScriptAST,
        pointer_id: u64,
        current_pos: Vec2,
        original_size: Vec2,
        cast_pos: Vec2,
        drag_radius: f32,
        initial_swipe_done: Arc<AtomicBool>,
        block_direction_pad: bool,
        pad_action: MappingAction,
        enable_randomization: bool,
        random_anchor: Vec2,
        random_offset: Vec2,
    ) -> Self {
        Self {
            mouse_flag: false,
            key,
            state_scope,
            after_script_ast,
            pointer_id,
            current_pos,
            original_size,
            cast_pos,
            drag_radius,
            initial_swipe_done,
            center_pos: Vec2::ZERO,
            cast_radius: 0.,
            horizontal_scale_factor: 0.,
            vertical_scale_factor: 0.,
            cast_no_direction: false,
            pad_action: Some(pad_action),
            last_state: Vec2::ZERO,
            block_direction_pad,
            enable_randomization,
            random_anchor,
            random_offset,
            current_jitter: Vec2::ZERO,
            next_jitter_at: Instant::now(),
            move_gen: Arc::new(AtomicU64::new(0)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MouseCastReleaseMode {
    OnPress,
    OnRelease,
    OnSecondPress,
}

#[derive(Debug, Clone)]
pub struct BindMappingMouseCastSpell {
    pub id: String,
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub center: Position,
    pub horizontal_scale_factor: f32,
    pub vertical_scale_factor: f32,
    pub drag_radius: f32,
    pub cast_radius: f32,
    pub release_mode: MouseCastReleaseMode,
    pub cast_no_direction: bool,
    pub initial_duration: u64,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
    pub random_offset_x: f32,
    pub random_offset_y: f32,
    pub script_hooks: BindMappingScriptHooks,
}

impl From<MappingMouseCastSpell> for BindMappingMouseCastSpell {
    fn from(value: MappingMouseCastSpell) -> Self {
        Self {
            id: value.id,
            note: value.note,
            pointer_id: value.pointer_id,
            position: value.position,
            center: value.center,
            horizontal_scale_factor: value.horizontal_scale_factor,
            vertical_scale_factor: value.vertical_scale_factor,
            drag_radius: value.drag_radius,
            cast_radius: value.cast_radius,
            release_mode: value.release_mode,
            cast_no_direction: value.cast_no_direction,
            initial_duration: value.initial_duration,
            bind: value.bind.clone(),
            input_binding: ContinuousBinding::hold(value.bind).0,
            random_offset_x: value.random_offset_x,
            random_offset_y: value.random_offset_y,
            script_hooks: value.script_hooks.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingMouseCastSpell {
    #[serde(default = "crate::mask::mapping::config::default_mapping_id")]
    pub id: String,
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub center: Position,
    #[serde(serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp")]
    pub horizontal_scale_factor: f32,
    #[serde(serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp")]
    pub vertical_scale_factor: f32,
    #[serde(serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp")]
    pub drag_radius: f32,
    #[serde(serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp")]
    pub cast_radius: f32,
    pub release_mode: MouseCastReleaseMode,
    pub cast_no_direction: bool,
    #[serde(default)]
    pub initial_duration: u64,
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

impl ValidateMappingConfig for MappingMouseCastSpell {
    fn validate(&self) -> Result<(), String> {
        self.script_hooks.validate()
    }
}

#[derive(Resource, Default)]
pub struct MouseCastSpellLifecycleState(MappingLifecycleState<CastSpellReleaseContext>);

#[derive(Resource, Default)]
pub struct PadCastSpellLifecycleState(MappingLifecycleState<CastSpellReleaseContext>);

#[derive(Clone)]
struct CastSpellReleaseContext {
    cursor_pos: Vec2,
    mask_size: Vec2,
    raw_input_flag: bool,
    fps_mode_flag: bool,
}

fn mouse_cast_capture_owner(action: &str) -> String {
    format!("MouseCastSpell:{action}")
}

fn mouse_cast_should_capture(
    mapping: &BindMappingMouseCastSpell,
    cursor_state: CursorState,
) -> bool {
    cursor_state != CursorState::Fps
        && !mapping.cast_no_direction
        && !matches!(mapping.release_mode, MouseCastReleaseMode::OnPress)
}

fn mouse_cast_has_before_hook(mapping: &BindMappingMouseCastSpell) -> bool {
    !mapping.script_hooks.before_script_ast.empty
}

fn pad_cast_has_before_hook(mapping: &BindMappingPadCastSpell) -> bool {
    !mapping.script_hooks.before_script_ast.empty
}

fn make_active_cast_after_context(
    cs_tx: &broadcast::Sender<crate::scrcpy::control_msg::ScrcpyControlMsg>,
    script_command_tx: &ScriptRuntimeCommandSender,
    shared_state: &ScriptSharedState,
    cast: &ActiveCastSpellItem,
    cursor_pos: Vec2,
    mask_size: Vec2,
    raw_input_flag: bool,
    fps_mode_flag: bool,
) -> MappingExecutionContext {
    MappingExecutionContext {
        cs_tx: cs_tx.clone(),
        script_command_tx: script_command_tx.0.clone(),
        shared_state: shared_state.clone(),
        state_scope: cast.state_scope.clone(),
        original_size: cast.original_size,
        cursor_pos,
        mask_size,
        raw_input_flag,
        fps_mode_flag,
    }
}

async fn run_active_cast_after_hook(cast: ActiveCastSpellItem, exec_ctx: MappingExecutionContext) {
    if cast.after_script_ast.empty {
        return;
    }
    if let Err(e) = run_script_hook(&cast.after_script_ast, &exec_ctx).await {
        log::error!("[CastSpell] script hook runtime error: {:?}", e);
    }
}

fn spawn_active_cast_after_hook(
    runtime: &TokioTasksRuntime,
    cast: ActiveCastSpellItem,
    exec_ctx: MappingExecutionContext,
) {
    runtime.spawn_background_task(move |_task_ctx| async move {
        run_active_cast_after_hook(cast, exec_ctx).await;
    });
}

fn take_active_cast_for_release(
    cs_tx: &broadcast::Sender<crate::scrcpy::control_msg::ScrcpyControlMsg>,
    mask_size: Vec2,
    active_cast: &mut ActiveCastSpell,
    block_direction_pad: &mut BlockDirectionPad,
) -> Option<ActiveCastSpellItem> {
    let cast = active_cast.0.take()?;
    ControlMsgHelper::send_touch(
        cs_tx,
        MotionEventAction::Up,
        cast.pointer_id,
        mask_size,
        cast.current_pos,
    );
    if cast.block_direction_pad {
        block_direction_pad.0 = false;
    }
    Some(cast)
}

fn release_active_cast_and_spawn_after(
    cs_tx: &broadcast::Sender<crate::scrcpy::control_msg::ScrcpyControlMsg>,
    runtime: &TokioTasksRuntime,
    active_cast: &mut ActiveCastSpell,
    block_direction_pad: &mut BlockDirectionPad,
    script_command_tx: &ScriptRuntimeCommandSender,
    shared_state: &ScriptSharedState,
    cursor_pos: Vec2,
    mask_size: Vec2,
    raw_input_flag: bool,
    fps_mode_flag: bool,
) -> Option<String> {
    let cast = take_active_cast_for_release(cs_tx, mask_size, active_cast, block_direction_pad)?;
    let released_key = cast.key.clone();
    let exec_ctx = make_active_cast_after_context(
        cs_tx,
        script_command_tx,
        shared_state,
        &cast,
        cursor_pos,
        mask_size,
        raw_input_flag,
        fps_mode_flag,
    );
    spawn_active_cast_after_hook(runtime, cast, exec_ctx);
    Some(released_key)
}

fn cal_mouse_cast_spell_current_pos(
    cursor_pos: Vec2,
    mut center_pos: Vec2,
    mut cast_pos: Vec2,
    mut cast_radius: f32,
    mut drag_radius: f32,
    mask_size: Vec2,
    original_size: Vec2,
    horizontal_scale_factor: f32,
    vertical_scale_factor: f32,
) -> Vec2 {
    // convert to mask scale
    center_pos = center_pos / original_size * mask_size;
    cast_pos = cast_pos / original_size * mask_size;
    cast_radius = cast_radius / original_size.y * mask_size.y;
    drag_radius = drag_radius / original_size.y * mask_size.y;

    let mut delta = cursor_pos - center_pos;
    // set the larger ratio to 1
    let scale = if horizontal_scale_factor > vertical_scale_factor {
        let r = vertical_scale_factor / horizontal_scale_factor;
        cast_radius *= r;
        Vec2::new(1.0, r)
    } else {
        let r = horizontal_scale_factor / vertical_scale_factor;
        cast_radius *= r;
        Vec2::new(r, 1.0)
    };
    delta *= scale;

    if delta.length_squared() > cast_radius * cast_radius {
        // outside of cast range
        delta = delta.normalize() * drag_radius;
    } else {
        // inside of cast range
        delta = delta / cast_radius * drag_radius;
    }

    cast_pos + delta
}

fn start_mouse_cast_after_before(
    cs_tx: &broadcast::Sender<crate::scrcpy::control_msg::ScrcpyControlMsg>,
    runtime: &TokioTasksRuntime,
    active_cast: &mut ActiveCastSpell,
    block_direction_pad: &mut BlockDirectionPad,
    mut normal_cursor_capture: Option<&mut NormalCursorCapture>,
    script_command_tx: &ScriptRuntimeCommandSender,
    shared_state: &ScriptSharedState,
    action: String,
    mapping: &BindMappingMouseCastSpell,
    original_size: Vec2,
    mask_size: Vec2,
    cursor_pos: Vec2,
    raw_input_flag: bool,
    fps_mode_flag: bool,
) -> bool {
    let released_key = release_active_cast_and_spawn_after(
        cs_tx,
        runtime,
        active_cast,
        block_direction_pad,
        script_command_tx,
        shared_state,
        cursor_pos,
        mask_size,
        raw_input_flag,
        fps_mode_flag,
    );
    if let Some(released_key) = released_key.as_deref()
        && let Some(capture) = normal_cursor_capture.as_mut()
    {
        capture.release(&mouse_cast_capture_owner(released_key));
    }
    if released_key.as_deref() == Some(action.as_str()) {
        return false;
    }

    let pointer_id = mapping.pointer_id;
    let original_pos: Vec2 = mapping.position.into();
    let original_pos = random_offset_vec2(
        original_pos,
        Vec2::new(mapping.random_offset_x, mapping.random_offset_y),
    );
    let center_pos: Vec2 = mapping.center.into();
    let release_mode = mapping.release_mode.clone();
    let cast_no_direction = mapping.cast_no_direction;
    let cast_radius = mapping.cast_radius;
    let drag_radius = mapping.drag_radius;
    let horizontal_scale_factor = mapping.horizontal_scale_factor;
    let vertical_scale_factor = mapping.vertical_scale_factor;
    let current_pos = original_pos / original_size * mask_size;

    ControlMsgHelper::send_touch(
        cs_tx,
        MotionEventAction::Down,
        pointer_id,
        mask_size,
        current_pos,
    );

    let target_pos = if !cast_no_direction {
        cal_mouse_cast_spell_current_pos(
            cursor_pos,
            center_pos,
            original_pos,
            cast_radius,
            drag_radius,
            mask_size,
            original_size,
            horizontal_scale_factor,
            vertical_scale_factor,
        )
    } else {
        current_pos
    };

    let initial_swipe_done = spawn_initial_swipe(
        runtime,
        cs_tx,
        pointer_id,
        mask_size,
        current_pos,
        target_pos,
        mapping.initial_duration,
        DEFAULT_SWIPE_DURATION,
        SingleSwipeStrategy::ArcWithEaseOut,
    );

    if matches!(release_mode, MouseCastReleaseMode::OnPress) {
        let cs_tx = cs_tx.clone();
        let after_script_ast = mapping.script_hooks.after_script_ast.clone();
        let exec_ctx = MappingExecutionContext {
            cs_tx: cs_tx.clone(),
            script_command_tx: script_command_tx.0.clone(),
            shared_state: shared_state.clone(),
            state_scope: mapping.id.clone(),
            original_size,
            cursor_pos,
            mask_size,
            raw_input_flag,
            fps_mode_flag,
        };
        runtime.spawn_background_task(move |_ctx| async move {
            while !initial_swipe_done.load(Ordering::Relaxed) {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            ControlMsgHelper::send_touch(
                &cs_tx,
                MotionEventAction::Up,
                pointer_id,
                mask_size,
                target_pos,
            );
            if !after_script_ast.empty {
                if let Err(e) = run_script_hook(&after_script_ast, &exec_ctx).await {
                    log::error!("[MouseCastSpell] script hook runtime error: {:?}", e);
                }
            }
        });
    } else {
        active_cast.0 = Some(ActiveCastSpellItem::new_mouse_item(
            action,
            mapping.id.clone(),
            mapping.script_hooks.after_script_ast.clone(),
            pointer_id,
            target_pos,
            original_size,
            original_pos,
            mapping.drag_radius,
            initial_swipe_done,
            center_pos,
            mapping.cast_radius,
            mapping.horizontal_scale_factor,
            mapping.vertical_scale_factor,
            mapping.cast_no_direction,
        ));
    }

    true
}

pub fn handle_mouse_cast_spell_trigger(
    cs_tx_res: Res<ChannelSenderCS>,
    mask_size: Res<MaskSize>,
    cursor_pos: Res<CursorPosition>,
    mut active_cast: ResMut<ActiveCastSpell>,
) {
    if let Some(active_cast) = active_cast.0.as_mut() {
        if active_cast.cast_no_direction
            || !active_cast.mouse_flag
            || !active_cast.initial_swipe_done.load(Ordering::Relaxed)
        {
            return;
        }

        let new_pos = cal_mouse_cast_spell_current_pos(
            cursor_pos.0,
            active_cast.center_pos,
            active_cast.cast_pos,
            active_cast.cast_radius,
            active_cast.drag_radius,
            mask_size.0,
            active_cast.original_size,
            active_cast.horizontal_scale_factor,
            active_cast.vertical_scale_factor,
        );
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Move,
            active_cast.pointer_id,
            mask_size.0,
            new_pos,
        );
        active_cast.current_pos = new_pos;
    }
}

pub fn handle_mouse_cast_spell(
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
    mut active_cast: ResMut<ActiveCastSpell>,
    mut block_direction_pad: ResMut<BlockDirectionPad>,
    mut lifecycle_state: ResMut<MouseCastSpellLifecycleState>,
    mut normal_cursor_capture: ResMut<NormalCursorCapture>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("MouseCastSpell") {
                let mapping = mapping.as_ref_mousecastspell();
                if ineffable.just_activated(action.ineff_continuous()) {
                    let cur_cursor_pos = cursor_pos.0;
                    let cur_mask_size = mask_size.0;
                    let capture_owner = mouse_cast_capture_owner(action.as_ref());
                    let capture_requested = mouse_cast_should_capture(mapping, *cursor_state.get());
                    if capture_requested {
                        normal_cursor_capture.request(capture_owner.clone());
                    }
                    if mouse_cast_has_before_hook(mapping) {
                        let action = action.to_string();
                        let version = lifecycle_state.0.begin_start(&action);
                        let mapping = mapping.clone();
                        let before_script_ast = mapping.script_hooks.before_script_ast.clone();
                        let exec_ctx = make_mapping_execution_context(
                            &cs_tx_res,
                            &script_command_tx,
                            &shared_state,
                            mapping.id.clone(),
                            active_mapping.original_size.into(),
                            cur_cursor_pos,
                            cur_mask_size,
                            mapping_state.get() == &MappingState::RawInput,
                            cursor_state.get() == &CursorState::Fps,
                        );
                        let cs_tx = cs_tx_res.0.clone();
                        let script_command_tx = script_command_tx.clone();
                        let shared_state = shared_state.clone();
                        let raw_input_flag = mapping_state.get() == &MappingState::RawInput;
                        let fps_mode_flag = cursor_state.get() == &CursorState::Fps;
                        let error_capture_owner = capture_owner.clone();
                        let start_capture_owner = capture_owner.clone();
                        runtime.spawn_background_task(move |mut task_ctx| async move {
                            if let Err(e) = run_script_hook(&before_script_ast, &exec_ctx).await {
                                task_ctx
                                    .run_on_main_thread({
                                        let action = action.clone();
                                        move |main_ctx| {
                                            main_ctx
                                                .world
                                                .resource_mut::<MouseCastSpellLifecycleState>()
                                                .0
                                                .cancel_start(&action, version);
                                            if capture_requested {
                                                main_ctx
                                                    .world
                                                    .resource_mut::<NormalCursorCapture>()
                                                    .release(&error_capture_owner);
                                            }
                                        }
                                    })
                                    .await;
                                log::error!("[MouseCastSpell] script hook runtime error: {:?}", e);
                                return;
                            }

                            task_ctx
                                .run_on_main_thread(move |main_ctx| {
                                    let start = main_ctx
                                        .world
                                        .resource_mut::<MouseCastSpellLifecycleState>()
                                        .0
                                        .finish_start(&action, version);
                                    let pending_release = match start {
                                        MappingLifecycleStart::Stale => return,
                                        MappingLifecycleStart::Ready { pending_release } => {
                                            pending_release
                                        }
                                    };

                                    let start_mask_size = pending_release
                                        .as_ref()
                                        .map(|release| release.mask_size)
                                        .unwrap_or_else(|| main_ctx.world.resource::<MaskSize>().0);
                                    let start_cursor_pos = pending_release
                                        .as_ref()
                                        .map(|release| release.cursor_pos)
                                        .unwrap_or_else(|| {
                                            main_ctx.world.resource::<CursorPosition>().0
                                        });
                                    let active_mapping = main_ctx
                                        .world
                                        .resource::<ActiveMappingConfig>()
                                        .0
                                        .as_ref()
                                        .cloned()
                                        .expect("active mapping missing");
                                    let mut active_cast = main_ctx
                                        .world
                                        .remove_resource::<ActiveCastSpell>()
                                        .expect("ActiveCastSpell resource missing");
                                    let mut block_direction_pad = main_ctx
                                        .world
                                        .remove_resource::<BlockDirectionPad>()
                                        .expect("BlockDirectionPad resource missing");
                                    let mut normal_cursor_capture = main_ctx
                                        .world
                                        .remove_resource::<NormalCursorCapture>()
                                        .expect("NormalCursorCapture resource missing");
                                    {
                                        let runtime =
                                            main_ctx.world.resource::<TokioTasksRuntime>();
                                        let started = start_mouse_cast_after_before(
                                            &cs_tx,
                                            runtime,
                                            &mut active_cast,
                                            &mut block_direction_pad,
                                            Some(&mut normal_cursor_capture),
                                            &script_command_tx,
                                            &shared_state,
                                            action.clone(),
                                            &mapping,
                                            active_mapping.original_size.into(),
                                            start_mask_size,
                                            start_cursor_pos,
                                            raw_input_flag,
                                            fps_mode_flag,
                                        );
                                        if !started && capture_requested {
                                            normal_cursor_capture.release(&start_capture_owner);
                                        }
                                        if let Some(release) = pending_release {
                                            let released_key = release_active_cast_and_spawn_after(
                                                &cs_tx,
                                                runtime,
                                                &mut active_cast,
                                                &mut block_direction_pad,
                                                &script_command_tx,
                                                &shared_state,
                                                release.cursor_pos,
                                                release.mask_size,
                                                release.raw_input_flag,
                                                release.fps_mode_flag,
                                            );
                                            if let Some(released_key) = released_key {
                                                normal_cursor_capture.release(
                                                    &mouse_cast_capture_owner(&released_key),
                                                );
                                            }
                                        }
                                    }
                                    main_ctx.world.insert_resource(active_cast);
                                    main_ctx.world.insert_resource(block_direction_pad);
                                    main_ctx.world.insert_resource(normal_cursor_capture);
                                })
                                .await;
                        });
                    } else {
                        let started = start_mouse_cast_after_before(
                            &cs_tx_res.0,
                            &runtime,
                            &mut active_cast,
                            &mut block_direction_pad,
                            Some(&mut normal_cursor_capture),
                            &script_command_tx,
                            &shared_state,
                            action.to_string(),
                            mapping,
                            active_mapping.original_size.into(),
                            cur_mask_size,
                            cur_cursor_pos,
                            mapping_state.get() == &MappingState::RawInput,
                            cursor_state.get() == &CursorState::Fps,
                        );
                        if !started && capture_requested {
                            normal_cursor_capture.release(&capture_owner);
                        }
                    }
                } else if ineffable.just_deactivated(action.ineff_continuous()) {
                    if let MouseCastReleaseMode::OnRelease = mapping.release_mode {
                        let released = active_cast
                            .0
                            .as_ref()
                            .is_some_and(|cast| cast.key == action.as_ref());
                        if released {
                            release_active_cast_and_spawn_after(
                                &cs_tx_res.0,
                                &runtime,
                                &mut active_cast,
                                &mut block_direction_pad,
                                &script_command_tx,
                                &shared_state,
                                cursor_pos.0,
                                mask_size.0,
                                mapping_state.get() == &MappingState::RawInput,
                                cursor_state.get() == &CursorState::Fps,
                            );
                            lifecycle_state.0.clear_pending(action.as_ref());
                            normal_cursor_capture
                                .release(&mouse_cast_capture_owner(action.as_ref()));
                        } else if mouse_cast_has_before_hook(mapping) {
                            lifecycle_state.0.record_early_release(
                                action.as_ref(),
                                CastSpellReleaseContext {
                                    cursor_pos: cursor_pos.0,
                                    mask_size: mask_size.0,
                                    raw_input_flag: mapping_state.get() == &MappingState::RawInput,
                                    fps_mode_flag: cursor_state.get() == &CursorState::Fps,
                                },
                            );
                            normal_cursor_capture
                                .release(&mouse_cast_capture_owner(action.as_ref()));
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_mouse_cast_spell_focus_lost(
    window: Single<&Window>,
    mut was_focused: Local<bool>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    script_command_tx: Res<ScriptRuntimeCommandSender>,
    shared_state: Res<ScriptSharedState>,
    cursor_pos: Res<CursorPosition>,
    mask_size: Res<MaskSize>,
    mapping_state: Res<State<MappingState>>,
    cursor_state: Res<State<CursorState>>,
    runtime: ResMut<TokioTasksRuntime>,
    mut active_cast: ResMut<ActiveCastSpell>,
    mut block_direction_pad: ResMut<BlockDirectionPad>,
    mut lifecycle_state: ResMut<MouseCastSpellLifecycleState>,
    mut normal_cursor_capture: ResMut<NormalCursorCapture>,
) {
    let lost_focus = *was_focused && !window.focused;
    *was_focused = window.focused;
    if !lost_focus {
        return;
    }

    if active_cast.0.as_ref().is_some_and(|cast| cast.mouse_flag)
        && let Some(released_key) = release_active_cast_and_spawn_after(
            &cs_tx_res.0,
            &runtime,
            &mut active_cast,
            &mut block_direction_pad,
            &script_command_tx,
            &shared_state,
            cursor_pos.0,
            mask_size.0,
            mapping_state.get() == &MappingState::RawInput,
            cursor_state.get() == &CursorState::Fps,
        )
    {
        normal_cursor_capture.release(&mouse_cast_capture_owner(&released_key));
        lifecycle_state.0.clear_pending(&released_key);
    }

    if let Some(active_mapping) = &active_mapping.0 {
        for action in active_mapping.mappings.keys() {
            if action.as_ref().starts_with("MouseCastSpell") {
                lifecycle_state.0.cancel_pending(action.as_ref());
                normal_cursor_capture.release(&mouse_cast_capture_owner(action.as_ref()));
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PadCastReleaseMode {
    OnRelease,
    OnSecondPress,
}

#[derive(Debug, Clone)]
pub struct BindMappingPadCastSpell {
    pub id: String,
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub release_mode: PadCastReleaseMode,
    pub drag_radius: f32,
    pub block_direction_pad: bool,
    pub pad_action: MappingAction,
    pub pad_bind: DirectionBinding,
    pub pad_input_binding: InputBinding,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
    pub random_offset_x: f32,
    pub random_offset_y: f32,
    pub enable_randomization: bool,
    pub script_hooks: BindMappingScriptHooks,
}

impl From<MappingPadCastSpell> for BindMappingPadCastSpell {
    fn from(value: MappingPadCastSpell) -> Self {
        Self {
            id: value.id,
            note: value.note,
            pointer_id: value.pointer_id,
            position: value.position,
            release_mode: value.release_mode,
            drag_radius: value.drag_radius,
            block_direction_pad: value.block_direction_pad,
            pad_action: MappingAction::PadCastDirection1, // temp value
            pad_bind: value.pad_bind.clone(),
            pad_input_binding: value.pad_bind.into(),
            bind: value.bind.clone(),
            input_binding: ContinuousBinding::hold(value.bind).0,
            random_offset_x: value.random_offset_x,
            random_offset_y: value.random_offset_y,
            enable_randomization: value.enable_randomization,
            script_hooks: value.script_hooks.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingPadCastSpell {
    #[serde(default = "crate::mask::mapping::config::default_mapping_id")]
    pub id: String,
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub release_mode: PadCastReleaseMode,
    #[serde(serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp")]
    pub drag_radius: f32,
    pub block_direction_pad: bool,
    pub pad_bind: DirectionBinding,
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
    pub enable_randomization: bool,
    #[serde(default)]
    pub script_hooks: MappingScriptHooks,
}

impl ValidateMappingConfig for MappingPadCastSpell {
    fn validate(&self) -> Result<(), String> {
        self.script_hooks.validate()
    }
}

fn scale_direction_2d_state(d_state: Vec2, drag_radius: f32) -> Vec2 {
    if d_state.x == 0.0 && d_state.y == 0.0 {
        return d_state;
    }

    let scaled = d_state * drag_radius;
    if scaled.length() > drag_radius {
        scaled.normalize() * drag_radius
    } else {
        scaled
    }
}

fn release_direction_pads_and_spawn_after(
    active_mapping: &BindMappingConfig,
    cs_tx: &broadcast::Sender<crate::scrcpy::control_msg::ScrcpyControlMsg>,
    runtime: &TokioTasksRuntime,
    script_command_tx: &ScriptRuntimeCommandSender,
    shared_state: &ScriptSharedState,
    direction_pad_map: &mut DirectionPadMap,
    cursor_pos: Vec2,
    mask_size: Vec2,
    raw_input_flag: bool,
    fps_mode_flag: bool,
) {
    let original_size: Vec2 = active_mapping.original_size.into();
    for (key, item) in direction_pad_map.0.drain() {
        let last_pos = if item.enable_randomization {
            item.random_anchor + item.last_state + item.current_jitter
        } else {
            item.original_pos + item.last_state
        };
        ControlMsgHelper::send_touch(
            cs_tx,
            MotionEventAction::Up,
            item.pointer_id,
            item.original_size,
            last_pos,
        );

        let Some((_, BindMappingType::DirectionPad(mapping))) = active_mapping
            .mappings
            .iter()
            .find(|(action, _)| action.as_ref() == key)
        else {
            continue;
        };

        if mapping.script_hooks.after_script_ast.empty {
            continue;
        }

        let after_script_ast = mapping.script_hooks.after_script_ast.clone();
        let exec_ctx = MappingExecutionContext {
            cs_tx: cs_tx.clone(),
            script_command_tx: script_command_tx.0.clone(),
            shared_state: shared_state.clone(),
            state_scope: mapping.id.clone(),
            original_size,
            cursor_pos,
            mask_size,
            raw_input_flag,
            fps_mode_flag,
        };
        runtime.spawn_background_task(move |_task_ctx| async move {
            if let Err(e) = run_script_hook(&after_script_ast, &exec_ctx).await {
                log::error!("[DirectionPad] script hook runtime error: {:?}", e);
            }
        });
    }
}

fn start_pad_cast_after_before(
    active_mapping: &BindMappingConfig,
    cs_tx: &broadcast::Sender<crate::scrcpy::control_msg::ScrcpyControlMsg>,
    runtime: &TokioTasksRuntime,
    active_cast: &mut ActiveCastSpell,
    direction_pad_map: &mut DirectionPadMap,
    block_direction_pad: &mut BlockDirectionPad,
    script_command_tx: &ScriptRuntimeCommandSender,
    shared_state: &ScriptSharedState,
    action: String,
    mapping: &BindMappingPadCastSpell,
    mask_size: Vec2,
    cursor_pos: Vec2,
    raw_input_flag: bool,
    fps_mode_flag: bool,
) -> bool {
    let released_key = release_active_cast_and_spawn_after(
        cs_tx,
        runtime,
        active_cast,
        block_direction_pad,
        script_command_tx,
        shared_state,
        cursor_pos,
        mask_size,
        raw_input_flag,
        fps_mode_flag,
    );
    if released_key.as_deref() == Some(action.as_str()) {
        return false;
    }

    if mapping.block_direction_pad {
        block_direction_pad.0 = true;
        release_direction_pads_and_spawn_after(
            active_mapping,
            cs_tx,
            runtime,
            script_command_tx,
            shared_state,
            direction_pad_map,
            cursor_pos,
            mask_size,
            raw_input_flag,
            fps_mode_flag,
        );
    }

    let original_size: Vec2 = active_mapping.original_size.into();
    let pointer_id = mapping.pointer_id;
    let original_pos: Vec2 = mapping.position.into();
    let original_pos = random_offset_vec2(
        original_pos,
        Vec2::new(mapping.random_offset_x, mapping.random_offset_y),
    );
    let current_pos = original_pos / original_size * mask_size;
    let (random_anchor, random_offset) = if mapping.enable_randomization {
        let offset = anchor_random_offset(mapping.drag_radius, mapping.drag_radius);
        let anchor = random_offset_vec2(original_pos, offset);
        (anchor, offset)
    } else {
        (Vec2::ZERO, Vec2::ZERO)
    };

    ControlMsgHelper::send_touch(
        cs_tx,
        MotionEventAction::Down,
        pointer_id,
        mask_size,
        current_pos,
    );

    let slide_start = if mapping.enable_randomization {
        random_anchor / original_size * mask_size
    } else {
        current_pos
    };
    let strategy = if mapping.enable_randomization {
        SingleSwipeStrategy::ArcWithEaseOut
    } else {
        SingleSwipeStrategy::Linear
    };
    let initial_swipe_done = spawn_initial_swipe(
        runtime,
        cs_tx,
        pointer_id,
        mask_size,
        slide_start,
        slide_start,
        0,
        DEFAULT_SWIPE_DURATION,
        strategy,
    );

    active_cast.0 = Some(ActiveCastSpellItem::new_pad_item(
        action,
        mapping.id.clone(),
        mapping.script_hooks.after_script_ast.clone(),
        pointer_id,
        current_pos,
        original_size,
        original_pos,
        mapping.drag_radius,
        initial_swipe_done,
        mapping.block_direction_pad,
        mapping.pad_action.clone(),
        mapping.enable_randomization,
        random_anchor,
        random_offset,
    ));

    true
}

pub fn handle_pad_cast_spell_trigger(
    ineffable: Res<Ineffable>,
    cs_tx_res: Res<ChannelSenderCS>,
    runtime: ResMut<TokioTasksRuntime>,
    mut active_cast: ResMut<ActiveCastSpell>,
) {
    if let Some(active_cast) = active_cast.0.as_mut() {
        if active_cast.mouse_flag || !active_cast.initial_swipe_done.load(Ordering::Relaxed) {
            return;
        }

        let state = scale_direction_2d_state(
            ineffable.direction_2d(active_cast.pad_action.as_ref().unwrap().ineff_dual_axis()),
            active_cast.drag_radius,
        );

        if state != active_cast.last_state {
            let old_state = active_cast.last_state;

            if active_cast.enable_randomization {
                handle_direction_move_randomized(
                    old_state,
                    state,
                    active_cast.random_anchor,
                    &mut active_cast.current_jitter,
                    &mut active_cast.next_jitter_at,
                    &active_cast.move_gen,
                    active_cast.pointer_id,
                    active_cast.original_size,
                    &cs_tx_res.0,
                    &runtime,
                    SingleSwipeStrategy::ArcWithEaseInOut,
                );
                active_cast.last_state = state;
            } else {
                ControlMsgHelper::send_touch(
                    &cs_tx_res.0,
                    MotionEventAction::Move,
                    active_cast.pointer_id,
                    active_cast.original_size,
                    active_cast.cast_pos + state,
                );
                active_cast.last_state = state;
            }
        } else if active_cast.enable_randomization && Instant::now() > active_cast.next_jitter_at {
            handle_direction_jitter(
                state,
                active_cast.random_anchor,
                &mut active_cast.current_jitter,
                &mut active_cast.next_jitter_at,
                active_cast.random_offset,
                active_cast.pointer_id,
                active_cast.original_size,
                &cs_tx_res.0,
            );
        }
    }
}

pub fn handle_pad_cast_spell(
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
    mut active_cast: ResMut<ActiveCastSpell>,
    mut direction_pad_map: ResMut<DirectionPadMap>,
    mut block_direction_pad: ResMut<BlockDirectionPad>,
    mut lifecycle_state: ResMut<PadCastSpellLifecycleState>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("PadCastSpell") {
                let mapping = mapping.as_ref_padcastspell();
                if ineffable.just_activated(action.ineff_continuous()) {
                    if pad_cast_has_before_hook(mapping) {
                        let action = action.to_string();
                        let version = lifecycle_state.0.begin_start(&action);
                        let mapping = mapping.clone();
                        let active_mapping = active_mapping.clone();
                        let before_script_ast = mapping.script_hooks.before_script_ast.clone();
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
                        let cs_tx = cs_tx_res.0.clone();
                        let script_command_tx = script_command_tx.clone();
                        let shared_state = shared_state.clone();
                        let cursor_pos = cursor_pos.0;
                        let mask_size = mask_size.0;
                        let raw_input_flag = mapping_state.get() == &MappingState::RawInput;
                        let fps_mode_flag = cursor_state.get() == &CursorState::Fps;
                        runtime.spawn_background_task(move |mut task_ctx| async move {
                            if let Err(e) = run_script_hook(&before_script_ast, &exec_ctx).await {
                                task_ctx
                                    .run_on_main_thread({
                                        let action = action.clone();
                                        move |main_ctx| {
                                            main_ctx
                                                .world
                                                .resource_mut::<PadCastSpellLifecycleState>()
                                                .0
                                                .cancel_start(&action, version);
                                        }
                                    })
                                    .await;
                                log::error!("[PadCastSpell] script hook runtime error: {:?}", e);
                                return;
                            }

                            task_ctx
                                .run_on_main_thread(move |main_ctx| {
                                    let start = main_ctx
                                        .world
                                        .resource_mut::<PadCastSpellLifecycleState>()
                                        .0
                                        .finish_start(&action, version);
                                    let pending_release = match start {
                                        MappingLifecycleStart::Stale => return,
                                        MappingLifecycleStart::Ready { pending_release } => {
                                            pending_release
                                        }
                                    };

                                    let mut active_cast = main_ctx
                                        .world
                                        .remove_resource::<ActiveCastSpell>()
                                        .expect("ActiveCastSpell resource missing");
                                    let mut direction_pad_map = main_ctx
                                        .world
                                        .remove_resource::<DirectionPadMap>()
                                        .expect("DirectionPadMap resource missing");
                                    let mut block_direction_pad = main_ctx
                                        .world
                                        .remove_resource::<BlockDirectionPad>()
                                        .expect("BlockDirectionPad resource missing");
                                    {
                                        let runtime =
                                            main_ctx.world.resource::<TokioTasksRuntime>();
                                        start_pad_cast_after_before(
                                            &active_mapping,
                                            &cs_tx,
                                            runtime,
                                            &mut active_cast,
                                            &mut direction_pad_map,
                                            &mut block_direction_pad,
                                            &script_command_tx,
                                            &shared_state,
                                            action.clone(),
                                            &mapping,
                                            mask_size,
                                            cursor_pos,
                                            raw_input_flag,
                                            fps_mode_flag,
                                        );
                                        if let Some(release) = pending_release {
                                            release_active_cast_and_spawn_after(
                                                &cs_tx,
                                                runtime,
                                                &mut active_cast,
                                                &mut block_direction_pad,
                                                &script_command_tx,
                                                &shared_state,
                                                release.cursor_pos,
                                                release.mask_size,
                                                release.raw_input_flag,
                                                release.fps_mode_flag,
                                            );
                                        }
                                    }
                                    main_ctx.world.insert_resource(active_cast);
                                    main_ctx.world.insert_resource(direction_pad_map);
                                    main_ctx.world.insert_resource(block_direction_pad);
                                })
                                .await;
                        });
                    } else {
                        start_pad_cast_after_before(
                            active_mapping,
                            &cs_tx_res.0,
                            &runtime,
                            &mut active_cast,
                            &mut direction_pad_map,
                            &mut block_direction_pad,
                            &script_command_tx,
                            &shared_state,
                            action.to_string(),
                            mapping,
                            mask_size.0,
                            cursor_pos.0,
                            mapping_state.get() == &MappingState::RawInput,
                            cursor_state.get() == &CursorState::Fps,
                        );
                    }
                } else if ineffable.just_deactivated(action.ineff_continuous()) {
                    if let PadCastReleaseMode::OnRelease = mapping.release_mode {
                        let released = active_cast
                            .0
                            .as_ref()
                            .is_some_and(|cast| cast.key == action.as_ref());
                        if released {
                            release_active_cast_and_spawn_after(
                                &cs_tx_res.0,
                                &runtime,
                                &mut active_cast,
                                &mut block_direction_pad,
                                &script_command_tx,
                                &shared_state,
                                cursor_pos.0,
                                mask_size.0,
                                mapping_state.get() == &MappingState::RawInput,
                                cursor_state.get() == &CursorState::Fps,
                            );
                            lifecycle_state.0.clear_pending(action.as_ref());
                        } else if pad_cast_has_before_hook(mapping) {
                            lifecycle_state.0.record_early_release(
                                action.as_ref(),
                                CastSpellReleaseContext {
                                    cursor_pos: cursor_pos.0,
                                    mask_size: mask_size.0,
                                    raw_input_flag: mapping_state.get() == &MappingState::RawInput,
                                    fps_mode_flag: cursor_state.get() == &CursorState::Fps,
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BindMappingCancelCast {
    pub id: String,
    pub note: String,
    pub position: Position,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
    pub script_hooks: BindMappingScriptHooks,
}

impl From<MappingCancelCast> for BindMappingCancelCast {
    fn from(value: MappingCancelCast) -> Self {
        Self {
            id: value.id,
            position: value.position,
            note: value.note,
            bind: value.bind.clone(),
            input_binding: PulseBinding::just_pressed(value.bind).0,
            script_hooks: value.script_hooks.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingCancelCast {
    #[serde(default = "crate::mask::mapping::config::default_mapping_id")]
    pub id: String,
    pub note: String,
    pub position: Position,
    pub bind: ButtonBinding,
    #[serde(default)]
    pub script_hooks: MappingScriptHooks,
}

impl ValidateMappingConfig for MappingCancelCast {
    fn validate(&self) -> Result<(), String> {
        self.script_hooks.validate()
    }
}

pub fn release_active_cast(
    cs_tx: &broadcast::Sender<crate::scrcpy::control_msg::ScrcpyControlMsg>,
    runtime: &TokioTasksRuntime,
    mask_size: Vec2,
    cursor_pos: Vec2,
    active_cast: &mut ActiveCastSpell,
    block_direction_pad: &mut BlockDirectionPad,
    script_command_tx: &ScriptRuntimeCommandSender,
    shared_state: &ScriptSharedState,
    raw_input_flag: bool,
    fps_mode_flag: bool,
) {
    release_active_cast_and_spawn_after(
        cs_tx,
        runtime,
        active_cast,
        block_direction_pad,
        script_command_tx,
        shared_state,
        cursor_pos,
        mask_size,
        raw_input_flag,
        fps_mode_flag,
    );
}

pub fn cancel_active_cast(
    cs_tx: &broadcast::Sender<crate::scrcpy::control_msg::ScrcpyControlMsg>,
    runtime: &TokioTasksRuntime,
    active_cast: &mut ActiveCastSpell,
    cancel_mapping: &BindMappingCancelCast,
    original_size: Vec2,
    mask_size: Vec2,
    script_command_tx: &ScriptRuntimeCommandSender,
    shared_state: &ScriptSharedState,
    cursor_pos: Vec2,
    raw_input_flag: bool,
    fps_mode_flag: bool,
) {
    cancel_active_cast_with_completion(
        cs_tx,
        runtime,
        active_cast,
        cancel_mapping,
        original_size,
        mask_size,
        script_command_tx,
        shared_state,
        cursor_pos,
        raw_input_flag,
        fps_mode_flag,
        None,
    );
}

pub fn cancel_active_cast_with_completion(
    cs_tx: &broadcast::Sender<crate::scrcpy::control_msg::ScrcpyControlMsg>,
    runtime: &TokioTasksRuntime,
    active_cast: &mut ActiveCastSpell,
    cancel_mapping: &BindMappingCancelCast,
    original_size: Vec2,
    mask_size: Vec2,
    script_command_tx: &ScriptRuntimeCommandSender,
    shared_state: &ScriptSharedState,
    cursor_pos: Vec2,
    raw_input_flag: bool,
    fps_mode_flag: bool,
    completion: Option<oneshot::Sender<Result<(), String>>>,
) {
    if let Some(cast) = active_cast.0.take() {
        let mut cancel_pos: Vec2 = cancel_mapping.position.into();
        let current_pos = cast.current_pos;

        cancel_pos = cancel_pos / original_size * mask_size;

        let cs_tx = cs_tx.clone();
        let pointer_id = cast.pointer_id;
        let cast_block_direction_pad = cast.block_direction_pad;
        let cast_initial_swipe_done = cast.initial_swipe_done.clone();
        let cast_after_exec_ctx = make_active_cast_after_context(
            &cs_tx,
            script_command_tx,
            shared_state,
            &cast,
            cursor_pos,
            mask_size,
            raw_input_flag,
            fps_mode_flag,
        );
        runtime.spawn_background_task(move |mut ctx| async move {
            while !cast_initial_swipe_done.load(Ordering::Relaxed) {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            let move_points = build_single_segment_swipe_intermediate_points(
                current_pos,
                cancel_pos,
                SingleSwipeStrategy::Linear,
                DEFAULT_SWIPE_DURATION,
            );
            let mut end_pos = current_pos;
            for point in move_points {
                ControlMsgHelper::send_touch(
                    &cs_tx,
                    MotionEventAction::Move,
                    pointer_id,
                    mask_size,
                    point.pos,
                );
                tokio::time::sleep(std::time::Duration::from_millis(point.wait_ms)).await;
                end_pos = point.pos;
            }

            let steps: u64 = 10;
            let step_interval = CAST_SPELL_DELAY / steps;
            for _ in 0..steps {
                end_pos.x += 5.;
                ControlMsgHelper::send_touch(
                    &cs_tx,
                    MotionEventAction::Move,
                    pointer_id,
                    mask_size,
                    end_pos,
                );
                tokio::time::sleep(std::time::Duration::from_millis(step_interval)).await;
            }

            ControlMsgHelper::send_touch(
                &cs_tx,
                MotionEventAction::Up,
                pointer_id,
                mask_size,
                cancel_pos,
            );

            if cast_block_direction_pad {
                ctx.run_on_main_thread(move |ctx| {
                    let mut block_direction_pad = ctx.world.resource_mut::<BlockDirectionPad>();
                    block_direction_pad.0 = false;
                })
                .await;
            }

            run_active_cast_after_hook(cast, cast_after_exec_ctx).await;

            if let Some(completion) = completion {
                let _ = completion.send(Ok(()));
            }
        });
    } else if let Some(completion) = completion {
        let _ = completion.send(Ok(()));
    }
}

pub fn handle_cancel_cast(
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
            if action.as_ref().starts_with("CancelCast") {
                let mapping = mapping.as_ref_cancelcast();
                if ineffable.just_pulsed(action.ineff_pulse()) {
                    let original_size: Vec2 = active_mapping.original_size.into();
                    let mapping_id = mapping.id.clone();
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
                            let (ack, rx) = oneshot::channel();
                            ctx.script_command_tx
                                .send(ScriptRuntimeCommand::CancelCast {
                                    id: mapping_id,
                                    ack,
                                })
                                .map_err(|e| {
                                    MappingExecutionError::Action(format!(
                                        "failed to send cancel_cast command: {e}"
                                    ))
                                })?;

                            rx.await
                                .map_err(|e| {
                                    MappingExecutionError::Action(format!(
                                        "failed to receive cancel_cast completion: {e}"
                                    ))
                                })?
                                .map_err(MappingExecutionError::Action)?;

                            Ok(())
                        })
                        .await;
                        if let Err(e) = result {
                            log::error!("[CancelCast] mapping execution error: {:?}", e);
                        }
                    });
                }
            }
        }
    }
}
