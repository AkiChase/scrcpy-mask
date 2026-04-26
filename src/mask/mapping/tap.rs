use std::{collections::HashMap, time::Duration};

use bevy::{
    ecs::{
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    math::Vec2,
    time::{Time, Timer, TimerMode},
};
use bevy_ineffable::prelude::{ContinuousBinding, Ineffable, InputBinding, PulseBinding};
use bevy_tokio_tasks::TokioTasksRuntime;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::{
    mask::{
        mapping::{
            binding::{ButtonBinding, ValidateMappingConfig},
            config::ActiveMappingConfig,
            utils::{ControlMsgHelper, Position, default_random_offset, random_offset_vec2},
        },
    },
    scrcpy::constant::MotionEventAction,
    utils::ChannelSenderCS,
};

pub fn tap_init(mut commands: Commands) {
    commands.insert_resource(ActiveRepeatTapMap::default());
    commands.insert_resource(ActiveSingleTapMap::default());
}

#[derive(Debug, Clone)]
pub struct BindMappingSingleTap {
    pub position: Position,
    pub note: String,
    pub pointer_id: u64,
    pub duration: u64,
    pub sync: bool,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
    pub random_offset_x: f32,
    pub random_offset_y: f32,
}

impl From<MappingSingleTap> for BindMappingSingleTap {
    fn from(value: MappingSingleTap) -> Self {
        Self {
            position: value.position,
            note: value.note,
            pointer_id: value.pointer_id,
            duration: value.duration,
            sync: value.sync,
            bind: value.bind.clone(),
            random_offset_x: value.random_offset_x,
            random_offset_y: value.random_offset_y,
            input_binding: ContinuousBinding::hold(value.bind).0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingSingleTap {
    pub position: Position,
    pub note: String,
    pub pointer_id: u64,
    pub duration: u64,
    pub sync: bool,
    pub bind: ButtonBinding,
    #[serde(default = "default_random_offset")]
    pub random_offset_x: f32,
    #[serde(default = "default_random_offset")]
    pub random_offset_y: f32,
}

impl ValidateMappingConfig for MappingSingleTap {}

#[derive(Resource, Default)]
pub struct ActiveSingleTapMap(HashMap<String, Vec2>);

pub fn handle_single_tap(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    mut active_single_tap: ResMut<ActiveSingleTapMap>,
    cs_tx_res: Res<ChannelSenderCS>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("SingleTap") {
                let original_size: Vec2 = active_mapping.original_size.into();
                let mapping = mapping.as_ref_singletap();
                if ineffable.just_activated(action.ineff_continuous()) {
                    if mapping.sync {
                        let random_pos = random_offset_vec2(
                            mapping.position.into(),
                            Vec2::new(mapping.random_offset_x, mapping.random_offset_y),
                        );
                        // Tap down sync
                        ControlMsgHelper::send_touch(
                            &cs_tx_res.0,
                            MotionEventAction::Down,
                            mapping.pointer_id,
                            original_size,
                            random_pos,
                        );
                        // add to active_map
                        active_single_tap.0.insert(action.to_string(), random_pos);
                    } else {
                        let cs_tx = cs_tx_res.0.clone();
                        let pointer_id = mapping.pointer_id;
                        let random_pos = random_offset_vec2(
                            mapping.position.into(),
                            Vec2::new(mapping.random_offset_x, mapping.random_offset_y),
                        );
                        let duration = Duration::from_millis(mapping.duration as u64);
                        // Tap down
                        ControlMsgHelper::send_touch(
                            &cs_tx,
                            MotionEventAction::Down,
                            pointer_id,
                            original_size,
                            random_pos,
                        );
                        // wait and Tap up
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
                } else if mapping.sync && ineffable.just_deactivated(action.ineff_continuous()) {
                    if let Some(random_pos) = active_single_tap.0.remove(action.as_ref()) {
                        // Tap up sync
                        ControlMsgHelper::send_touch(
                            &cs_tx_res.0,
                            MotionEventAction::Up,
                            mapping.pointer_id,
                            original_size,
                            random_pos,
                        );
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BindMappingRepeatTap {
    pub position: Position,
    pub note: String,
    pub pointer_id: u64,
    pub duration: u64,
    pub interval: u32,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
    pub random_offset_x: f32,
    pub random_offset_y: f32,
}

impl From<MappingRepeatTap> for BindMappingRepeatTap {
    fn from(value: MappingRepeatTap) -> Self {
        Self {
            position: value.position,
            note: value.note,
            pointer_id: value.pointer_id,
            duration: value.duration,
            interval: value.interval,
            bind: value.bind.clone(),
            input_binding: ContinuousBinding::hold(value.bind).0,
            random_offset_x: value.random_offset_x,
            random_offset_y: value.random_offset_y,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingRepeatTap {
    pub position: Position,
    pub note: String,
    pub pointer_id: u64,
    pub duration: u64,
    pub interval: u32,
    pub bind: ButtonBinding,
    #[serde(default = "default_random_offset")]
    pub random_offset_x: f32,
    #[serde(default = "default_random_offset")]
    pub random_offset_y: f32,
}

impl ValidateMappingConfig for MappingRepeatTap {}

#[derive(Resource, Default)]
pub struct ActiveRepeatTapMap(HashMap<String, RepeatTapTimer>);

struct RepeatTapTimer {
    timer: Timer,
    pointer_id: u64,
    original_pos: Vec2,
    original_size: Vec2,
    duration: Duration,
    random_offset: Vec2,
}

pub fn handle_repeat_tap_trigger(
    time: Res<Time>,
    mut active_map: ResMut<ActiveRepeatTapMap>,
    cs_tx_res: Res<ChannelSenderCS>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    for (_, timer) in active_map.0.iter_mut() {
        if timer.timer.tick(time.delta()).just_finished() {
            let cs_tx = cs_tx_res.0.clone();
            let original_size = timer.original_size;
            let pointer_id = timer.pointer_id;
            let random_pos = random_offset_vec2(timer.original_pos, timer.random_offset);
            let duration = timer.duration;
            // Tap down
            ControlMsgHelper::send_touch(
                &cs_tx,
                MotionEventAction::Down,
                pointer_id,
                original_size,
                random_pos,
            );
            // wait and Tap up
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
    }
}

pub fn handle_repeat_tap(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    mut active_map: ResMut<ActiveRepeatTapMap>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("RepeatTap") {
                let mapping = mapping.as_ref_repeattap();
                if ineffable.just_activated(action.ineff_continuous()) {
                    let interval = Duration::from_millis(mapping.interval as u64);
                    let original_size: Vec2 = active_mapping.original_size.into();
                    let mut timer = Timer::new(interval, TimerMode::Repeating);
                    timer.tick(interval);
                    active_map.0.insert(
                        action.to_string(),
                        RepeatTapTimer {
                            timer,
                            pointer_id: mapping.pointer_id,
                            original_pos: mapping.position.into(),
                            original_size: original_size,
                            duration: Duration::from_millis(mapping.duration as u64),
                            random_offset: Vec2::new(
                                mapping.random_offset_x,
                                mapping.random_offset_y,
                            ),
                        },
                    );
                } else if ineffable.just_deactivated(action.ineff_continuous()) {
                    active_map.0.remove(action.as_ref());
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
    pub note: String,
    pub pointer_id: u64,
    pub items: Vec<MappingMultipleTapItem>,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
    pub random_offset_x: f32,
    pub random_offset_y: f32,
}

impl From<MappingMultipleTap> for BindMappingMultipleTap {
    fn from(value: MappingMultipleTap) -> Self {
        Self {
            note: value.note,
            pointer_id: value.pointer_id,
            items: value.items,
            bind: value.bind.clone(),
            input_binding: PulseBinding::just_pressed(value.bind).0,
            random_offset_x: value.random_offset_x,
            random_offset_y: value.random_offset_y,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingMultipleTap {
    pub note: String,
    pub pointer_id: u64,
    pub items: Vec<MappingMultipleTapItem>,
    pub bind: ButtonBinding,
    #[serde(default = "default_random_offset")]
    pub random_offset_x: f32,
    #[serde(default = "default_random_offset")]
    pub random_offset_y: f32,
}

impl ValidateMappingConfig for MappingMultipleTap {
    fn validate(&self) -> Result<(), String> {
        if self.items.is_empty() {
            return Err("MultipleTap's operation item list is empty".to_string());
        }
        Ok(())
    }
}

pub fn handle_multiple_tap(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("MultipleTap") {
                let mapping = mapping.as_ref_multipletap();
                if ineffable.just_pulsed(action.ineff_pulse()) {
                    let cs_tx = cs_tx_res.0.clone();
                    let original_size: Vec2 = active_mapping.original_size.into();
                    let pointer_id = mapping.pointer_id;
                    let items = mapping.items.clone();
                    let random_offset = Vec2::new(mapping.random_offset_x, mapping.random_offset_y);
                    runtime.spawn_background_task(move |_ctx| async move {
                        for item in items {
                            let random_pos =
                                random_offset_vec2(item.position.into(), random_offset);
                            sleep(Duration::from_millis(item.wait)).await;
                            ControlMsgHelper::send_touch(
                                &cs_tx,
                                MotionEventAction::Down,
                                pointer_id,
                                original_size,
                                random_pos,
                            );
                            sleep(Duration::from_millis(item.duration)).await;
                            ControlMsgHelper::send_touch(
                                &cs_tx,
                                MotionEventAction::Up,
                                pointer_id,
                                original_size,
                                random_pos,
                            );
                        }
                    });
                }
            }
        }
    }
}
