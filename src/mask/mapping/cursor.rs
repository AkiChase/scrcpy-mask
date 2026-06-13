use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

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
    scrcpy::{constant::MotionEventAction, control_msg::ScrcpyControlMsg},
    utils::ChannelSenderCS,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FpsTouchMode {
    #[default]
    None,
    Clean {
        another_pointer_id: u64,
    },
    Delayed {
        interval: u64,
        another_pointer_id: u64,
    },
    Overlap {
        another_pointer_id: u64,
    },
}

impl FpsTouchMode {
    pub fn another_pointer_id(&self) -> Option<u64> {
        match self {
            FpsTouchMode::None => None,
            FpsTouchMode::Clean { another_pointer_id }
            | FpsTouchMode::Delayed {
                another_pointer_id, ..
            }
            | FpsTouchMode::Overlap { another_pointer_id } => Some(*another_pointer_id),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PendingFpsTouch {
    pointer_id: u64,
    pos: Vec2,
    release_at: Option<Instant>,
    overlap: bool,
    deferred_delta: Vec2,
}

#[derive(Resource)]
pub struct ActiveCursorFpsConfig {
    pub ignore_fps_motion: bool,
    pub sensitivity: Vec2,
    pub pointer_id: u64,
    pub active_pointer_id: u64,
    pub original_pos: Vec2,
    pub original_size: Vec2,
    pub max_offset: Vec2,
    pub touch_mode: FpsTouchMode,
    pending_touch: Option<PendingFpsTouch>,
}

impl Default for ActiveCursorFpsConfig {
    fn default() -> Self {
        Self {
            ignore_fps_motion: false,
            sensitivity: Vec2::ZERO,
            pointer_id: 0,
            active_pointer_id: 0,
            original_pos: Vec2::ZERO,
            original_size: Vec2::ZERO,
            max_offset: Vec2::splat(-1.0),
            touch_mode: FpsTouchMode::None,
            pending_touch: None,
        }
    }
}

impl ActiveCursorFpsConfig {
    pub fn reset_touch_state(&mut self) {
        self.active_pointer_id = self.pointer_id;
        self.pending_touch = None;
    }
}

pub fn release_fps_touches(
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    fps_config: &mut ActiveCursorFpsConfig,
    mask_size: Vec2,
    active_pos: Vec2,
) {
    ControlMsgHelper::send_touch(
        cs_tx,
        MotionEventAction::Up,
        fps_config.active_pointer_id,
        mask_size,
        active_pos,
    );
    if let Some(pending) = fps_config.pending_touch.take() {
        ControlMsgHelper::send_touch(
            cs_tx,
            MotionEventAction::Up,
            pending.pointer_id,
            mask_size,
            pending.pos,
        );
    }
    fps_config.reset_touch_state();
}

pub fn restore_fps_touch(
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    fps_config: &mut ActiveCursorFpsConfig,
) {
    fps_config.reset_touch_state();
    ControlMsgHelper::send_touch(
        cs_tx,
        MotionEventAction::Down,
        fps_config.active_pointer_id,
        fps_config.original_size,
        fps_config.original_pos,
    );
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

fn physical_bounds(mask_size: Vec2) -> (Vec2, Vec2) {
    let max_x = (mask_size.x - FPS_MARGIN).max(FPS_MARGIN);
    let max_y = (mask_size.y - FPS_MARGIN).max(FPS_MARGIN);
    (Vec2::splat(FPS_MARGIN), Vec2::new(max_x, max_y))
}

fn clamp_to_bounds(pos: Vec2, min: Vec2, max: Vec2) -> Vec2 {
    Vec2::new(pos.x.clamp(min.x, max.x), pos.y.clamp(min.y, max.y))
}

fn fps_center_pos(fps_config: &ActiveCursorFpsConfig, mask_size: Vec2) -> Vec2 {
    fps_config.original_pos / fps_config.original_size * mask_size
}

fn fps_effective_bounds(fps_config: &ActiveCursorFpsConfig, mask_size: Vec2) -> (Vec2, Vec2) {
    let (physical_min, physical_max) = physical_bounds(mask_size);
    let center = fps_center_pos(fps_config, mask_size);
    let scale = mask_size / fps_config.original_size;
    let mut min = physical_min;
    let mut max = physical_max;

    if fps_config.max_offset.x >= 0.0 {
        let offset = fps_config.max_offset.x * scale.x;
        min.x = (center.x - offset).max(physical_min.x);
        max.x = (center.x + offset).min(physical_max.x);
    }
    if fps_config.max_offset.y >= 0.0 {
        let offset = fps_config.max_offset.y * scale.y;
        min.y = (center.y - offset).max(physical_min.y);
        max.y = (center.y + offset).min(physical_max.y);
    }

    (min, max)
}

fn is_out_of_bounds(pos: Vec2, min: Vec2, max: Vec2) -> bool {
    pos.x <= min.x || pos.x >= max.x || pos.y <= min.y || pos.y >= max.y
}

fn send_fps_touch(
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    action: MotionEventAction,
    pointer_id: u64,
    mask_size: Vec2,
    pos: Vec2,
) {
    ControlMsgHelper::send_touch(cs_tx, action, pointer_id, mask_size, pos);
}

fn cleanup_pending_fps_touch(
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    fps_config: &mut ActiveCursorFpsConfig,
    mask_size: Vec2,
) -> Option<Vec2> {
    let should_release = fps_config
        .pending_touch
        .is_some_and(|pending| pending.release_at.is_some_and(|at| Instant::now() >= at));
    if should_release {
        if let Some(pending) = fps_config.pending_touch.take() {
            send_fps_touch(
                cs_tx,
                MotionEventAction::Up,
                pending.pointer_id,
                mask_size,
                pending.pos,
            );
            return Some(pending.deferred_delta);
        }
    }
    None
}

fn consume_overlap_fps_touch(
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    fps_config: &mut ActiveCursorFpsConfig,
    mask_size: Vec2,
    delta: Vec2,
) -> Option<Vec2> {
    let Some(pending) = fps_config.pending_touch else {
        return None;
    };
    if !pending.overlap {
        return None;
    }

    let (physical_min, physical_max) = physical_bounds(mask_size);
    let pos = clamp_to_bounds(pending.pos + delta, physical_min, physical_max);
    send_fps_touch(
        cs_tx,
        MotionEventAction::Move,
        pending.pointer_id,
        mask_size,
        pos,
    );
    send_fps_touch(
        cs_tx,
        MotionEventAction::Up,
        pending.pointer_id,
        mask_size,
        pos,
    );
    fps_config.pending_touch = None;
    Some(pending.deferred_delta + delta)
}

fn alternate_fps_pointer_id(fps_config: &ActiveCursorFpsConfig) -> u64 {
    let Some(another_pointer_id) = fps_config.touch_mode.another_pointer_id() else {
        return fps_config.pointer_id;
    };
    if fps_config.active_pointer_id == fps_config.pointer_id {
        another_pointer_id
    } else {
        fps_config.pointer_id
    }
}

fn send_fps_down_and_optional_move(
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    pointer_id: u64,
    mask_size: Vec2,
    center_pos: Vec2,
    new_pos: Vec2,
) {
    send_fps_touch(
        cs_tx,
        MotionEventAction::Down,
        pointer_id,
        mask_size,
        center_pos,
    );
    if new_pos != center_pos {
        send_fps_touch(
            cs_tx,
            MotionEventAction::Move,
            pointer_id,
            mask_size,
            new_pos,
        );
    }
}

fn recenter_fps_touch(
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    fps_config: &mut ActiveCursorFpsConfig,
    mask_size: Vec2,
    old_pos: Vec2,
    physical_overflow: Vec2,
) -> Vec2 {
    let center_pos = fps_center_pos(fps_config, mask_size);
    let (physical_min, physical_max) = physical_bounds(mask_size);
    let new_pos = clamp_to_bounds(center_pos + physical_overflow, physical_min, physical_max);

    if let Some(pending) = fps_config.pending_touch.take() {
        send_fps_touch(
            cs_tx,
            MotionEventAction::Up,
            pending.pointer_id,
            mask_size,
            pending.pos,
        );
    }

    match fps_config.touch_mode {
        FpsTouchMode::None => {
            send_fps_touch(
                cs_tx,
                MotionEventAction::Up,
                fps_config.active_pointer_id,
                mask_size,
                old_pos,
            );
            send_fps_down_and_optional_move(
                cs_tx,
                fps_config.active_pointer_id,
                mask_size,
                center_pos,
                new_pos,
            );
        }
        FpsTouchMode::Clean { .. } => {
            let old_pointer_id = fps_config.active_pointer_id;
            let new_pointer_id = alternate_fps_pointer_id(fps_config);
            send_fps_touch(
                cs_tx,
                MotionEventAction::Down,
                new_pointer_id,
                mask_size,
                center_pos,
            );
            send_fps_touch(
                cs_tx,
                MotionEventAction::Up,
                old_pointer_id,
                mask_size,
                old_pos,
            );
            if new_pos != center_pos {
                send_fps_touch(
                    cs_tx,
                    MotionEventAction::Move,
                    new_pointer_id,
                    mask_size,
                    new_pos,
                );
            }
            fps_config.active_pointer_id = new_pointer_id;
        }
        FpsTouchMode::Delayed { interval, .. } => {
            let old_pointer_id = fps_config.active_pointer_id;
            let new_pointer_id = alternate_fps_pointer_id(fps_config);
            send_fps_touch(
                cs_tx,
                MotionEventAction::Down,
                new_pointer_id,
                mask_size,
                center_pos,
            );
            fps_config.pending_touch = Some(PendingFpsTouch {
                pointer_id: old_pointer_id,
                pos: old_pos,
                release_at: Some(Instant::now() + Duration::from_millis(interval)),
                overlap: false,
                deferred_delta: physical_overflow,
            });
            fps_config.active_pointer_id = new_pointer_id;
            return center_pos;
        }
        FpsTouchMode::Overlap { .. } => {
            let old_pointer_id = fps_config.active_pointer_id;
            let new_pointer_id = alternate_fps_pointer_id(fps_config);
            send_fps_touch(
                cs_tx,
                MotionEventAction::Down,
                new_pointer_id,
                mask_size,
                center_pos,
            );
            fps_config.pending_touch = Some(PendingFpsTouch {
                pointer_id: old_pointer_id,
                pos: old_pos,
                release_at: None,
                overlap: true,
                deferred_delta: physical_overflow,
            });
            fps_config.active_pointer_id = new_pointer_id;
            return center_pos;
        }
    }

    new_pos
}

fn apply_fps_delta(
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    fps_config: &mut ActiveCursorFpsConfig,
    mask_size: Vec2,
    cursor_pos: Vec2,
    delta: Vec2,
) -> Vec2 {
    if delta == Vec2::ZERO {
        return cursor_pos;
    }

    let raw_pos = cursor_pos + delta;
    let (bounds_min, bounds_max) = fps_effective_bounds(fps_config, mask_size);
    let should_recenter = is_out_of_bounds(raw_pos, bounds_min, bounds_max);
    let (physical_min, physical_max) = physical_bounds(mask_size);
    let new_pos = clamp_to_bounds(raw_pos, physical_min, physical_max);
    let physical_overflow = raw_pos - new_pos;
    send_fps_touch(
        cs_tx,
        MotionEventAction::Move,
        fps_config.active_pointer_id,
        mask_size,
        new_pos,
    );

    if should_recenter {
        recenter_fps_touch(cs_tx, fps_config, mask_size, new_pos, physical_overflow)
    } else {
        new_pos
    }
}

fn handle_cursor_fps(
    accumulated_motion: Res<AccumulatedMouseMotion>,
    mut cursor_pos: ResMut<CursorPosition>,
    mut fps_config: ResMut<ActiveCursorFpsConfig>,
    mut ignore_first_motion: ResMut<IgnoreFirstMotion>,
    mask_size: Res<MaskSize>,
    cs_tx_res: Res<ChannelSenderCS>,
) {
    if ignore_first_motion.0 {
        ignore_first_motion.0 = false;
        return;
    }

    let delta = accumulated_motion.delta * fps_config.sensitivity;

    if let Some(deferred_delta) =
        cleanup_pending_fps_touch(&cs_tx_res.0, &mut fps_config, mask_size.0)
    {
        cursor_pos.0 = apply_fps_delta(
            &cs_tx_res.0,
            &mut fps_config,
            mask_size.0,
            cursor_pos.0,
            deferred_delta + delta,
        );
        return;
    }

    if let Some(pending) = fps_config.pending_touch.as_mut()
        && !pending.overlap
    {
        pending.deferred_delta += delta;
        return;
    }

    if let Some(overlap_delta) =
        consume_overlap_fps_touch(&cs_tx_res.0, &mut fps_config, mask_size.0, delta)
    {
        cursor_pos.0 = apply_fps_delta(
            &cs_tx_res.0,
            &mut fps_config,
            mask_size.0,
            cursor_pos.0,
            overlap_delta,
        );
        return;
    }

    cursor_pos.0 = apply_fps_delta(
        &cs_tx_res.0,
        &mut fps_config,
        mask_size.0,
        cursor_pos.0,
        delta,
    );
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
