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

    pub fn clear(&mut self) {
        self.owners.clear();
        self.reset_grab();
    }

    fn reset_grab(&mut self) {
        self.grabbed = false;
        self.skip_next_nonzero_motion = false;
    }
}

#[derive(Component)]
struct VirtualCursor;

const VIRTUAL_CURSOR_SIZE: f32 = 48.0;
const VIRTUAL_CURSOR_CENTER: f32 = VIRTUAL_CURSOR_SIZE / 2.0;
const VIRTUAL_CURSOR_OUTER_BORDER: f32 = 4.0;
const VIRTUAL_CURSOR_INNER_INSET: f32 = 6.0;
const VIRTUAL_CURSOR_INNER_SIZE: f32 = VIRTUAL_CURSOR_SIZE - VIRTUAL_CURSOR_INNER_INSET * 2.0;
const VIRTUAL_CURSOR_CENTER_RING_SIZE: f32 = 12.0;
const VIRTUAL_CURSOR_CENTER_DOT_SIZE: f32 = 8.0;
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

pub fn cleanup_cursor_capture_on_stop(mut normal_capture: ResMut<NormalCursorCapture>) {
    normal_capture.clear();
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FpsTouchMode {
    Single {
        interval: u64,
    },
    Dual {
        another_pointer_id: u64,
        #[serde(flatten)]
        strategy: FpsDualTouchStrategy,
    },
}

impl Default for FpsTouchMode {
    fn default() -> Self {
        Self::Single { interval: 0 }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "strategy", rename_all = "snake_case")]
pub enum FpsDualTouchStrategy {
    Delay { interval: u64 },
    Overlap,
}

impl FpsTouchMode {
    pub fn another_pointer_id(&self) -> Option<u64> {
        match self {
            FpsTouchMode::Single { .. } => None,
            FpsTouchMode::Dual {
                another_pointer_id, ..
            } => Some(*another_pointer_id),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum PendingFpsTouch {
    Restore {
        pointer_id: u64,
        pos: Vec2,
        down_at: Instant,
        deferred_delta: Vec2,
    },
    Release {
        pointer_id: u64,
        pos: Vec2,
        release_at: Instant,
        deferred_delta: Vec2,
    },
    Overlap {
        pointer_id: u64,
        pos: Vec2,
        deferred_delta: Vec2,
    },
}

impl PendingFpsTouch {
    fn add_deferred_delta(&mut self, delta: Vec2) {
        match self {
            PendingFpsTouch::Restore { deferred_delta, .. }
            | PendingFpsTouch::Release { deferred_delta, .. }
            | PendingFpsTouch::Overlap { deferred_delta, .. } => *deferred_delta += delta,
        }
    }
}

#[derive(Resource)]
pub struct ActiveCursorFpsConfig {
    pub touch_active: bool,
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
            touch_active: false,
            ignore_fps_motion: false,
            sensitivity: Vec2::ZERO,
            pointer_id: 0,
            active_pointer_id: 0,
            original_pos: Vec2::ZERO,
            original_size: Vec2::ZERO,
            max_offset: Vec2::ZERO,
            touch_mode: FpsTouchMode::default(),
            pending_touch: None,
        }
    }
}

impl ActiveCursorFpsConfig {
    pub fn reset_touch_state(&mut self) {
        self.active_pointer_id = self.pointer_id;
        self.pending_touch = None;
    }

    pub fn clear_runtime_state(&mut self) {
        self.touch_active = false;
        self.ignore_fps_motion = false;
        self.reset_touch_state();
    }
}

pub fn release_fps_touches(
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    fps_config: &mut ActiveCursorFpsConfig,
    mask_size: Vec2,
    active_pos: Vec2,
) {
    if fps_config.touch_active {
        ControlMsgHelper::send_touch(
            cs_tx,
            MotionEventAction::Up,
            fps_config.active_pointer_id,
            mask_size,
            active_pos,
        );
        fps_config.touch_active = false;
    }
    if let Some(pending) = fps_config.pending_touch.take() {
        match pending {
            PendingFpsTouch::Restore { .. } => {}
            PendingFpsTouch::Release {
                pointer_id, pos, ..
            }
            | PendingFpsTouch::Overlap {
                pointer_id, pos, ..
            } => {
                ControlMsgHelper::send_touch(
                    cs_tx,
                    MotionEventAction::Up,
                    pointer_id,
                    mask_size,
                    pos,
                );
            }
        }
    }
    fps_config.reset_touch_state();
}

pub fn restore_fps_touch(
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    fps_config: &mut ActiveCursorFpsConfig,
) {
    fps_config.reset_touch_state();
    fps_config.touch_active = true;
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
                            width: Val::Px(VIRTUAL_CURSOR_SIZE),
                            height: Val::Px(VIRTUAL_CURSOR_SIZE),
                            border: UiRect::all(Val::Px(VIRTUAL_CURSOR_OUTER_BORDER)),
                            border_radius: BorderRadius::all(Val::Px(VIRTUAL_CURSOR_CENTER)),
                            ..default()
                        },
                        outline_color,
                        BackgroundColor(Color::NONE),
                    ));
                    cursor.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(VIRTUAL_CURSOR_INNER_INSET),
                            top: Val::Px(VIRTUAL_CURSOR_INNER_INSET),
                            width: Val::Px(VIRTUAL_CURSOR_INNER_SIZE),
                            height: Val::Px(VIRTUAL_CURSOR_INNER_SIZE),
                            border: UiRect::all(Val::Px(VIRTUAL_CURSOR_OUTER_BORDER)),
                            border_radius: BorderRadius::all(Val::Px(
                                VIRTUAL_CURSOR_INNER_SIZE / 2.0,
                            )),
                            ..default()
                        },
                        gold_color,
                        BackgroundColor(Color::NONE),
                    ));
                    cursor.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(
                                VIRTUAL_CURSOR_CENTER - VIRTUAL_CURSOR_CENTER_RING_SIZE / 2.0,
                            ),
                            top: Val::Px(
                                VIRTUAL_CURSOR_CENTER - VIRTUAL_CURSOR_CENTER_RING_SIZE / 2.0,
                            ),
                            width: Val::Px(VIRTUAL_CURSOR_CENTER_RING_SIZE),
                            height: Val::Px(VIRTUAL_CURSOR_CENTER_RING_SIZE),
                            border: UiRect::all(Val::Px(1.5)),
                            border_radius: BorderRadius::all(Val::Px(
                                VIRTUAL_CURSOR_CENTER_RING_SIZE / 2.0,
                            )),
                            ..default()
                        },
                        outline_color,
                        BackgroundColor(Color::srgba(0.12, 0.12, 0.12, 0.58)),
                    ));
                    cursor.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(
                                VIRTUAL_CURSOR_CENTER - VIRTUAL_CURSOR_CENTER_DOT_SIZE / 2.0,
                            ),
                            top: Val::Px(
                                VIRTUAL_CURSOR_CENTER - VIRTUAL_CURSOR_CENTER_DOT_SIZE / 2.0,
                            ),
                            width: Val::Px(VIRTUAL_CURSOR_CENTER_DOT_SIZE),
                            height: Val::Px(VIRTUAL_CURSOR_CENTER_DOT_SIZE),
                            border_radius: BorderRadius::all(Val::Px(
                                VIRTUAL_CURSOR_CENTER_DOT_SIZE / 2.0,
                            )),
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
const FPS_MAX_RECENTER_ITERATIONS: usize = 4;

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

    if fps_config.max_offset.x > 0.0 {
        let offset = fps_config.max_offset.x * scale.x;
        min.x = (center.x - offset).max(physical_min.x);
        max.x = (center.x + offset).min(physical_max.x);
    }
    if fps_config.max_offset.y > 0.0 {
        let offset = fps_config.max_offset.y * scale.y;
        min.y = (center.y - offset).max(physical_min.y);
        max.y = (center.y + offset).min(physical_max.y);
    }

    (min, max)
}

fn is_out_of_bounds(pos: Vec2, min: Vec2, max: Vec2) -> bool {
    pos.x <= min.x || pos.x >= max.x || pos.y <= min.y || pos.y >= max.y
}

fn first_bounds_crossing(
    cursor_pos: Vec2,
    delta: Vec2,
    min: Vec2,
    max: Vec2,
) -> Option<(Vec2, Vec2)> {
    let raw_pos = cursor_pos + delta;
    if !is_out_of_bounds(raw_pos, min, max) {
        return None;
    }

    let mut progress: f32 = 1.0;
    if delta.x > 0.0 && raw_pos.x >= max.x {
        progress = progress.min((max.x - cursor_pos.x) / delta.x);
    } else if delta.x < 0.0 && raw_pos.x <= min.x {
        progress = progress.min((min.x - cursor_pos.x) / delta.x);
    }
    if delta.y > 0.0 && raw_pos.y >= max.y {
        progress = progress.min((max.y - cursor_pos.y) / delta.y);
    } else if delta.y < 0.0 && raw_pos.y <= min.y {
        progress = progress.min((min.y - cursor_pos.y) / delta.y);
    }

    let progress = progress.clamp(0.0, 1.0);
    let boundary_pos = cursor_pos + delta * progress;
    let remaining_delta = delta * (1.0 - progress);
    Some((boundary_pos, remaining_delta))
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
    let now = Instant::now();
    let ready = fps_config
        .pending_touch
        .is_some_and(|pending| match pending {
            PendingFpsTouch::Restore { down_at, .. } => now >= down_at,
            PendingFpsTouch::Release { release_at, .. } => now >= release_at,
            PendingFpsTouch::Overlap { .. } => false,
        });
    if !ready {
        return None;
    }

    match fps_config.pending_touch.take() {
        Some(PendingFpsTouch::Restore {
            pointer_id,
            pos,
            deferred_delta,
            ..
        }) => {
            send_fps_touch(cs_tx, MotionEventAction::Down, pointer_id, mask_size, pos);
            fps_config.touch_active = true;
            Some(deferred_delta)
        }
        Some(PendingFpsTouch::Release {
            pointer_id,
            pos,
            deferred_delta,
            ..
        }) => {
            send_fps_touch(cs_tx, MotionEventAction::Up, pointer_id, mask_size, pos);
            Some(deferred_delta)
        }
        Some(PendingFpsTouch::Overlap { .. }) | None => None,
    }
}

fn consume_overlap_fps_touch(
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    fps_config: &mut ActiveCursorFpsConfig,
    mask_size: Vec2,
    delta: Vec2,
) -> Option<Vec2> {
    let Some(PendingFpsTouch::Overlap {
        pointer_id,
        pos,
        deferred_delta,
    }) = fps_config.pending_touch
    else {
        return None;
    };

    let (physical_min, physical_max) = physical_bounds(mask_size);
    let pos = clamp_to_bounds(pos + delta, physical_min, physical_max);
    send_fps_touch(cs_tx, MotionEventAction::Move, pointer_id, mask_size, pos);
    send_fps_touch(cs_tx, MotionEventAction::Up, pointer_id, mask_size, pos);
    fps_config.pending_touch = None;
    Some(deferred_delta + delta)
}

fn alternate_fps_pointer_id(fps_config: &ActiveCursorFpsConfig) -> u64 {
    let FpsTouchMode::Dual {
        another_pointer_id, ..
    } = fps_config.touch_mode
    else {
        return fps_config.active_pointer_id;
    };
    if fps_config.active_pointer_id == fps_config.pointer_id {
        another_pointer_id
    } else {
        fps_config.pointer_id
    }
}

enum FpsRecenterResult {
    Continue {
        cursor_pos: Vec2,
        remaining_delta: Vec2,
    },
    Stop(Vec2),
}

fn recenter_fps_touch(
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    fps_config: &mut ActiveCursorFpsConfig,
    mask_size: Vec2,
    old_pos: Vec2,
    remaining_delta: Vec2,
) -> FpsRecenterResult {
    let center_pos = fps_center_pos(fps_config, mask_size);

    if let Some(pending) = fps_config.pending_touch.take() {
        match pending {
            PendingFpsTouch::Restore { .. } => {}
            PendingFpsTouch::Release {
                pointer_id, pos, ..
            }
            | PendingFpsTouch::Overlap {
                pointer_id, pos, ..
            } => {
                send_fps_touch(cs_tx, MotionEventAction::Up, pointer_id, mask_size, pos);
            }
        }
    }

    match fps_config.touch_mode {
        FpsTouchMode::Single { interval } => {
            send_fps_touch(
                cs_tx,
                MotionEventAction::Up,
                fps_config.active_pointer_id,
                mask_size,
                old_pos,
            );
            fps_config.touch_active = false;
            if interval > 0 {
                fps_config.pending_touch = Some(PendingFpsTouch::Restore {
                    pointer_id: fps_config.active_pointer_id,
                    pos: center_pos,
                    down_at: Instant::now() + Duration::from_millis(interval),
                    deferred_delta: remaining_delta,
                });
                return FpsRecenterResult::Stop(center_pos);
            }
            send_fps_touch(
                cs_tx,
                MotionEventAction::Down,
                fps_config.active_pointer_id,
                mask_size,
                center_pos,
            );
            fps_config.touch_active = true;
        }
        FpsTouchMode::Dual {
            strategy: FpsDualTouchStrategy::Delay { interval },
            ..
        } => {
            let old_pointer_id = fps_config.active_pointer_id;
            let new_pointer_id = alternate_fps_pointer_id(fps_config);
            send_fps_touch(
                cs_tx,
                MotionEventAction::Down,
                new_pointer_id,
                mask_size,
                center_pos,
            );
            fps_config.active_pointer_id = new_pointer_id;
            if interval > 0 {
                fps_config.pending_touch = Some(PendingFpsTouch::Release {
                    pointer_id: old_pointer_id,
                    pos: old_pos,
                    release_at: Instant::now() + Duration::from_millis(interval),
                    deferred_delta: remaining_delta,
                });
                return FpsRecenterResult::Stop(center_pos);
            }
            send_fps_touch(
                cs_tx,
                MotionEventAction::Up,
                old_pointer_id,
                mask_size,
                old_pos,
            );
        }
        FpsTouchMode::Dual {
            strategy: FpsDualTouchStrategy::Overlap,
            ..
        } => {
            let old_pointer_id = fps_config.active_pointer_id;
            let new_pointer_id = alternate_fps_pointer_id(fps_config);
            send_fps_touch(
                cs_tx,
                MotionEventAction::Down,
                new_pointer_id,
                mask_size,
                center_pos,
            );
            fps_config.pending_touch = Some(PendingFpsTouch::Overlap {
                pointer_id: old_pointer_id,
                pos: old_pos,
                deferred_delta: remaining_delta,
            });
            fps_config.active_pointer_id = new_pointer_id;
            return FpsRecenterResult::Stop(center_pos);
        }
    }

    FpsRecenterResult::Continue {
        cursor_pos: center_pos,
        remaining_delta,
    }
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

    let (physical_min, physical_max) = physical_bounds(mask_size);
    let mut cursor_pos = cursor_pos;
    let mut delta = delta;
    let mut recenter_count = 0;

    loop {
        if delta == Vec2::ZERO {
            return cursor_pos;
        }

        let (bounds_min, bounds_max) = fps_effective_bounds(fps_config, mask_size);
        let Some((boundary_pos, remaining_delta)) =
            first_bounds_crossing(cursor_pos, delta, bounds_min, bounds_max)
        else {
            let new_pos = clamp_to_bounds(cursor_pos + delta, physical_min, physical_max);
            send_fps_touch(
                cs_tx,
                MotionEventAction::Move,
                fps_config.active_pointer_id,
                mask_size,
                new_pos,
            );
            return new_pos;
        };

        if recenter_count >= FPS_MAX_RECENTER_ITERATIONS {
            return cursor_pos;
        }

        let touch_pos = clamp_to_bounds(boundary_pos, physical_min, physical_max);
        send_fps_touch(
            cs_tx,
            MotionEventAction::Move,
            fps_config.active_pointer_id,
            mask_size,
            touch_pos,
        );

        match recenter_fps_touch(cs_tx, fps_config, mask_size, touch_pos, remaining_delta) {
            FpsRecenterResult::Continue {
                cursor_pos: next_cursor_pos,
                remaining_delta: next_delta,
            } => {
                recenter_count += 1;
                cursor_pos = next_cursor_pos;
                delta = next_delta;
            }
            FpsRecenterResult::Stop(next_cursor_pos) => return next_cursor_pos,
        }
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
        && !matches!(pending, PendingFpsTouch::Overlap { .. })
    {
        pending.add_deferred_delta(delta);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast::error::TryRecvError;

    fn fps_config(touch_mode: FpsTouchMode) -> ActiveCursorFpsConfig {
        ActiveCursorFpsConfig {
            touch_active: true,
            ignore_fps_motion: false,
            sensitivity: Vec2::ONE,
            pointer_id: 0,
            active_pointer_id: 0,
            original_pos: Vec2::new(500.0, 500.0),
            original_size: Vec2::new(1000.0, 1000.0),
            max_offset: Vec2::new(50.0, 0.0),
            touch_mode,
            pending_touch: None,
        }
    }

    fn collect_touch_events(
        rx: &mut broadcast::Receiver<ScrcpyControlMsg>,
    ) -> Vec<(MotionEventAction, u64, i32, i32)> {
        let mut events = Vec::new();
        loop {
            match rx.try_recv() {
                Ok(ScrcpyControlMsg::InjectTouchEvent {
                    action,
                    pointer_id,
                    x,
                    y,
                    ..
                }) => events.push((action, pointer_id, x, y)),
                Ok(_) => {}
                Err(TryRecvError::Empty) => break,
                Err(err) => panic!("unexpected broadcast receive error: {err}"),
            }
        }
        events
    }

    fn assert_vec2_near(actual: Vec2, expected: Vec2) {
        assert!(
            (actual - expected).length_squared() < 0.001,
            "expected {expected:?}, got {actual:?}"
        );
    }

    #[test]
    fn apply_fps_delta_consumes_multiple_immediate_boundaries() {
        let (tx, mut rx) = broadcast::channel(32);
        let mut config = fps_config(FpsTouchMode::Single { interval: 0 });

        let result = apply_fps_delta(
            &tx,
            &mut config,
            Vec2::new(1000.0, 1000.0),
            Vec2::new(500.0, 500.0),
            Vec2::new(130.0, 0.0),
        );

        assert_vec2_near(result, Vec2::new(530.0, 500.0));
        let events = collect_touch_events(&mut rx);
        let actions = events
            .iter()
            .map(|(action, _, _, _)| *action)
            .collect::<Vec<_>>();
        assert_eq!(
            actions,
            vec![
                MotionEventAction::Move,
                MotionEventAction::Up,
                MotionEventAction::Down,
                MotionEventAction::Move,
                MotionEventAction::Up,
                MotionEventAction::Down,
                MotionEventAction::Move,
            ]
        );
        assert_eq!(events.last(), Some(&(MotionEventAction::Move, 0, 530, 500)));
    }

    #[test]
    fn apply_fps_delta_limits_immediate_recenter_iterations() {
        let (tx, mut rx) = broadcast::channel(32);
        let mut config = fps_config(FpsTouchMode::Single { interval: 0 });

        let result = apply_fps_delta(
            &tx,
            &mut config,
            Vec2::new(1000.0, 1000.0),
            Vec2::new(500.0, 500.0),
            Vec2::new(260.0, 0.0),
        );

        assert_vec2_near(result, Vec2::new(500.0, 500.0));
        let events = collect_touch_events(&mut rx);
        assert_eq!(events.len(), FPS_MAX_RECENTER_ITERATIONS * 3);
        assert_eq!(
            events
                .iter()
                .filter(|(action, _, _, _)| *action == MotionEventAction::Up)
                .count(),
            FPS_MAX_RECENTER_ITERATIONS
        );
    }

    #[test]
    fn single_touch_interval_defers_next_down() {
        let (tx, mut rx) = broadcast::channel(32);
        let mut config = fps_config(FpsTouchMode::Single { interval: 1 });

        let result = apply_fps_delta(
            &tx,
            &mut config,
            Vec2::new(1000.0, 1000.0),
            Vec2::new(500.0, 500.0),
            Vec2::new(80.0, 0.0),
        );

        assert_vec2_near(result, Vec2::new(500.0, 500.0));
        assert!(!config.touch_active);
        assert!(matches!(
            config.pending_touch,
            Some(PendingFpsTouch::Restore { .. })
        ));
        assert_eq!(
            collect_touch_events(&mut rx),
            vec![
                (MotionEventAction::Move, 0, 550, 500),
                (MotionEventAction::Up, 0, 550, 500),
            ]
        );

        std::thread::sleep(Duration::from_millis(2));
        let deferred_delta = cleanup_pending_fps_touch(&tx, &mut config, Vec2::new(1000.0, 1000.0));

        assert!(config.touch_active);
        assert_eq!(deferred_delta, Some(Vec2::new(30.0, 0.0)));
        assert_eq!(
            collect_touch_events(&mut rx),
            vec![(MotionEventAction::Down, 0, 500, 500)]
        );
    }
}
