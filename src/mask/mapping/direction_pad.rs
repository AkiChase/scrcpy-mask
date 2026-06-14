use std::{
    collections::HashMap,
    sync::Arc,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Instant,
};

use crate::tokio_tasks::TokioTasksRuntime;
use crate::{
    mask::mapping::{
        MappingState,
        binding::{ButtonBinding, DirectionBinding, ValidateMappingConfig},
        config::ActiveMappingConfig,
        cursor::{CursorPosition, CursorState},
        executor::{
            MappingLifecycleStart, MappingLifecycleState, make_mapping_execution_context,
            run_script_hook,
        },
        script::{BindMappingScriptHooks, MappingScriptHooks},
        script_helper::{ScriptRuntimeCommandSender, ScriptSharedState},
        utils::{
            ControlMsgHelper, DEFAULT_SWIPE_DURATION, Position, SingleSwipeStrategy,
            default_jitter_offset, default_random_distance_max_scale,
            default_random_distance_min_scale, default_random_offset, handle_direction_jitter_path,
            handle_direction_move_randomized, next_jitter_deadline, random_offset_vec2,
            spawn_initial_swipe,
        },
    },
    mask::mask_command::MaskSize,
    scrcpy::constant::MotionEventAction,
    utils::ChannelSenderCS,
};
use bevy::{
    ecs::{
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    input::{ButtonInput, keyboard::KeyCode, mouse::MouseButton},
    math::Vec2,
    state::state::State,
};
use bevy_ineffable::prelude::{Ineffable, InputBinding};
use serde::{Deserialize, Serialize};

pub fn direction_pad_init(mut commands: Commands) {
    commands.insert_resource(DirectionPadMap::default());
    commands.insert_resource(BlockDirectionPad::default());
    commands.insert_resource(DirectionPadLifecycleState::default());
}

pub fn cleanup_direction_pad_on_stop(
    cs_tx_res: Res<ChannelSenderCS>,
    mut direction_pad_map: ResMut<DirectionPadMap>,
    mut block_direction_pad: ResMut<BlockDirectionPad>,
    mut lifecycle_state: ResMut<DirectionPadLifecycleState>,
) {
    for (_, item) in direction_pad_map.0.drain() {
        let last_pos = if item.enable_randomization {
            item.random_anchor + item.last_state_actual + item.current_jitter
        } else {
            item.original_pos + item.last_state
        };
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Up,
            item.pointer_id,
            item.original_size,
            last_pos,
        );
    }
    block_direction_pad.0 = false;
    lifecycle_state.0.clear_all();
}

#[derive(Debug, Clone)]
pub struct BindMappingDirectionPad {
    pub id: String,
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub initial_duration: u64,
    pub max_offset_x: f32,
    pub max_offset_y: f32,
    pub enable_randomization: bool,
    pub random_offset_x: f32,
    pub random_offset_y: f32,
    pub random_distance_min_scale: f32,
    pub random_distance_max_scale: f32,
    pub jitter_offset_x: f32,
    pub jitter_offset_y: f32,
    pub up_boost_key: Option<ButtonBinding>,
    pub up_boost_scale: f32,
    pub bind: DirectionBinding,
    pub input_binding: InputBinding,
    pub script_hooks: BindMappingScriptHooks,
}

impl From<MappingDirectionPad> for BindMappingDirectionPad {
    fn from(value: MappingDirectionPad) -> Self {
        Self {
            id: value.id,
            note: value.note,
            pointer_id: value.pointer_id,
            position: value.position,
            initial_duration: value.initial_duration,
            max_offset_x: value.max_offset_x,
            max_offset_y: value.max_offset_y,
            enable_randomization: value.enable_randomization,
            random_offset_x: value.random_offset_x,
            random_offset_y: value.random_offset_y,
            random_distance_min_scale: value.random_distance_min_scale,
            random_distance_max_scale: value.random_distance_max_scale,
            jitter_offset_x: value.jitter_offset_x,
            jitter_offset_y: value.jitter_offset_y,
            up_boost_key: value.up_boost_key,
            up_boost_scale: value.up_boost_scale,
            bind: value.bind.clone(),
            input_binding: value.bind.into(),
            script_hooks: value.script_hooks.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingDirectionPad {
    #[serde(default = "crate::mask::mapping::config::default_mapping_id")]
    pub id: String,
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub initial_duration: u64,
    #[serde(serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp")]
    pub max_offset_x: f32,
    #[serde(serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp")]
    pub max_offset_y: f32,
    #[serde(default)]
    pub enable_randomization: bool,
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
    #[serde(
        default = "default_random_distance_min_scale",
        serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp"
    )]
    pub random_distance_min_scale: f32,
    #[serde(
        default = "default_random_distance_max_scale",
        serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp"
    )]
    pub random_distance_max_scale: f32,
    #[serde(
        default = "default_jitter_offset",
        serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp"
    )]
    pub jitter_offset_x: f32,
    #[serde(
        default = "default_jitter_offset",
        serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp"
    )]
    pub jitter_offset_y: f32,
    #[serde(default)]
    pub up_boost_key: Option<ButtonBinding>,
    #[serde(
        default = "default_up_boost_scale",
        serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp"
    )]
    pub up_boost_scale: f32,
    pub bind: DirectionBinding,
    #[serde(default)]
    pub script_hooks: MappingScriptHooks,
}

fn default_up_boost_scale() -> f32 {
    1.4
}

impl ValidateMappingConfig for MappingDirectionPad {
    fn validate(&self) -> Result<(), String> {
        if self.random_offset_x < 0.0 || self.random_offset_y < 0.0 {
            return Err(
                "DirectionPad random offset must be greater than or equal to 0".to_string(),
            );
        }
        if self.jitter_offset_x < 0.0 || self.jitter_offset_y < 0.0 {
            return Err(
                "DirectionPad jitter offset must be greater than or equal to 0".to_string(),
            );
        }
        if self.random_distance_min_scale < 0.0 || self.random_distance_max_scale < 0.0 {
            return Err(
                "DirectionPad random distance scale must be greater than or equal to 0".to_string(),
            );
        }
        if self.random_distance_min_scale > self.random_distance_max_scale {
            return Err(
                "DirectionPad random distance minimum scale must not exceed maximum scale"
                    .to_string(),
            );
        }
        self.script_hooks.validate()
    }
}

#[derive(Resource, Default)]
pub struct DirectionPadMap(pub HashMap<String, DirectionPadItem>);

#[derive(Resource, Default)]
pub struct DirectionPadLifecycleState(MappingLifecycleState<DirectionPadReleaseContext>);

#[derive(Clone)]
struct DirectionPadReleaseContext {
    cursor_pos: Vec2,
    mask_size: Vec2,
    raw_input_flag: bool,
    fps_mode_flag: bool,
}

pub struct DirectionPadItem {
    pub initial_swipe_done: Arc<AtomicBool>,
    pub pointer_id: u64,
    pub original_size: Vec2,
    pub original_pos: Vec2,
    pub random_anchor: Vec2,
    pub last_state: Vec2,
    pub last_state_actual: Vec2,
    pub next_jitter_at: Instant,
    pub current_jitter: Vec2,
    pub enable_randomization: bool,
    pub jitter_offset: Vec2,
    pub move_gen: Arc<AtomicU64>,
}

fn scale_direction_2d_state(d_state: Vec2, mapping: &BindMappingDirectionPad) -> Vec2 {
    if d_state.x == 0.0 && d_state.y == 0.0 {
        return d_state;
    }

    let max_x = mapping.max_offset_x;
    let max_y = mapping.max_offset_y;

    let scaled = Vec2 {
        x: d_state.x * max_x,
        y: d_state.y * max_y,
    };

    let ellipse_norm = (scaled.x / max_x).powi(2) + (scaled.y / max_y).powi(2);

    if ellipse_norm > 1.0 {
        let norm = (d_state.x.powi(2) + d_state.y.powi(2)).sqrt();
        let unit = Vec2 {
            x: d_state.x / norm,
            y: d_state.y / norm,
        };
        Vec2 {
            x: unit.x * max_x,
            y: unit.y * max_y,
        }
    } else {
        scaled
    }
}

#[derive(Resource, Default)]
pub struct BlockDirectionPad(pub bool);

fn direction_pad_has_before_hook(mapping: &BindMappingDirectionPad) -> bool {
    !mapping.script_hooks.before_script_ast.empty
}

fn direction_pad_has_after_hook(mapping: &BindMappingDirectionPad) -> bool {
    !mapping.script_hooks.after_script_ast.empty
}

fn random_distance_scale(mapping: &BindMappingDirectionPad) -> f32 {
    let min = mapping.random_distance_min_scale;
    let max = mapping.random_distance_max_scale;
    if min == max {
        min
    } else {
        min + rand::random::<f32>() * (max - min)
    }
}

fn randomize_direction_pad_state(state: Vec2, mapping: &BindMappingDirectionPad) -> Vec2 {
    if mapping.enable_randomization {
        state * random_distance_scale(mapping)
    } else {
        state
    }
}

fn apply_direction_pad_down(
    cs_tx: &ChannelSenderCS,
    runtime: &TokioTasksRuntime,
    direction_pad_map: &mut DirectionPadMap,
    action: String,
    mapping: &BindMappingDirectionPad,
    original_size: Vec2,
    state: Vec2,
) {
    let pointer_id = mapping.pointer_id;
    let original_pos: Vec2 = mapping.position.into();

    let random_offset = Vec2::new(mapping.random_offset_x, mapping.random_offset_y);
    let jitter_offset = Vec2::new(mapping.jitter_offset_x, mapping.jitter_offset_y);
    let random_anchor = if mapping.enable_randomization {
        random_offset_vec2(original_pos, random_offset)
    } else {
        original_pos
    };
    let actual_state = randomize_direction_pad_state(state, mapping);

    ControlMsgHelper::send_touch(
        &cs_tx.0,
        MotionEventAction::Down,
        pointer_id,
        original_size,
        random_anchor,
    );

    let strategy = if mapping.enable_randomization {
        SingleSwipeStrategy::ArcWithEaseOut
    } else {
        SingleSwipeStrategy::Linear
    };
    let swipe_start = if mapping.enable_randomization {
        random_anchor
    } else {
        original_pos
    };
    let initial_swipe_done = spawn_initial_swipe(
        runtime,
        &cs_tx.0,
        pointer_id,
        original_size,
        swipe_start,
        swipe_start + actual_state,
        mapping.initial_duration,
        DEFAULT_SWIPE_DURATION,
        strategy,
    );

    direction_pad_map.0.insert(
        action,
        DirectionPadItem {
            initial_swipe_done,
            pointer_id,
            original_size,
            original_pos,
            random_anchor,
            last_state: state,
            last_state_actual: actual_state,
            next_jitter_at: next_jitter_deadline(),
            current_jitter: Vec2::ZERO,
            enable_randomization: mapping.enable_randomization,
            jitter_offset,
            move_gen: Arc::new(AtomicU64::new(0)),
        },
    );
}

fn apply_direction_pad_tap_without_swipe(
    cs_tx: &ChannelSenderCS,
    mapping: &BindMappingDirectionPad,
    original_size: Vec2,
) {
    let original_pos: Vec2 = mapping.position.into();
    let random_offset = Vec2::new(mapping.random_offset_x, mapping.random_offset_y);
    let random_anchor = if mapping.enable_randomization {
        random_offset_vec2(original_pos, random_offset)
    } else {
        original_pos
    };

    ControlMsgHelper::send_touch(
        &cs_tx.0,
        MotionEventAction::Down,
        mapping.pointer_id,
        original_size,
        random_anchor,
    );
    ControlMsgHelper::send_touch(
        &cs_tx.0,
        MotionEventAction::Up,
        mapping.pointer_id,
        original_size,
        random_anchor,
    );
}

fn apply_direction_pad_up(
    cs_tx: &ChannelSenderCS,
    direction_pad_map: &mut DirectionPadMap,
    action: &str,
) -> bool {
    if let Some(item) = direction_pad_map.0.remove(action) {
        let last_pos = if item.enable_randomization {
            item.random_anchor + item.last_state_actual + item.current_jitter
        } else {
            item.original_pos + item.last_state
        };
        ControlMsgHelper::send_touch(
            &cs_tx.0,
            MotionEventAction::Up,
            item.pointer_id,
            item.original_size,
            last_pos,
        );
        true
    } else {
        false
    }
}

pub fn handle_direction_pad(
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
    block: Res<BlockDirectionPad>,
    mut direction_pad_map: ResMut<DirectionPadMap>,
    mut lifecycle_state: ResMut<DirectionPadLifecycleState>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    if block.0 {
        return;
    }

    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("DirectionPad") {
                let mapping = mapping.as_ref_directionpad();
                let key = action.to_string();
                let original_size: Vec2 = active_mapping.original_size.into();
                let mut state = scale_direction_2d_state(
                    ineffable.direction_2d(action.ineff_dual_axis()),
                    mapping,
                );
                if state.y < 0.0
                    && mapping.up_boost_key.as_ref().is_some_and(|b| {
                        b.is_any_key_pressed(&key_input) || b.is_any_mouse_pressed(&mouse_input)
                    })
                {
                    state.y *= mapping.up_boost_scale;
                }
                if direction_pad_map.0.contains_key(&key) {
                    let item = direction_pad_map.0.get_mut(&key).unwrap();
                    if !item.initial_swipe_done.load(Ordering::Relaxed) {
                        continue;
                    }
                    let original_pos: Vec2 = mapping.position.into();
                    if state.x == 0.0 && state.y == 0.0 {
                        if mapping
                            .bind
                            .is_any_direction_active(&key_input, &mouse_input)
                        {
                            // Opposite directions canceled — move back to center, don't lift.
                            let old_state = item.last_state_actual;
                            let actual_state = randomize_direction_pad_state(state, mapping);
                            item.last_state = state;
                            item.last_state_actual = actual_state;

                            if item.enable_randomization {
                                handle_direction_move_randomized(
                                    old_state,
                                    actual_state,
                                    item.random_anchor,
                                    &mut item.current_jitter,
                                    &mut item.next_jitter_at,
                                    &item.move_gen,
                                    mapping.pointer_id,
                                    original_size,
                                    &cs_tx_res.0,
                                    &runtime,
                                    SingleSwipeStrategy::ArcWithEaseInOut,
                                );
                            } else {
                                ControlMsgHelper::send_touch(
                                    &cs_tx_res.0,
                                    MotionEventAction::Move,
                                    mapping.pointer_id,
                                    original_size,
                                    original_pos + state,
                                );
                            }
                        } else {
                            // Genuine release: all keys up.
                            let released =
                                apply_direction_pad_up(&cs_tx_res, &mut direction_pad_map, &key);
                            if released {
                                lifecycle_state.0.clear_pending(&key);
                            }
                            if released && direction_pad_has_after_hook(mapping) {
                                let after_script_ast =
                                    mapping.script_hooks.after_script_ast.clone();
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
                                runtime.spawn_background_task(move |_task_ctx| async move {
                                    if let Err(e) =
                                        run_script_hook(&after_script_ast, &exec_ctx).await
                                    {
                                        log::error!(
                                            "[DirectionPad] script hook runtime error: {:?}",
                                            e
                                        );
                                    }
                                });
                            }
                        }
                    } else if state != item.last_state {
                        let old_state = item.last_state_actual;
                        let actual_state = randomize_direction_pad_state(state, mapping);
                        item.last_state = state;
                        item.last_state_actual = actual_state;

                        if item.enable_randomization {
                            handle_direction_move_randomized(
                                old_state,
                                actual_state,
                                item.random_anchor,
                                &mut item.current_jitter,
                                &mut item.next_jitter_at,
                                &item.move_gen,
                                mapping.pointer_id,
                                original_size,
                                &cs_tx_res.0,
                                &runtime,
                                SingleSwipeStrategy::ArcWithEaseInOut,
                            );
                        } else {
                            ControlMsgHelper::send_touch(
                                &cs_tx_res.0,
                                MotionEventAction::Move,
                                mapping.pointer_id,
                                original_size,
                                original_pos + state,
                            );
                        }
                    } else if item.enable_randomization && Instant::now() > item.next_jitter_at {
                        handle_direction_jitter_path(
                            item.last_state_actual,
                            item.random_anchor,
                            &mut item.current_jitter,
                            &mut item.next_jitter_at,
                            item.jitter_offset,
                            mapping.pointer_id,
                            original_size,
                            &cs_tx_res.0,
                        );
                    }
                } else if state.x != 0.0 || state.y != 0.0 {
                    if direction_pad_has_before_hook(mapping) {
                        let version = lifecycle_state.0.begin_start(&key);
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
                                        let key = key.clone();
                                        move |main_ctx| {
                                            main_ctx
                                                .world
                                                .resource_mut::<DirectionPadLifecycleState>()
                                                .0
                                                .cancel_start(&key, version);
                                        }
                                    })
                                    .await;
                                log::error!("[DirectionPad] script hook runtime error: {:?}", e);
                                return;
                            }

                            let pending_release = task_ctx
                                .run_on_main_thread(move |main_ctx| {
                                    let start = main_ctx
                                        .world
                                        .resource_mut::<DirectionPadLifecycleState>()
                                        .0
                                        .finish_start(&key, version);
                                    let pending_release = match start {
                                        MappingLifecycleStart::Stale => return None,
                                        MappingLifecycleStart::Ready { pending_release } => {
                                            pending_release
                                        }
                                    };
                                    if main_ctx.world.resource::<BlockDirectionPad>().0 {
                                        return None;
                                    }

                                    if pending_release.is_some() {
                                        apply_direction_pad_tap_without_swipe(
                                            &ChannelSenderCS(cs_tx),
                                            &mapping,
                                            original_size,
                                        );
                                    } else {
                                        let mut direction_pad_map = main_ctx
                                            .world
                                            .remove_resource::<DirectionPadMap>()
                                            .expect("DirectionPadMap resource missing");
                                        let runtime =
                                            main_ctx.world.resource::<TokioTasksRuntime>();
                                        apply_direction_pad_down(
                                            &ChannelSenderCS(cs_tx),
                                            runtime,
                                            &mut direction_pad_map,
                                            key,
                                            &mapping,
                                            original_size,
                                            state,
                                        );
                                        main_ctx.world.insert_resource(direction_pad_map);
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
                                        run_script_hook(&after_script_ast, &after_exec_ctx).await
                                    {
                                        log::error!(
                                            "[DirectionPad] script hook runtime error: {:?}",
                                            e
                                        );
                                    }
                                }
                            }
                        });
                    } else {
                        apply_direction_pad_down(
                            &cs_tx_res,
                            &runtime,
                            &mut direction_pad_map,
                            key,
                            mapping,
                            original_size,
                            state,
                        );
                    }
                } else if direction_pad_has_before_hook(mapping)
                    && !mapping
                        .bind
                        .is_any_direction_active(&key_input, &mouse_input)
                {
                    lifecycle_state.0.record_early_release(
                        &key,
                        DirectionPadReleaseContext {
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
