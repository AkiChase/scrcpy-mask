use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*, window::CursorGrabMode};

use crate::{
    mask::{
        mapping::{MappingState, utils::ControlMsgHelper},
        mask_command::MaskSize,
    },
    scrcpy::constant::MotionEventAction,
    utils::ChannelSenderCS,
};

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
pub enum CursorState {
    #[default]
    Normal,
    Fps,
}

#[derive(Resource)]
pub struct CursorPosition(pub Vec2);

pub struct CursorPlugins;

impl Plugin for CursorPlugins {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorPosition((0., 0.).into()))
            .insert_state(CursorState::Normal)
            .insert_resource(IgnoreFirstMotion(false))
            .insert_resource(ActiveCursorFpsConfig::default())
            .add_systems(
                Update,
                (
                    handle_cursor_normal.run_if(
                        not(in_state(MappingState::Stop)).and(in_state(CursorState::Normal)),
                    ),
                    handle_cursor_fps.run_if(
                        in_state(CursorState::Fps)
                            .and(in_state(MappingState::Normal))
                            .and(run_if_handle_cursor_fps),
                    ),
                ),
            )
            .add_systems(
                Update,
                handle_normal_left_click
                    .run_if(not(in_state(MappingState::Stop)).and(in_state(CursorState::Normal))),
            )
            .add_systems(OnEnter(CursorState::Fps), on_enter_cursor_fps)
            .add_systems(OnExit(CursorState::Fps), on_exit_cursor_fps);
    }
}

#[derive(Resource, Default)]
pub struct ActiveCursorFpsConfig {
    pub ignore_fps_motion: bool,
    pub sensitivity: Vec2,
    pub pointer_id: u64,
    pub original_pos: Vec2,
    pub original_size: Vec2,
}

fn handle_cursor_normal(
    accumulated_motion: Res<AccumulatedMouseMotion>,
    window: Single<&Window>,
    mut cursor_pos: ResMut<CursorPosition>,
) {
    let mut new_pos = cursor_pos.0;
    if let Some(pos) = window.cursor_position() {
        new_pos = pos;
    } else {
        new_pos += accumulated_motion.delta;
    }
    if new_pos != cursor_pos.0 {
        cursor_pos.0 = new_pos;
    }
}

fn on_enter_cursor_fps(
    mut window: Single<&mut Window>,
    mut cursor_pos: ResMut<CursorPosition>,
    mut ignore_first_motion: ResMut<IgnoreFirstMotion>,
    fps_config: Res<ActiveCursorFpsConfig>,
    mask_size: Res<MaskSize>,
) {
    let center_pos = fps_config.original_pos / fps_config.original_size * mask_size.0;

    window.cursor_options.grab_mode = CursorGrabMode::Locked;
    window.cursor_options.visible = false;

    if window.cursor_position().is_none() {
        window.set_cursor_position(Some(center_pos));
        ignore_first_motion.0 = true;
    }

    cursor_pos.0 = center_pos;
}

fn on_exit_cursor_fps(
    mut window: Single<&mut Window>,
    mut cursor_pos: ResMut<CursorPosition>,
    fps_config: Res<ActiveCursorFpsConfig>,
    mask_size: Res<MaskSize>,
) {
    let center_pos = fps_config.original_pos / fps_config.original_size * mask_size.0;

    window.set_cursor_position(Some(center_pos));
    cursor_pos.0 = center_pos;
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
}

#[derive(Resource)]
struct IgnoreFirstMotion(bool);

pub const FPS_MARGIN: f32 = 25.;

fn run_if_handle_cursor_fps(
    window: Single<&Window>,
    fps_config: Res<ActiveCursorFpsConfig>,
) -> bool {
    // fire key is not pressed and window is focused
    !fps_config.ignore_fps_motion && window.focused
}

fn handle_cursor_fps(
    accumulated_motion: Res<AccumulatedMouseMotion>,
    mut cursor_pos: ResMut<CursorPosition>,
    fps_config: Res<ActiveCursorFpsConfig>,
    mut ignore_first_motion: ResMut<IgnoreFirstMotion>,
    mask_size: Res<MaskSize>,
    cs_tx_res: Res<ChannelSenderCS>,
) {
    if accumulated_motion.delta.x == 0. && accumulated_motion.delta.y == 0. {
        return;
    }

    if ignore_first_motion.0 {
        ignore_first_motion.0 = false;
        return;
    }

    let mut new_pos = cursor_pos.0 + accumulated_motion.delta * fps_config.sensitivity;

    let is_out_of_bounds = |pos: Vec2| -> bool {
        pos.x < FPS_MARGIN
            || pos.x > mask_size.0.x - FPS_MARGIN
            || pos.y < FPS_MARGIN
            || pos.y > mask_size.0.y - FPS_MARGIN
    };

    if is_out_of_bounds(new_pos) {
        let center_pos = fps_config.original_pos / fps_config.original_size * mask_size.0;
        let mut delta = new_pos - cursor_pos.0;
        // move to the edge and touch up
        let edge_pos = Vec2::new(
            new_pos.x.clamp(FPS_MARGIN, mask_size.0.x - FPS_MARGIN),
            new_pos.y.clamp(FPS_MARGIN, mask_size.0.y - FPS_MARGIN),
        );
        delta -= edge_pos - cursor_pos.0;
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Move,
            fps_config.pointer_id,
            mask_size.0,
            edge_pos,
        );
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Up,
            fps_config.pointer_id,
            mask_size.0,
            edge_pos,
        );
        // touch down center
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Down,
            fps_config.pointer_id,
            mask_size.0,
            center_pos,
        );
        new_pos = center_pos + delta;
        if is_out_of_bounds(new_pos) {
            // still out of bounds
            // move to the edge and touch up
            let edge_pos = Vec2::new(
                new_pos.x.clamp(FPS_MARGIN, mask_size.0.x - FPS_MARGIN),
                new_pos.y.clamp(FPS_MARGIN, mask_size.0.y - FPS_MARGIN),
            );
            ControlMsgHelper::send_touch(
                &cs_tx_res.0,
                MotionEventAction::Move,
                fps_config.pointer_id,
                mask_size.0,
                edge_pos,
            );
            ControlMsgHelper::send_touch(
                &cs_tx_res.0,
                MotionEventAction::Up,
                fps_config.pointer_id,
                mask_size.0,
                edge_pos,
            );
            // touch down center
            ControlMsgHelper::send_touch(
                &cs_tx_res.0,
                MotionEventAction::Down,
                fps_config.pointer_id,
                mask_size.0,
                center_pos,
            );
            new_pos = edge_pos;
        } else {
            // move to finnal pos
            ControlMsgHelper::send_touch(
                &cs_tx_res.0,
                MotionEventAction::Move,
                fps_config.pointer_id,
                mask_size.0,
                new_pos,
            );
        }
    } else {
        // move to new_pos
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Move,
            fps_config.pointer_id,
            mask_size.0,
            new_pos,
        );
    }
    cursor_pos.0 = new_pos;
}

fn handle_normal_left_click(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorPosition>,
    cs_tx_res: Res<ChannelSenderCS>,
    mask_size: Res<MaskSize>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Down,
            0,
            mask_size.0,
            cursor_pos.0,
        );
        return;
    }

    if mouse_button_input.pressed(MouseButton::Left) {
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Move,
            0,
            mask_size.0,
            cursor_pos.0,
        );
        return;
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Up,
            0,
            mask_size.0,
            cursor_pos.0,
        );
        return;
    }
}
