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
            binding::{ButtonBinding, ValidateMappingConfig},
            config::ActiveMappingConfig,
            cursor::{ActiveCursorFpsConfig, CursorPosition, CursorState, FPS_MARGIN},
            utils::{ControlMsgHelper, Position},
        },
        mask_command::MaskSize,
    },
    scrcpy::constant::MotionEventAction,
    utils::ChannelSenderCS,
};

pub fn fire_init(mut commands: Commands) {
    commands.insert_resource(ActiveFireMap::default());
}

#[derive(Debug, Clone)]
pub struct BindMappingFps {
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub sensitivity_x: f32,
    pub sensitivity_y: f32,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
}

impl From<MappingFps> for BindMappingFps {
    fn from(value: MappingFps) -> Self {
        Self {
            note: value.note,
            pointer_id: value.pointer_id,
            position: value.position,
            sensitivity_x: value.sensitivity_x,
            sensitivity_y: value.sensitivity_y,
            bind: value.bind.clone(),
            input_binding: PulseBinding::just_pressed(value.bind).0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingFps {
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub sensitivity_x: f32,
    pub sensitivity_y: f32,
    pub bind: ButtonBinding,
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
        Ok(())
    }
}

pub fn handle_fps(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    mut fps_config: ResMut<ActiveCursorFpsConfig>,
    state: Res<State<CursorState>>,
    mut next_state: ResMut<NextState<CursorState>>,
    cursor_pos: Res<CursorPosition>,
    mask_size: Res<MaskSize>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("Fps") {
                if ineffable.just_pulsed(action.ineff_pulse()) {
                    let original_size: Vec2 = active_mapping.original_size.into();
                    match state.get() {
                        CursorState::Normal => {
                            let mapping = mapping.as_ref_fps();
                            let original_pos = mapping.position.into();
                            fps_config.pointer_id = mapping.pointer_id;
                            fps_config.original_pos = original_pos;
                            fps_config.original_size = original_size;
                            fps_config.ignore_fps_motion = false;
                            fps_config.sensitivity =
                                (mapping.sensitivity_x, mapping.sensitivity_y).into();
                            // touch down center
                            ControlMsgHelper::send_touch(
                                &cs_tx_res.0,
                                MotionEventAction::Down,
                                mapping.pointer_id,
                                original_size,
                                original_pos,
                            );
                            next_state.set(CursorState::Fps);
                            log::info!("[Cursor] {}", t!("mask.mapping.enterFpsMode"));
                        }
                        CursorState::Fps => {
                            // touch up
                            ControlMsgHelper::send_touch(
                                &cs_tx_res.0,
                                MotionEventAction::Up,
                                0,
                                mask_size.0, // cursor_pos is related to mask size
                                cursor_pos.0,
                            );
                            next_state.set(CursorState::Normal);
                            log::info!("[Cursor] {}", t!("mask.mapping.exitFpsMode"));
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
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub sensitivity_x: f32,
    pub sensitivity_y: f32,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
}

impl From<MappingFire> for BindMappingFire {
    fn from(value: MappingFire) -> Self {
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
pub struct MappingFire {
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub sensitivity_x: f32,
    pub sensitivity_y: f32,
    pub bind: ButtonBinding,
}

impl ValidateMappingConfig for MappingFire {}

#[derive(Resource, Default)]
pub struct ActiveFireMap(HashMap<String, FireItem>);

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

pub fn handle_fire(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    mut fps_config: ResMut<ActiveCursorFpsConfig>,
    mut active_map: ResMut<ActiveFireMap>,
    mut cursor_pos: ResMut<CursorPosition>,
    mask_size: Res<MaskSize>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("Fire") {
                let mapping = mapping.as_ref_fire();
                if ineffable.just_activated(action.ineff_continuous()) {
                    // stop fps motion
                    fps_config.ignore_fps_motion = true;
                    // touch up fps
                    ControlMsgHelper::send_touch(
                        &cs_tx_res.0,
                        MotionEventAction::Up,
                        fps_config.pointer_id,
                        mask_size.0,
                        cursor_pos.0, // fps cursor pos
                    );
                    let original_size: Vec2 = active_mapping.original_size.into();
                    let original_pos: Vec2 = mapping.position.into();
                    let sensitivity: Vec2 = (mapping.sensitivity_x, mapping.sensitivity_y).into();
                    let pointer_id = mapping.pointer_id;
                    let current_pos = original_pos / original_size * mask_size.0;
                    // touch down fire
                    ControlMsgHelper::send_touch(
                        &cs_tx_res.0,
                        MotionEventAction::Down,
                        pointer_id,
                        original_size,
                        original_pos,
                    );
                    // add to active_map
                    active_map.0.insert(
                        action.to_string(),
                        FireItem {
                            current_pos, // independent pos
                            pointer_id,
                            sensitivity,
                        },
                    );
                } else if ineffable.just_deactivated(action.ineff_continuous()) {
                    if let Some(fire_item) = active_map.0.remove(action.as_ref()) {
                        // touch up fire
                        ControlMsgHelper::send_touch(
                            &cs_tx_res.0,
                            MotionEventAction::Up,
                            fire_item.pointer_id,
                            mask_size.0,
                            fire_item.current_pos,
                        );
                        // touch down fps center
                        ControlMsgHelper::send_touch(
                            &cs_tx_res.0,
                            MotionEventAction::Down,
                            fps_config.pointer_id,
                            fps_config.original_size,
                            fps_config.original_pos,
                        );
                        // set cursor pos to fps center
                        cursor_pos.0 =
                            fps_config.original_pos / fps_config.original_size * mask_size.0;
                        // continue fps motion
                        fps_config.ignore_fps_motion = false;
                    }
                }
            }
        }
    }
}
