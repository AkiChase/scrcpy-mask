use std::collections::HashMap;

use bevy::{
    ecs::{
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    math::Vec2,
};
use bevy_ineffable::prelude::{ContinuousBinding, Ineffable, InputBinding};
use serde::{Deserialize, Serialize};

use crate::{
    mask::{
        mapping::{
            binding::{ButtonBinding, ValidateMappingConfig},
            config::ActiveMappingConfig,
            cursor::CursorPosition,
            utils::{ControlMsgHelper, Position},
        },
        mask_command::MaskSize,
    },
    scrcpy::constant::MotionEventAction,
    utils::ChannelSenderCS,
};

pub fn init_observation(mut commands: Commands) {
    commands.insert_resource(ActiveObservation::default());
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BindMappingObservation {
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub sensitivity_x: f32,
    pub sensitivity_y: f32,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
}

impl From<MappingObservation> for BindMappingObservation {
    fn from(value: MappingObservation) -> Self {
        Self {
            note: value.note,
            pointer_id: value.pointer_id,
            position: value.position,
            sensitivity_x: value.sensitivity_x,
            sensitivity_y: value.sensitivity_y,
            bind: value.bind.clone(),
            input_binding: ContinuousBinding::hold(value.bind).0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingObservation {
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub sensitivity_x: f32,
    pub sensitivity_y: f32,
    pub bind: ButtonBinding,
}

impl ValidateMappingConfig for MappingObservation {}

#[derive(Resource, Default)]
pub struct ActiveObservation(HashMap<String, ObservationItem>);

struct ObservationItem {
    start_cursor_pos: Vec2,
    mask_pos: Vec2,
    pointer_id: u64,
    sensitivity: Vec2,
}

pub fn handle_observation_trigger(
    cs_tx_res: Res<ChannelSenderCS>,
    mask_size: Res<MaskSize>,
    cursor_pos: Res<CursorPosition>,
    mut active_map: ResMut<ActiveObservation>,
) {
    for (_, item) in active_map.0.iter_mut() {
        let delta = (cursor_pos.0 - item.start_cursor_pos) * item.sensitivity;
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Move,
            item.pointer_id,
            mask_size.0,
            item.mask_pos + delta,
        );
    }
}

pub fn handle_observation(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    mask_size: Res<MaskSize>,
    cursor_pos: Res<CursorPosition>,
    mut active_map: ResMut<ActiveObservation>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("Observation") {
                let mapping = mapping.as_ref_observation();
                if ineffable.just_activated(action.ineff_continuous()) {
                    let original_size: Vec2 = active_mapping.original_size.into();
                    let original_pos: Vec2 = mapping.position.into();
                    let sensitivity: Vec2 = (mapping.sensitivity_x, mapping.sensitivity_y).into();
                    let pointer_id = mapping.pointer_id;
                    let mask_pos = original_pos / original_size * mask_size.0;
                    // touch down
                    ControlMsgHelper::send_touch(
                        &cs_tx_res.0,
                        MotionEventAction::Down,
                        pointer_id,
                        mask_size.0,
                        mask_pos,
                    );
                    // add to active_map
                    active_map.0.insert(
                        action.to_string(),
                        ObservationItem {
                            start_cursor_pos: cursor_pos.0,
                            mask_pos,
                            pointer_id,
                            sensitivity,
                        },
                    );
                } else if ineffable.just_deactivated(action.ineff_continuous()) {
                    if let Some(item) = active_map.0.remove(action.as_ref()) {
                        // touch up
                        let delta = (cursor_pos.0 - item.start_cursor_pos) * item.sensitivity;
                        ControlMsgHelper::send_touch(
                            &cs_tx_res.0,
                            MotionEventAction::Up,
                            item.pointer_id,
                            mask_size.0,
                            item.mask_pos + delta,
                        );
                    }
                }
            }
        }
    }
}
