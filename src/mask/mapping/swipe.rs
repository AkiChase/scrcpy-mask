use std::time::Duration;

use bevy::{
    ecs::system::{Res, ResMut},
    math::Vec2,
};
use bevy_ineffable::prelude::*;
use bevy_tokio_tasks::TokioTasksRuntime;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::{
    mask::mapping::{
        binding::{ButtonBinding, ValidateMappingConfig},
        config::ActiveMappingConfig,
        utils::{ControlMsgHelper, Position, SwipeStrategy, build_swipe_intermediate_points},
    },
    scrcpy::constant::MotionEventAction,
    utils::ChannelSenderCS,
};

#[derive(Debug, Clone)]
pub struct BindMappingSwipe {
    pub note: String,
    pub pointer_id: u64,
    pub positions: Vec<Position>,
    pub duration: u64,
    pub enable_randomization: bool,
    pub strategy: SwipeStrategy,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
}

impl From<MappingSwipe> for BindMappingSwipe {
    fn from(value: MappingSwipe) -> Self {
        // Backward compat: if old config had enable_randomization=true but no
        // explicit strategy, map to ArcWithCubicEasing.
        let strategy = if value.strategy == SwipeStrategy::default() && value.enable_randomization {
            SwipeStrategy::ArcWithCubicEasing
        } else {
            value.strategy
        };
        Self {
            note: value.note,
            pointer_id: value.pointer_id,
            positions: value.positions,
            duration: value.duration,
            enable_randomization: value.enable_randomization,
            strategy,
            bind: value.bind.clone(),
            input_binding: PulseBinding::just_pressed(value.bind).0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingSwipe {
    pub note: String,
    pub pointer_id: u64,
    pub positions: Vec<Position>,
    pub duration: u64,
    #[serde(default)]
    pub enable_randomization: bool,
    #[serde(default)]
    pub strategy: SwipeStrategy,
    pub bind: ButtonBinding,
}

impl ValidateMappingConfig for MappingSwipe {
    fn validate(&self) -> Result<(), String> {
        if self.positions.is_empty() {
            return Err("Swipe's position list is empty".to_string());
        }
        Ok(())
    }
}

pub fn handle_swipe(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("Swipe") {
                let mapping = mapping.as_ref_swipe();
                let original_size: Vec2 = active_mapping.original_size.into();
                if ineffable.just_pulsed(action.ineff_pulse()) {
                    let cs_tx = cs_tx_res.0.clone();
                    let pointer_id = mapping.pointer_id;
                    let points = mapping.positions.clone();
                    let duration = mapping.duration;
                    let strategy = mapping.strategy;
                    runtime.spawn_background_task(move |_ctx| async move {
                        ControlMsgHelper::send_touch(
                            &cs_tx,
                            MotionEventAction::Down,
                            pointer_id,
                            original_size,
                            points[0].into(),
                        );
                        let mut cur_pos: Vec2 = points[0].into();
                        for i in 1..points.len() {
                            let next_pos: Vec2 = points[i].into();
                            for step in build_swipe_intermediate_points(
                                cur_pos,
                                next_pos,
                                strategy,
                                duration,
                            ) {
                                ControlMsgHelper::send_touch(
                                    &cs_tx,
                                    MotionEventAction::Move,
                                    pointer_id,
                                    original_size,
                                    step.pos,
                                );
                                sleep(Duration::from_millis(step.wait_ms)).await;
                            }

                            ControlMsgHelper::send_touch(
                                &cs_tx,
                                MotionEventAction::Move,
                                pointer_id,
                                original_size,
                                next_pos,
                            );
                            cur_pos = next_pos;
                        }
                        ControlMsgHelper::send_touch(
                            &cs_tx,
                            MotionEventAction::Up,
                            pointer_id,
                            original_size,
                            cur_pos.into(),
                        );
                    });
                }
            }
        }
    }
}
