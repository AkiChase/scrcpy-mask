use std::{
    collections::HashMap,
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
        utils::{ControlMsgHelper, MIN_MOVE_STEP_INTERVAL, Position, ease_sigmoid_like},
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
    pub last_state: Vec2,
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
                        // wait for initial duration
                        continue;
                    }
                    let original_pos: Vec2 = mapping.position.into();
                    if state.x == 0.0 && state.y == 0.0 {
                        // touch up and remove state
                        ControlMsgHelper::send_touch(
                            &cs_tx_res.0,
                            MotionEventAction::Up,
                            mapping.pointer_id,
                            original_size,
                            original_pos + item.last_state,
                        );
                        direction_pad_map.0.remove(&key);
                    } else if state != item.last_state {
                        // record new state
                        item.last_state = state;
                        // move to new state
                        ControlMsgHelper::send_touch(
                            &cs_tx_res.0,
                            MotionEventAction::Move,
                            mapping.pointer_id,
                            original_size,
                            original_pos + state,
                        );
                    }
                } else if state.x != 0.0 || state.y != 0.0 {
                    let pointer_id = mapping.pointer_id;
                    let original_size: Vec2 = active_mapping.original_size.into();
                    let original_pos: Vec2 = mapping.position.into();

                    let enable_instant = Instant::now()
                        + Duration::from_millis(mapping.initial_duration + MIN_MOVE_STEP_INTERVAL);

                    // record new item
                    direction_pad_map.0.insert(
                        key,
                        DirectionPadItem {
                            enable_instant,
                            pointer_id,
                            original_size,
                            original_pos: original_pos,
                            last_state: state,
                        },
                    );
                    // touch down
                    let cs_tx = cs_tx_res.0.clone();
                    ControlMsgHelper::send_touch(
                        &cs_tx,
                        MotionEventAction::Down,
                        pointer_id,
                        original_size,
                        original_pos,
                    );
                    // move to state with initial_duration
                    let delta = state;
                    let steps: u64 =
                        std::cmp::max(1, mapping.initial_duration / MIN_MOVE_STEP_INTERVAL);

                    runtime.spawn_background_task(move |_ctx| async move {
                        for step in 1..=steps {
                            let linear_t = step as f32 / steps as f32;
                            let eased_t = ease_sigmoid_like(linear_t);
                            let interp = original_pos + delta * eased_t;
                            ControlMsgHelper::send_touch(
                                &cs_tx,
                                MotionEventAction::Move,
                                pointer_id,
                                original_size,
                                interp,
                            );
                            sleep(Duration::from_millis(MIN_MOVE_STEP_INTERVAL)).await;
                        }
                    });
                }
            }
        }
    }
}
