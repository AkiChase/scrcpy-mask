use std::collections::HashSet;

use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

use crate::{
    mask::{
        mapping::{MappingState, mask_not_resizing, utils::ControlMsgHelper},
        mask_command::{MaskSize, TitlebarState},
        ui::basic::{MaskContentEntity, TITLEBAR_HEIGHT},
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

#[derive(Resource, Default)]
pub struct NormalCursorCapture {
    owners: HashSet<String>,
    grabbed: bool,
    skip_next_nonzero_motion: bool,
}

impl NormalCursorCapture {
    pub fn request(&mut self, owner: impl Into<String>) {
        self.owners.insert(owner.into());
    }

    pub fn release(&mut self, owner: &str) {
        self.owners.remove(owner);
    }

    pub fn is_active(&self) -> bool {
        !self.owners.is_empty()
    }

    fn reset_grab(&mut self) {
        self.grabbed = false;
        self.skip_next_nonzero_motion = false;
    }
}

#[derive(Component)]
struct VirtualCursor;

const VIRTUAL_CURSOR_SIZE: f32 = 24.0;
const VIRTUAL_CURSOR_CENTER: f32 = VIRTUAL_CURSOR_SIZE / 2.0;
const SYSTEM_CURSOR_RESTORE_INSET: f32 = 1.0;

#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CursorFrameSet {
    UpdatePosition,
    HandleMappings,
    ApplyCapture,
    SyncVirtualCursor,
}

pub struct CursorPlugins;

impl Plugin for CursorPlugins {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorPosition((0., 0.).into()))
            .insert_resource(NormalCursorCapture::default())
            .insert_state(CursorState::Normal)
            .insert_resource(IgnoreFirstMotion(false))
            .insert_resource(ActiveCursorFpsConfig::default())
            .add_systems(
                Update,
                handle_cursor_normal
                    .in_set(CursorFrameSet::UpdatePosition)
                    .run_if(
                        not(in_state(MappingState::Stop)).and_then(in_state(CursorState::Normal)),
                    ),
            )
            .add_systems(
                Update,
                sync_normal_cursor_capture_window
                    .in_set(CursorFrameSet::ApplyCapture)
                    .run_if(in_state(CursorState::Normal)),
            )
            .add_systems(
                Update,
                sync_virtual_cursor
                    .in_set(CursorFrameSet::SyncVirtualCursor)
                    .run_if(in_state(CursorState::Normal)),
            )
            .add_systems(
                Update,
                handle_normal_left_click
                    .in_set(CursorFrameSet::HandleMappings)
                    .run_if(
                        not(in_state(MappingState::Stop)).and_then(in_state(CursorState::Normal)),
                    ),
            )
            .add_systems(
                Update,
                handle_cursor_fps
                    .in_set(CursorFrameSet::UpdatePosition)
                    .run_if(
                        in_state(CursorState::Fps)
                            .and_then(in_state(MappingState::Normal))
                            .and_then(run_if_handle_cursor_fps)
                            .and_then(mask_not_resizing),
                    ),
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
    titlebar_state: Res<TitlebarState>,
    mut normal_capture: ResMut<NormalCursorCapture>,
    mask_size: Res<MaskSize>,
) {
    let mut new_pos = cursor_pos.0;
    if normal_capture.grabbed {
        if normal_capture.skip_next_nonzero_motion && accumulated_motion.delta != Vec2::ZERO {
            normal_capture.skip_next_nonzero_motion = false;
            new_pos = clamped_virtual_cursor_pos(new_pos, mask_size.0);
        } else {
            new_pos += accumulated_motion.delta;
            new_pos = clamped_virtual_cursor_pos(new_pos, mask_size.0);
        }
    } else if let Some(pos) = window.cursor_position() {
        new_pos = pos - Vec2::new(0., titlebar_state.offset());
    } else {
        new_pos += accumulated_motion.delta;
    }
    if new_pos != cursor_pos.0 {
        cursor_pos.0 = new_pos;
    }
}

fn sync_normal_cursor_capture_window(
    window: Single<(&mut Window, &mut CursorOptions)>,
    mut normal_capture: ResMut<NormalCursorCapture>,
    mut cursor_pos: ResMut<CursorPosition>,
    mask_size: Res<MaskSize>,
    titlebar_state: Res<TitlebarState>,
) {
    let (mut window, mut cursor_options) = window.into_inner();
    let active = normal_capture.is_active() && window.focused;

    if active && !normal_capture.grabbed {
        if let Some(pos) = window.cursor_position() {
            cursor_pos.0 = clamped_virtual_cursor_pos(
                pos - Vec2::new(0., titlebar_state.offset()),
                mask_size.0,
            );
        }
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
        normal_capture.grabbed = true;
        normal_capture.skip_next_nonzero_motion = true;
        return;
    }

    if !active && normal_capture.grabbed {
        let restore_pos = clamped_system_cursor_restore_pos(cursor_pos.0, mask_size.0);
        cursor_pos.0 = restore_pos;
        cursor_options.grab_mode = CursorGrabMode::None;
        window.set_cursor_position(Some(restore_pos + Vec2::new(0., titlebar_state.offset())));
        cursor_options.visible = true;
        normal_capture.grabbed = false;
        normal_capture.skip_next_nonzero_motion = false;
    }
}

fn sync_virtual_cursor(
    mut commands: Commands,
    mask_content: Res<MaskContentEntity>,
    normal_capture: Res<NormalCursorCapture>,
    cursor_pos: Res<CursorPosition>,
    mask_size: Res<MaskSize>,
    mut query: Query<&mut Node, With<VirtualCursor>>,
) {
    if query.is_empty() {
        commands.entity(mask_content.0).with_children(|parent| {
            parent
                .spawn((
                    VirtualCursor,
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(VIRTUAL_CURSOR_SIZE),
                        height: Val::Px(VIRTUAL_CURSOR_SIZE),
                        display: Display::None,
                        ..default()
                    },
                    ZIndex(100),
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|cursor| {
                    let outline_color = BorderColor::all(Color::srgba(0.12, 0.12, 0.12, 0.78));
                    let gold_color = BorderColor::all(Color::srgba(0.93, 0.72, 0.26, 0.96));

                    cursor.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            width: Val::Px(24.0),
                            height: Val::Px(24.0),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(12.0)),
                            ..default()
                        },
                        outline_color,
                        BackgroundColor(Color::NONE),
                    ));
                    cursor.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(3.0),
                            top: Val::Px(3.0),
                            width: Val::Px(18.0),
                            height: Val::Px(18.0),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(9.0)),
                            ..default()
                        },
                        gold_color,
                        BackgroundColor(Color::NONE),
                    ));
                    cursor.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(VIRTUAL_CURSOR_CENTER - 3.0),
                            top: Val::Px(VIRTUAL_CURSOR_CENTER - 3.0),
                            width: Val::Px(6.0),
                            height: Val::Px(6.0),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                            ..default()
                        },
                        outline_color,
                        BackgroundColor(Color::srgba(0.12, 0.12, 0.12, 0.58)),
                    ));
                    cursor.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(VIRTUAL_CURSOR_CENTER - 2.0),
                            top: Val::Px(VIRTUAL_CURSOR_CENTER - 2.0),
                            width: Val::Px(4.0),
                            height: Val::Px(4.0),
                            border_radius: BorderRadius::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.93, 0.72, 0.26, 0.96)),
                    ));
                });
        });
        return;
    }

    let display_pos = clamped_virtual_cursor_pos(cursor_pos.0, mask_size.0);
    for mut node in query.iter_mut() {
        node.display = if normal_capture.is_active() {
            Display::Flex
        } else {
            Display::None
        };
        node.left = Val::Px(display_pos.x - VIRTUAL_CURSOR_SIZE / 2.0);
        node.top = Val::Px(display_pos.y - VIRTUAL_CURSOR_SIZE / 2.0);
    }
}

fn clamped_virtual_cursor_pos(pos: Vec2, mask_size: Vec2) -> Vec2 {
    Vec2::new(pos.x.clamp(0.0, mask_size.x), pos.y.clamp(0.0, mask_size.y))
}

fn clamped_system_cursor_restore_pos(pos: Vec2, mask_size: Vec2) -> Vec2 {
    let min = SYSTEM_CURSOR_RESTORE_INSET;
    let max_x = (mask_size.x - SYSTEM_CURSOR_RESTORE_INSET).max(0.0);
    let max_y = (mask_size.y - SYSTEM_CURSOR_RESTORE_INSET).max(0.0);
    let min_x = min.min(max_x);
    let min_y = min.min(max_y);
    Vec2::new(pos.x.clamp(min_x, max_x), pos.y.clamp(min_y, max_y))
}

fn on_enter_cursor_fps(
    window: Single<(&mut Window, &mut CursorOptions)>,
    mut cursor_pos: ResMut<CursorPosition>,
    mut ignore_first_motion: ResMut<IgnoreFirstMotion>,
    fps_config: Res<ActiveCursorFpsConfig>,
    mask_size: Res<MaskSize>,
    titlebar_state: Res<TitlebarState>,
    mut normal_capture: ResMut<NormalCursorCapture>,
) {
    let center_pos = fps_config.original_pos / fps_config.original_size * mask_size.0;
    let (mut window, mut cursor_options) = window.into_inner();

    normal_capture.reset_grab();
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;

    if window.cursor_position().is_none() {
        window.set_cursor_position(Some(center_pos + Vec2::new(0., titlebar_state.offset())));
        ignore_first_motion.0 = true;
    }

    cursor_pos.0 = center_pos;
}

fn on_exit_cursor_fps(
    window: Single<(&mut Window, &mut CursorOptions)>,
    mut cursor_pos: ResMut<CursorPosition>,
    fps_config: Res<ActiveCursorFpsConfig>,
    mask_size: Res<MaskSize>,
    titlebar_state: Res<TitlebarState>,
    mut normal_capture: ResMut<NormalCursorCapture>,
) {
    let center_pos = fps_config.original_pos / fps_config.original_size * mask_size.0;
    let (mut window, mut cursor_options) = window.into_inner();

    window.set_cursor_position(Some(center_pos + Vec2::new(0., titlebar_state.offset())));
    cursor_pos.0 = center_pos;
    cursor_options.grab_mode = CursorGrabMode::None;
    cursor_options.visible = true;
    normal_capture.reset_grab();
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
    titlebar_state: Res<TitlebarState>,
) {
    if titlebar_state.visible && cursor_pos.0.y < 0. && cursor_pos.0.y >= -TITLEBAR_HEIGHT {
        return;
    }
    if mouse_button_input.just_pressed(MouseButton::Left) {
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Down,
            0,
            mask_size.0,
            cursor_pos.0,
        );
        return;
    } else if mouse_button_input.pressed(MouseButton::Left) {
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Move,
            0,
            mask_size.0,
            cursor_pos.0,
        );
        return;
    } else if mouse_button_input.just_released(MouseButton::Left) {
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
