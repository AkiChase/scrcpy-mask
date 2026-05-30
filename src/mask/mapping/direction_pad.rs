use std::{
    collections::HashMap,
    sync::{atomic::{AtomicU64, Ordering}, Arc},
    time::{Duration, Instant},
};

use bevy::{
    ecs::{
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    math::Vec2,
};
use bevy_ineffable::prelude::{Ineffable, InputBinding};
use bevy_tokio_tasks::TokioTasksRuntime;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::{
    mask::mapping::{
        binding::{DirectionBinding, ValidateMappingConfig},
        config::ActiveMappingConfig,
        utils::{
            anchor_random_offset, build_single_segment_swipe_intermediate_points,
            ControlMsgHelper, DEFAULT_SWIPE_DURATION, Position, SingleSwipeStrategy,
            micro_jitter, next_jitter_deadline, random_offset_vec2,
        },
    },
    scrcpy::constant::MotionEventAction,
    utils::ChannelSenderCS,
};

pub fn direction_pad_init(mut commands: Commands) {
    commands.insert_resource(DirectionPadMap::default());
    commands.insert_resource(BlockDirectionPad::default());
}

#[derive(Debug, Clone)]
pub struct BindMappingDirectionPad {
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub initial_duration: u64,
    pub max_offset_x: f32,
    pub max_offset_y: f32,
    pub enable_randomization: bool,
    pub bind: DirectionBinding,
    pub input_binding: InputBinding,
}

impl From<MappingDirectionPad> for BindMappingDirectionPad {
    fn from(value: MappingDirectionPad) -> Self {
        Self {
            note: value.note,
            pointer_id: value.pointer_id,
            position: value.position,
            initial_duration: value.initial_duration,
            max_offset_x: value.max_offset_x,
            max_offset_y: value.max_offset_y,
            enable_randomization: value.enable_randomization,
            bind: value.bind.clone(),
            input_binding: value.bind.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingDirectionPad {
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub initial_duration: u64,
    pub max_offset_x: f32,
    pub max_offset_y: f32,
    #[serde(default)]
    pub enable_randomization: bool,
    pub bind: DirectionBinding,
}

impl ValidateMappingConfig for MappingDirectionPad {}

#[derive(Resource, Default)]
pub struct DirectionPadMap(pub HashMap<String, DirectionPadItem>);

pub struct DirectionPadItem {
    pub enable_instant: Instant,
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
                let state = scale_direction_2d_state(
                    ineffable.direction_2d(action.ineff_dual_axis()),
                    mapping,
                );
                if direction_pad_map.0.contains_key(&key) {
                    let item = direction_pad_map.0.get_mut(&key).unwrap();
                    if item.enable_instant > Instant::now() {
                        continue;
                    }
                    let original_pos: Vec2 = mapping.position.into();
                    if state.x == 0.0 && state.y == 0.0 {
                        // touch up
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
                    } else if state != item.last_state {
                        let old_state = item.last_state_actual;
                        item.last_state = state;
                        item.last_state_actual = state;

                        if item.enable_randomization {
                            let cur = item.random_anchor + old_state + item.current_jitter;
                            let target = item.random_anchor + state;
                            let dist = cur.distance(target);
                            if dist > 2.0 {
                                let duration = DEFAULT_SWIPE_DURATION;
                                let points = build_single_segment_swipe_intermediate_points(
                                    cur,
                                    target,
                                    SingleSwipeStrategy::ArcWithEaseInOut,
                                    duration,
                                );
                                let cs_tx = cs_tx_res.0.clone();
                                let pointer_id = mapping.pointer_id;
                                let expected_gen = item.move_gen.fetch_add(1, Ordering::SeqCst) + 1;
                                let move_gen = item.move_gen.clone();
                                runtime.spawn_background_task(move |_ctx| async move {
                                    for point in points {
                                        if move_gen.load(Ordering::Relaxed) != expected_gen {
                                            return;
                                        }
                                        ControlMsgHelper::send_touch(
                                            &cs_tx,
                                            MotionEventAction::Move,
                                            pointer_id,
                                            original_size,
                                            point.pos,
                                        );
                                        sleep(Duration::from_millis(point.wait_ms))
                                            .await;
                                    }
                                });
                            } else {
                                ControlMsgHelper::send_touch(
                                    &cs_tx_res.0,
                                    MotionEventAction::Move,
                                    mapping.pointer_id,
                                    original_size,
                                    target,
                                );
                            }
                            item.current_jitter = Vec2::ZERO;
                            item.next_jitter_at = next_jitter_deadline();
                        } else {
                            ControlMsgHelper::send_touch(
                                &cs_tx_res.0,
                                MotionEventAction::Move,
                                mapping.pointer_id,
                                original_size,
                                original_pos + state,
                            );
                        }
                    } else if item.enable_randomization
                        && Instant::now() > item.next_jitter_at
                    {
                        let jitter = micro_jitter(item.random_offset);
                        ControlMsgHelper::send_touch(
                            &cs_tx_res.0,
                            MotionEventAction::Move,
                            mapping.pointer_id,
                            original_size,
                            item.random_anchor + state + jitter,
                        );
                        item.current_jitter = jitter;
                        item.next_jitter_at = next_jitter_deadline();
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

                    let enable_instant = Instant::now()
                        + Duration::from_millis(
                            mapping.initial_duration + DEFAULT_SWIPE_DURATION,
                        );

                    direction_pad_map.0.insert(
                        key,
                        DirectionPadItem {
                            enable_instant,
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

                    // touch down
                    ControlMsgHelper::send_touch(
                        &cs_tx_res.0,
                        MotionEventAction::Down,
                        pointer_id,
                        original_size,
                        random_anchor,
                    );

                    // initial slide
                    if mapping.enable_randomization {
                        let points = build_single_segment_swipe_intermediate_points(
                            random_anchor,
                            random_anchor + state,
                            SingleSwipeStrategy::ArcWithEaseOut,
                            DEFAULT_SWIPE_DURATION,
                        );
                        let cs_tx = cs_tx_res.0.clone();
                        let initial_duration = mapping.initial_duration;
                        runtime.spawn_background_task(move |_ctx| async move {
                            sleep(Duration::from_millis(initial_duration)).await;
                            for point in points {
                                ControlMsgHelper::send_touch(
                                    &cs_tx,
                                    MotionEventAction::Move,
                                    pointer_id,
                                    original_size,
                                    point.pos,
                                );
                                sleep(Duration::from_millis(point.wait_ms)).await;
                            }
                        });
                    } else {
                        let points = build_single_segment_swipe_intermediate_points(
                            original_pos,
                            original_pos + state,
                            SingleSwipeStrategy::Linear,
                            DEFAULT_SWIPE_DURATION,
                        );
                        let cs_tx = cs_tx_res.0.clone();
                        let initial_duration = mapping.initial_duration;
                        runtime.spawn_background_task(move |_ctx| async move {
                            sleep(Duration::from_millis(initial_duration)).await;
                            for point in points {
                                ControlMsgHelper::send_touch(
                                    &cs_tx,
                                    MotionEventAction::Move,
                                    pointer_id,
                                    original_size,
                                    point.pos,
                                );
                                sleep(Duration::from_millis(point.wait_ms)).await;
                            }
                        });
                    }
                }
            }
        }
    }
}
