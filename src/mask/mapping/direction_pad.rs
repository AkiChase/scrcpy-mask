use std::{
    collections::HashMap,
    sync::Arc,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Instant,
};

use crate::tokio_tasks::TokioTasksRuntime;
use crate::{
    mask::mapping::{
        binding::{ButtonBinding, DirectionBinding, ValidateMappingConfig},
        config::ActiveMappingConfig,
        utils::{
            ControlMsgHelper, DEFAULT_SWIPE_DURATION, Position, SingleSwipeStrategy,
            anchor_random_offset, handle_direction_jitter, handle_direction_move_randomized,
            next_jitter_deadline, random_offset_vec2, spawn_initial_swipe,
        },
    },
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
};
use bevy_ineffable::prelude::{Ineffable, InputBinding};
use serde::{Deserialize, Serialize};

pub fn direction_pad_init(mut commands: Commands) {
    commands.insert_resource(DirectionPadMap::default());
    commands.insert_resource(BlockDirectionPad::default());
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
    pub up_boost_key: Option<ButtonBinding>,
    pub up_boost_scale: f32,
    pub bind: DirectionBinding,
    pub input_binding: InputBinding,
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
            up_boost_key: value.up_boost_key,
            up_boost_scale: value.up_boost_scale,
            bind: value.bind.clone(),
            input_binding: value.bind.into(),
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
    #[serde(default)]
    pub up_boost_key: Option<ButtonBinding>,
    #[serde(
        default = "default_up_boost_scale",
        serialize_with = "crate::mask::mapping::serde_float::serialize_f32_3dp"
    )]
    pub up_boost_scale: f32,
    pub bind: DirectionBinding,
}

fn default_up_boost_scale() -> f32 {
    1.4
}

impl ValidateMappingConfig for MappingDirectionPad {}

#[derive(Resource, Default)]
pub struct DirectionPadMap(pub HashMap<String, DirectionPadItem>);

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
    pub random_offset: Vec2,
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

pub fn handle_direction_pad(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    runtime: ResMut<TokioTasksRuntime>,
    block: Res<BlockDirectionPad>,
    mut direction_pad_map: ResMut<DirectionPadMap>,
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
                            item.last_state = state;
                            item.last_state_actual = state;

                            if item.enable_randomization {
                                handle_direction_move_randomized(
                                    old_state,
                                    state,
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
                            let last_pos = if item.enable_randomization {
                                item.random_anchor + item.last_state + item.current_jitter
                            } else {
                                original_pos + item.last_state
                            };
                            ControlMsgHelper::send_touch(
                                &cs_tx_res.0,
                                MotionEventAction::Up,
                                mapping.pointer_id,
                                original_size,
                                last_pos,
                            );
                            direction_pad_map.0.remove(&key);
                        }
                    } else if state != item.last_state {
                        let old_state = item.last_state_actual;
                        item.last_state = state;
                        item.last_state_actual = state;

                        if item.enable_randomization {
                            handle_direction_move_randomized(
                                old_state,
                                state,
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
                        handle_direction_jitter(
                            state,
                            item.random_anchor,
                            &mut item.current_jitter,
                            &mut item.next_jitter_at,
                            item.random_offset,
                            mapping.pointer_id,
                            original_size,
                            &cs_tx_res.0,
                        );
                    }
                } else if state.x != 0.0 || state.y != 0.0 {
                    let pointer_id = mapping.pointer_id;
                    let original_size: Vec2 = active_mapping.original_size.into();
                    let original_pos: Vec2 = mapping.position.into();

                    let random_offset =
                        anchor_random_offset(mapping.max_offset_x, mapping.max_offset_y);
                    let random_anchor = if mapping.enable_randomization {
                        random_offset_vec2(original_pos, random_offset)
                    } else {
                        original_pos
                    };

                    // touch down
                    ControlMsgHelper::send_touch(
                        &cs_tx_res.0,
                        MotionEventAction::Down,
                        pointer_id,
                        original_size,
                        random_anchor,
                    );

                    // initial swipe (jitter during initial_duration, then slide to target)
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
                        &runtime,
                        &cs_tx_res.0,
                        pointer_id,
                        original_size,
                        swipe_start,
                        swipe_start + state,
                        mapping.initial_duration,
                        DEFAULT_SWIPE_DURATION,
                        strategy,
                    );

                    direction_pad_map.0.insert(
                        key,
                        DirectionPadItem {
                            initial_swipe_done,
                            pointer_id,
                            original_size,
                            original_pos,
                            random_anchor,
                            last_state: state,
                            last_state_actual: state,
                            next_jitter_at: next_jitter_deadline(),
                            current_jitter: Vec2::ZERO,
                            enable_randomization: mapping.enable_randomization,
                            random_offset,
                            move_gen: Arc::new(AtomicU64::new(0)),
                        },
                    );
                }
            }
        }
    }
}
