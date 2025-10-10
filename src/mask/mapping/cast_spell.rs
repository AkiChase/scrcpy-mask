use std::time::{Duration, Instant};

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
use tokio::time::sleep;

use crate::{
    mask::{
        mapping::{
            binding::{ButtonBinding, DirectionBinding, ValidateMappingConfig},
            config::{ActiveMappingConfig, MappingAction},
            cursor::CursorPosition,
            direction_pad::{BlockDirectionPad, DirectionPadMap},
            utils::{ControlMsgHelper, MIN_MOVE_STEP_LENGTH, Position},
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
    enable_instant: Instant,
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
}

impl ActiveCastSpellItem {
    fn new_mouse_item(
        key: String,
        pointer_id: u64,
        current_pos: Vec2,
        original_size: Vec2,
        cast_pos: Vec2,
        drag_radius: f32,
        enable_instant: Instant,
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
            enable_instant,
            center_pos,
            cast_radius,
            horizontal_scale_factor,
            vertical_scale_factor,
            cast_no_direction,
            pad_action: None,
            last_state: Vec2::ZERO,
            block_direction_pad: false,
        }
    }

    fn new_pad_item(
        key: String,
        pointer_id: u64,
        current_pos: Vec2,
        original_size: Vec2,
        cast_pos: Vec2,
        drag_radius: f32,
        enable_instant: Instant,
        block_direction_pad: bool,
        pad_action: MappingAction,
    ) -> Self {
        Self {
            mouse_flag: false,
            key,
            pointer_id,
            current_pos,
            original_size,
            cast_pos,
            drag_radius,
            enable_instant,
            center_pos: Vec2::ZERO,
            cast_radius: 0.,
            horizontal_scale_factor: 0.,
            vertical_scale_factor: 0.,
            cast_no_direction: false,
            pad_action: Some(pad_action),
            last_state: Vec2::ZERO,
            block_direction_pad,
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
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
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
            bind: value.bind.clone(),
            input_binding: ContinuousBinding::hold(value.bind).0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingMouseCastSpell {
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
    pub bind: ButtonBinding,
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
            || active_cast.enable_instant > Instant::now()
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
                    let center_pos: Vec2 = mapping.center.into();
                    let release_mode = mapping.release_mode.clone();
                    let cast_no_direction = mapping.cast_no_direction;
                    let cast_radius = mapping.cast_radius;
                    let drag_radius = mapping.drag_radius;
                    let horizontal_scale_factor = mapping.horizontal_scale_factor;
                    let vertical_scale_factor = mapping.vertical_scale_factor;
                    let mut current_pos = original_pos / original_size * cur_mask_size;

                    if !matches!(mapping.release_mode, MouseCastReleaseMode::OnPress) {
                        // set active
                        let enable_instant =
                            Instant::now() + Duration::from_millis(CAST_SPELL_DELAY * 2);

                        let record_current_pos = if !cast_no_direction {
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

                        active_cast.0 = Some(ActiveCastSpellItem::new_mouse_item(
                            action.to_string(),
                            pointer_id,
                            record_current_pos,
                            original_size,
                            original_pos,
                            mapping.drag_radius,
                            enable_instant,
                            center_pos,
                            mapping.cast_radius,
                            mapping.horizontal_scale_factor,
                            mapping.vertical_scale_factor,
                            mapping.cast_no_direction,
                        ))
                    }

                    // touch down new cast
                    ControlMsgHelper::send_touch(
                        &cs_tx_res.0,
                        MotionEventAction::Down,
                        pointer_id,
                        mask_size.0,
                        current_pos,
                    );

                    let cs_tx = cs_tx_res.0.clone();
                    runtime.spawn_background_task(move |_ctx| async move {
                        // stay at the center
                        let steps: u64 = 5;
                        let step_interval = CAST_SPELL_DELAY / steps;
                        let mut delta = Vec2::new(0., 0.);
                        for _ in 0..steps {
                            ControlMsgHelper::send_touch(
                                &cs_tx,
                                MotionEventAction::Move,
                                pointer_id,
                                cur_mask_size,
                                current_pos + delta,
                            );
                            delta += Vec2::new(1., -1.);
                            sleep(Duration::from_millis(step_interval)).await;
                        }

                        if !cast_no_direction {
                            // move to direction
                            let new_pos = cal_mouse_cast_spell_current_pos(
                                cur_cursor_pos,
                                center_pos,
                                original_pos,
                                cast_radius,
                                drag_radius,
                                cur_mask_size,
                                original_size,
                                horizontal_scale_factor,
                                vertical_scale_factor,
                            );
                            let delta = new_pos - current_pos;
                            let steps = std::cmp::max(
                                2, // at least 2 steps
                                (delta.length() / MIN_MOVE_STEP_LENGTH).ceil() as i32,
                            );
                            for step in 1..=steps {
                                let linear_t = step as f32 / steps as f32;
                                let interp = current_pos + delta * linear_t;
                                ControlMsgHelper::send_touch(
                                    &cs_tx,
                                    MotionEventAction::Move,
                                    pointer_id,
                                    cur_mask_size,
                                    interp,
                                );
                            }
                            current_pos = new_pos;
                        }

                        // for OnPress cast, touch up here
                        if matches!(release_mode, MouseCastReleaseMode::OnPress) {
                            sleep(Duration::from_millis(CAST_SPELL_DELAY)).await;

                            ControlMsgHelper::send_touch(
                                &cs_tx,
                                MotionEventAction::Up,
                                pointer_id,
                                cur_mask_size,
                                current_pos,
                            );
                        }
                    });
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
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingPadCastSpell {
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub release_mode: PadCastReleaseMode,
    pub drag_radius: f32,
    pub block_direction_pad: bool,
    pub pad_bind: DirectionBinding,
    pub bind: ButtonBinding,
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
    mut active_cast: ResMut<ActiveCastSpell>,
) {
    if let Some(active_cast) = active_cast.0.as_mut() {
        if active_cast.mouse_flag || active_cast.enable_instant > Instant::now() {
            return;
        }

        let state = scale_direction_2d_state(
            ineffable.direction_2d(active_cast.pad_action.as_ref().unwrap().ineff_dual_axis()),
            active_cast.drag_radius,
        );

        if state != active_cast.last_state {
            // move to new state
            ControlMsgHelper::send_touch(
                &cs_tx_res.0,
                MotionEventAction::Move,
                active_cast.pointer_id,
                active_cast.original_size,
                active_cast.cast_pos + state,
            );
            // record last state
            active_cast.last_state = state;
        }
    }
}

pub fn handle_pad_cast_spell(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    mask_size: Res<MaskSize>,
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
                    let current_pos = original_pos / original_size * mask_size.0;
                    let enable_instant = Instant::now() + Duration::from_millis(CAST_SPELL_DELAY);

                    // set active
                    active_cast.0 = Some(ActiveCastSpellItem::new_pad_item(
                        action.to_string(),
                        pointer_id,
                        current_pos,
                        original_size,
                        original_pos,
                        mapping.drag_radius,
                        enable_instant,
                        mapping.block_direction_pad,
                        mapping.pad_action.clone(),
                    ));

                    // touch down new cast
                    ControlMsgHelper::send_touch(
                        &cs_tx_res.0,
                        MotionEventAction::Down,
                        pointer_id,
                        mask_size.0,
                        current_pos,
                    );
                    // touch move around current_pos
                    let steps: u64 = 5;
                    let mut delta = Vec2::new(0., 0.);
                    for _ in 0..steps {
                        ControlMsgHelper::send_touch(
                            &cs_tx_res.0,
                            MotionEventAction::Move,
                            pointer_id,
                            mask_size.0,
                            current_pos + delta,
                        );
                        delta += Vec2::new(1., -1.);
                    }
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
                        let delta = cancel_pos - current_pos;
                        let steps = std::cmp::min(
                            5, // at most 5 steps
                            (delta.length() / MIN_MOVE_STEP_LENGTH).ceil() as i32,
                        );
                        let cs_tx = cs_tx_res.0.clone();
                        let pointer_id = cast.pointer_id;
                        let cast_block_direction_pad = cast.block_direction_pad;
                        let cast_enable_instant = cast.enable_instant;
                        runtime.spawn_background_task(move |mut ctx| async move {
                            let now = Instant::now();
                            if cast_enable_instant > now {
                                sleep(cast_enable_instant - now).await;
                            }

                            let mut end_pos = current_pos;

                            for step in 1..=steps {
                                let linear_t = step as f32 / steps as f32;
                                let interp = current_pos + delta * linear_t;
                                ControlMsgHelper::send_touch(
                                    &cs_tx,
                                    MotionEventAction::Move,
                                    pointer_id,
                                    cur_mask_size,
                                    interp,
                                );
                                end_pos = interp;
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
                                sleep(Duration::from_millis(step_interval)).await;
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
