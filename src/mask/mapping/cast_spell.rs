use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Instant,
};

use bevy::{
    ecs::{
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    math::Vec2,
};
use bevy_ineffable::prelude::{ContinuousBinding, Ineffable, InputBinding, PulseBinding};
use bevy_tokio_tasks::TokioTasksRuntime;
use serde::{Deserialize, Serialize};

use crate::{
    mask::{
        mapping::{
            binding::{ButtonBinding, DirectionBinding, ValidateMappingConfig},
            config::{ActiveMappingConfig, MappingAction},
            cursor::CursorPosition,
            direction_pad::{BlockDirectionPad, DirectionPadMap},
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

pub fn cast_spell_init(mut commands: Commands) {
    commands.insert_resource(ActiveCastSpell::default());
}

#[derive(Resource, Default)]
pub struct ActiveCastSpell(Option<ActiveCastSpellItem>);

const CAST_SPELL_DELAY: u64 = 50;

struct ActiveCastSpellItem {
    key: String,
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
}

impl From<MappingMouseCastSpell> for BindMappingMouseCastSpell {
    fn from(value: MappingMouseCastSpell) -> Self {
        Self {
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
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingMouseCastSpell {
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
}

impl ValidateMappingConfig for MappingMouseCastSpell {}

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
    mask_size: Res<MaskSize>,
    cursor_pos: Res<CursorPosition>,
    runtime: ResMut<TokioTasksRuntime>,
    mut active_cast: ResMut<ActiveCastSpell>,
    mut block_direction_pad: ResMut<BlockDirectionPad>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("MouseCastSpell") {
                let mapping = mapping.as_ref_mousecastspell();
                if ineffable.just_activated(action.ineff_continuous()) {
                    let cur_cursor_pos = cursor_pos.0;
                    let cur_mask_size = mask_size.0;

                    // clear and touch up existing active cast
                    // for OnSecondPress cast, we do the same thing
                    if let Some(cast) = active_cast.0.take() {
                        ControlMsgHelper::send_touch(
                            &cs_tx_res.0,
                            MotionEventAction::Up,
                            cast.pointer_id,
                            cur_mask_size,
                            cast.current_pos,
                        );

                        if cast.block_direction_pad {
                            block_direction_pad.0 = false;
                        }

                        if cast.key == action.as_ref() {
                            continue;
                        }
                    }

                    let original_size: Vec2 = active_mapping.original_size.into();
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
                    let current_pos = original_pos / original_size * cur_mask_size;

                    // touch down new cast
                    ControlMsgHelper::send_touch(
                        &cs_tx_res.0,
                        MotionEventAction::Down,
                        pointer_id,
                        mask_size.0,
                        current_pos,
                    );

                    let target_pos = if !cast_no_direction {
                        cal_mouse_cast_spell_current_pos(
                            cur_cursor_pos,
                            center_pos,
                            original_pos,
                            cast_radius,
                            drag_radius,
                            cur_mask_size,
                            original_size,
                            horizontal_scale_factor,
                            vertical_scale_factor,
                        )
                    } else {
                        current_pos
                    };

                    let initial_swipe_done = spawn_initial_swipe(
                        &runtime,
                        &cs_tx_res.0,
                        pointer_id,
                        mask_size.0,
                        current_pos,
                        target_pos,
                        mapping.initial_duration,
                        DEFAULT_SWIPE_DURATION,
                        SingleSwipeStrategy::ArcWithEaseOut,
                    );

                    if matches!(release_mode, MouseCastReleaseMode::OnPress) {
                        // OnPress: self-contained, touch up after initial animation
                        let cs_tx = cs_tx_res.0.clone();
                        let cur_mask_size = mask_size.0;
                        runtime.spawn_background_task(move |_ctx| async move {
                            // wait for initial swipe to finish
                            while !initial_swipe_done.load(Ordering::Relaxed) {
                                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                            }
                            ControlMsgHelper::send_touch(
                                &cs_tx,
                                MotionEventAction::Up,
                                pointer_id,
                                cur_mask_size,
                                target_pos,
                            );
                        });
                    } else {
                        active_cast.0 = Some(ActiveCastSpellItem::new_mouse_item(
                            action.to_string(),
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
                } else if ineffable.just_deactivated(action.ineff_continuous()) {
                    if let MouseCastReleaseMode::OnRelease = mapping.release_mode {
                        let Some(cast) = &active_cast.0 else {
                            continue;
                        };

                        if cast.key != action.as_ref() {
                            continue;
                        }
                        // clear and touch up
                        if let Some(cast) = active_cast.0.take() {
                            ControlMsgHelper::send_touch(
                                &cs_tx_res.0,
                                MotionEventAction::Up,
                                cast.pointer_id,
                                mask_size.0,
                                cast.current_pos,
                            );

                            if cast.block_direction_pad {
                                block_direction_pad.0 = false;
                            }
                        }
                    }
                }
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
}

impl From<MappingPadCastSpell> for BindMappingPadCastSpell {
    fn from(value: MappingPadCastSpell) -> Self {
        Self {
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
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingPadCastSpell {
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
}

impl ValidateMappingConfig for MappingPadCastSpell {}

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
    mask_size: Res<MaskSize>,
    runtime: ResMut<TokioTasksRuntime>,
    mut active_cast: ResMut<ActiveCastSpell>,
    mut direction_pad_map: ResMut<DirectionPadMap>,
    mut block_direction_pad: ResMut<BlockDirectionPad>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("PadCastSpell") {
                let mapping = mapping.as_ref_padcastspell();
                if ineffable.just_activated(action.ineff_continuous()) {
                    // clear and touch up existing active cast
                    // for OnSecondPress cast, we do the same thing
                    if let Some(cast) = active_cast.0.take() {
                        ControlMsgHelper::send_touch(
                            &cs_tx_res.0,
                            MotionEventAction::Up,
                            cast.pointer_id,
                            mask_size.0,
                            cast.current_pos,
                        );

                        if cast.block_direction_pad {
                            block_direction_pad.0 = false;
                        }

                        if cast.key == action.as_ref() {
                            continue;
                        }
                    }

                    if mapping.block_direction_pad {
                        // block direction pad
                        block_direction_pad.0 = true;
                        // touch up and remove state
                        for (_key, item) in direction_pad_map.0.drain() {
                            ControlMsgHelper::send_touch(
                                &cs_tx_res.0,
                                MotionEventAction::Up,
                                item.pointer_id,
                                item.original_size,
                                item.original_pos + item.last_state,
                            );
                        }
                    }

                    let original_size: Vec2 = active_mapping.original_size.into();
                    let pointer_id = mapping.pointer_id;
                    let original_pos: Vec2 = mapping.position.into();
                    let original_pos = random_offset_vec2(
                        original_pos,
                        Vec2::new(mapping.random_offset_x, mapping.random_offset_y),
                    );
                    let current_pos = original_pos / original_size * mask_size.0;
                    let (random_anchor, random_offset) = if mapping.enable_randomization {
                        let offset = anchor_random_offset(mapping.drag_radius, mapping.drag_radius);
                        let anchor = random_offset_vec2(original_pos, offset);
                        (anchor, offset)
                    } else {
                        (Vec2::ZERO, Vec2::ZERO)
                    };

                    // touch down new cast
                    ControlMsgHelper::send_touch(
                        &cs_tx_res.0,
                        MotionEventAction::Down,
                        pointer_id,
                        mask_size.0,
                        current_pos,
                    );

                    // initial animation (jitter at start during initial_duration, then a
                    // zero-length swipe — the completion signal unlocks subsequent input)
                    let slide_start = if mapping.enable_randomization {
                        random_anchor / original_size * mask_size.0
                    } else {
                        current_pos
                    };
                    let strategy = if mapping.enable_randomization {
                        SingleSwipeStrategy::ArcWithEaseOut
                    } else {
                        SingleSwipeStrategy::Linear
                    };
                    let initial_swipe_done = spawn_initial_swipe(
                        &runtime,
                        &cs_tx_res.0,
                        pointer_id,
                        mask_size.0,
                        slide_start,
                        slide_start, // target = start (shown direction by handle_pad_cast_spell_trigger)
                        0,           // no initial wait: the activate button itself signals intent
                        DEFAULT_SWIPE_DURATION,
                        strategy,
                    );

                    // set active
                    active_cast.0 = Some(ActiveCastSpellItem::new_pad_item(
                        action.to_string(),
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
                } else if ineffable.just_deactivated(action.ineff_continuous()) {
                    if let PadCastReleaseMode::OnRelease = mapping.release_mode {
                        let Some(cast) = &active_cast.0 else {
                            continue;
                        };

                        if cast.key != action.as_ref() {
                            continue;
                        }
                        // clear and touch up
                        if let Some(cast) = active_cast.0.take() {
                            ControlMsgHelper::send_touch(
                                &cs_tx_res.0,
                                MotionEventAction::Up,
                                cast.pointer_id,
                                mask_size.0,
                                cast.current_pos,
                            );
                            if cast.block_direction_pad {
                                block_direction_pad.0 = false;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BindMappingCancelCast {
    pub note: String,
    pub position: Position,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
}

impl From<MappingCancelCast> for BindMappingCancelCast {
    fn from(value: MappingCancelCast) -> Self {
        Self {
            position: value.position,
            note: value.note,
            bind: value.bind.clone(),
            input_binding: PulseBinding::just_pressed(value.bind).0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingCancelCast {
    pub note: String,
    pub position: Position,
    pub bind: ButtonBinding,
}

impl ValidateMappingConfig for MappingCancelCast {}

pub fn handle_cancel_cast(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    mask_size: Res<MaskSize>,
    runtime: ResMut<TokioTasksRuntime>,
    mut active_cast: ResMut<ActiveCastSpell>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("CancelCast") {
                let mapping = mapping.as_ref_cancelcast();
                if ineffable.just_pulsed(action.ineff_pulse()) {
                    // clear
                    if let Some(cast) = active_cast.0.take() {
                        let original_size: Vec2 = active_mapping.original_size.into();
                        let mut cancel_pos: Vec2 = mapping.position.into();
                        let cur_mask_size = mask_size.0;
                        let current_pos = cast.current_pos;

                        cancel_pos = cancel_pos / original_size * cur_mask_size; // relative to mask

                        let cs_tx = cs_tx_res.0.clone();
                        let pointer_id = cast.pointer_id;
                        let cast_block_direction_pad = cast.block_direction_pad;
                        let cast_initial_swipe_done = cast.initial_swipe_done.clone();
                        runtime.spawn_background_task(move |mut ctx| async move {
                            while !cast_initial_swipe_done.load(Ordering::Relaxed) {
                                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                            }
                            let move_points = build_single_segment_swipe_intermediate_points(
                                current_pos,
                                cancel_pos,
                                SingleSwipeStrategy::Linear,
                                0,
                            );
                            let mut end_pos = current_pos;
                            for point in move_points {
                                ControlMsgHelper::send_touch(
                                    &cs_tx,
                                    MotionEventAction::Move,
                                    pointer_id,
                                    cur_mask_size,
                                    point.pos,
                                );
                                tokio::time::sleep(std::time::Duration::from_millis(point.wait_ms))
                                    .await;
                                end_pos = point.pos;
                            }

                            // stay at the end
                            let steps: u64 = 10;
                            let step_interval = CAST_SPELL_DELAY / steps;
                            for _ in 0..steps {
                                end_pos.x += 5.;
                                ControlMsgHelper::send_touch(
                                    &cs_tx,
                                    MotionEventAction::Move,
                                    pointer_id,
                                    cur_mask_size,
                                    end_pos,
                                );
                                tokio::time::sleep(std::time::Duration::from_millis(step_interval))
                                    .await;
                            }

                            ControlMsgHelper::send_touch(
                                &cs_tx,
                                MotionEventAction::Up,
                                pointer_id,
                                cur_mask_size,
                                cancel_pos,
                            );

                            if cast_block_direction_pad {
                                ctx.run_on_main_thread(move |ctx| {
                                    let mut block_direction_pad =
                                        ctx.world.resource_mut::<BlockDirectionPad>();
                                    block_direction_pad.0 = false;
                                })
                                .await;
                            }
                        });
                    }
                }
            }
        }
    }
}
