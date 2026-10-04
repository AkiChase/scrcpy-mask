use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    window::{Monitor, WindowLevel},
};
use bevy_ineffable::prelude::IneffableCommands;
use rust_i18n::t;

use crate::{
    config::LocalConfig,
    mask::{
        mapping::{
            MappingState,
            config::{ActiveMappingConfig, load_mapping_config},
            cursor::{CursorPosition, CursorState},
            script_helper::{ScriptAST, ScriptRuntimeCommandSender, ScriptSharedState},
        },
        ui::basic::TITLEBAR_HEIGHT,
    },
    tokio_tasks::TokioTasksRuntime,
    utils::{ChannelReceiverM, ChannelSenderCS, mask_rect_from_config},
};

#[derive(Debug)]
pub enum MaskCommand {
    WinMove {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    },
    SetAlwaysOnTop {
        enabled: bool,
    },
    SetTitlebarVisible {
        visible: bool,
    },
    UpdateMaskLayout {
        update: MaskLayoutUpdate,
        device_size: Option<(u32, u32)>,
    },
    DeviceConnectionChange {
        connect: bool,
    },
    GetActiveMapping,
    GetScaleFactor,
    LoadAndActivateMappingConfig {
        file_name: String,
    },
    RunScript {
        script: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum MaskLayoutUpdate {
    VerticalMaskHeight(u32),
    HorizontalMaskWidth(u32),
    VerticalPosition((i32, i32)),
    HorizontalPosition((i32, i32)),
}

#[derive(Resource)]
pub struct MaskSize(pub Vec2);

#[derive(Resource)]
pub struct TitlebarState {
    pub visible: bool,
}

#[derive(Resource, Default)]
pub struct PendingWindowFocus {
    frames_remaining: u8,
}

impl TitlebarState {
    pub fn offset(&self) -> f32 {
        if self.visible { TITLEBAR_HEIGHT } else { 0.0 }
    }
}

/// Grouped read-only resources: keeps `handle_mask_command` under the
/// maximum number of system parameters.
#[derive(SystemParam)]
pub struct MaskCommandResources<'w> {
    m_rx: Res<'w, ChannelReceiverM>,
    cs_tx_res: Res<'w, ChannelSenderCS>,
    script_command_tx: Res<'w, ScriptRuntimeCommandSender>,
    shared_state: Res<'w, ScriptSharedState>,
    cursor_pos: Res<'w, CursorPosition>,
    mapping_state: Res<'w, State<MappingState>>,
    cursor_state: Res<'w, State<CursorState>>,
}

pub fn handle_mask_command(
    res: MaskCommandResources,
    mut window: Single<&mut Window>,
    mut next_mapping_state: ResMut<NextState<MappingState>>,
    mut next_cursor_state: ResMut<NextState<CursorState>>,
    mut ineffable: IneffableCommands,
    mut active_mapping: ResMut<ActiveMappingConfig>,
    mut mask_size: ResMut<MaskSize>,
    mut titlebar_state: ResMut<TitlebarState>,
    mut pending_focus: ResMut<PendingWindowFocus>,
    monitors: Query<&Monitor>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    for (msg, oneshot_tx) in res.m_rx.0.try_iter() {
        match msg {
            MaskCommand::WinMove {
                left,
                top,
                right,
                bottom,
            } => {
                let content_width = (right - left) as f32;
                let content_height = (bottom - top) as f32;

                // A position persisted while the window was off-screen (e.g. the
                // transient sentinel coordinates Windows reports for minimized or
                // hidden windows) would otherwise restore the window outside the
                // desktop every time. Clamp it back onto a visible monitor.
                let scale_factor = window.resolution.scale_factor() as f32;
                let (left, top) = clamp_to_visible_monitor(
                    left,
                    top,
                    content_width,
                    content_height,
                    scale_factor,
                    &monitors,
                );

                apply_titlebar_dimensions(
                    &mut window,
                    &mut mask_size,
                    titlebar_state.visible,
                    content_width,
                    content_height,
                    left,
                    top,
                );

                let msg = t!(
                    "mask.windowMovedAndResized",
                    left => left,
                    top => top,
                    width => mask_size.0.x,
                    height => mask_size.0.y
                )
                .to_string();

                log::info!("[Mask] {}", msg);
                let _ = oneshot_tx.send(Ok(msg));
            }
            MaskCommand::SetAlwaysOnTop { enabled } => {
                let msg = apply_always_on_top(&mut window, enabled);
                LocalConfig::set_always_on_top(enabled);
                log::info!("{}", msg);
                let _ = oneshot_tx.send(Ok(msg));
            }
            MaskCommand::SetTitlebarVisible { visible } => {
                let msg = apply_titlebar_visible(
                    &mut window,
                    &mut mask_size,
                    &mut titlebar_state,
                    visible,
                );
                LocalConfig::set_titlebar_visible(visible);
                log::info!("{}", msg);
                let _ = oneshot_tx.send(Ok(msg));
            }
            MaskCommand::UpdateMaskLayout {
                update,
                device_size,
            } => {
                let result = apply_mask_layout_update(
                    &mut window,
                    &mut mask_size,
                    titlebar_state.visible,
                    update,
                    device_size,
                    &monitors,
                );
                match &result {
                    Ok(msg) if !msg.is_empty() => log::info!("[Mask] {}", msg),
                    Ok(_) => {}
                    Err(e) => log::error!("[Mask] {}", e),
                }
                let _ = oneshot_tx.send(result);
            }
            MaskCommand::DeviceConnectionChange { connect } => {
                let msg = if connect {
                    next_mapping_state.set(MappingState::Normal);
                    log::info!("[Mapping] {}", t!("mask.enterNormalMappingMode"));
                    window.visible = true;
                    window.focused = false;
                    pending_focus.frames_remaining = 2;
                    t!("mask.mainDeviceConnected").to_string()
                } else {
                    next_cursor_state.set(CursorState::Normal);
                    next_mapping_state.set(MappingState::Stop);
                    log::info!("[Mapping] {}", t!("mask.exitStopMappingMode"));
                    window.visible = false;
                    window.focused = false;
                    pending_focus.frames_remaining = 0;
                    t!("mask.mainDeviceDisconnected").to_string()
                };
                log::info!("[Mask] {}", msg);
                let _ = oneshot_tx.send(Ok(msg));
            }
            MaskCommand::GetActiveMapping => {
                let _ = oneshot_tx.send(Ok(active_mapping.1.clone()));
            }
            MaskCommand::GetScaleFactor => {
                let _ = oneshot_tx.send(Ok(window.resolution.scale_factor().to_string()));
            }
            MaskCommand::LoadAndActivateMappingConfig { file_name } => {
                log::info!(
                    "[Mapping] {}: {}",
                    t!("mask.loadActivateMappingConfig"),
                    file_name
                );
                match load_mapping_config(&file_name) {
                    Ok((mapping_config, input_config)) => {
                        ineffable.set_config(&input_config);
                        active_mapping.0 = Some(mapping_config);
                        active_mapping.1 = file_name;
                        let _ = oneshot_tx.send(Ok(String::new()));
                    }
                    Err(e) => {
                        let _ = oneshot_tx.send(Err(e));
                    }
                }
            }
            MaskCommand::RunScript { script } => {
                let ast = match ScriptAST::new(&script) {
                    Err(e) => {
                        let _ = oneshot_tx.send(Err(e));
                        return;
                    }
                    Ok(ast) => ast,
                };

                if let Some(mapping_config) = &active_mapping.0 {
                    let cs_tx = res.cs_tx_res.0.clone();
                    let script_command_tx = res.script_command_tx.0.clone();
                    let shared_state = res.shared_state.as_ref().clone();
                    let original_size = mapping_config.original_size.into();
                    let cursor_pos = res.cursor_pos.0;
                    let mask_size = mask_size.0;
                    let raw_input_flag = res.mapping_state.get() == &MappingState::RawInput;
                    let fps_mode_flag = res.cursor_state.get() == &CursorState::Fps;
                    runtime.spawn_background_task(move |_ctx| async move {
                        let result = ast
                            .run_script(
                                &cs_tx,
                                &script_command_tx,
                                &shared_state,
                                "RunScript",
                                original_size,
                                cursor_pos,
                                mask_size,
                                raw_input_flag,
                                fps_mode_flag,
                            )
                            .await
                            .map(|_| String::new())
                            .map_err(|e| e.to_string());
                        let _ = oneshot_tx.send(result);
                    });
                } else {
                    let _ = oneshot_tx.send(Err(t!("mask.runScriptnoMappingError").to_string()));
                }
            }
        }
    }
}

pub fn apply_pending_window_focus(
    mut pending_focus: ResMut<PendingWindowFocus>,
    mut window: Single<&mut Window>,
) {
    if pending_focus.frames_remaining == 0 {
        return;
    }

    pending_focus.frames_remaining -= 1;
    if pending_focus.frames_remaining == 0 {
        window.focused = true;
    }
}

fn apply_always_on_top(window: &mut Window, enabled: bool) -> String {
    window.window_level = if enabled {
        WindowLevel::AlwaysOnTop
    } else {
        WindowLevel::Normal
    };
    format!("[Mask] {}: {}", t!("mask.windowLevelChanged"), enabled)
}

fn apply_titlebar_visible(
    window: &mut Window,
    mask_size: &mut MaskSize,
    titlebar_state: &mut TitlebarState,
    visible: bool,
) -> String {
    if titlebar_state.visible == visible {
        return format!("[Mask] Titlebar visible: {}", visible);
    }

    let bevy::window::WindowPosition::At(pos) = window.position else {
        unreachable!("window position should always be At")
    };
    let scale_factor = window.resolution.scale_factor() as f32;
    let old_visible = titlebar_state.visible;
    let content_top = if old_visible {
        physical_to_logical_i32(pos.y, scale_factor) + TITLEBAR_HEIGHT.round() as i32
    } else {
        physical_to_logical_i32(pos.y, scale_factor)
    };
    let content_left = physical_to_logical_i32(pos.x, scale_factor);
    let content_width = mask_size.0.x;
    let content_height = mask_size.0.y;

    titlebar_state.visible = visible;
    apply_titlebar_dimensions(
        window,
        mask_size,
        visible,
        content_width,
        content_height,
        content_left,
        content_top,
    );

    format!("[Mask] Titlebar visible: {}", visible)
}

fn apply_mask_layout_update(
    window: &mut Window,
    mask_size: &mut MaskSize,
    titlebar_visible: bool,
    update: MaskLayoutUpdate,
    device_size: Option<(u32, u32)>,
    monitors: &Query<&Monitor>,
) -> Result<String, String> {
    let mut config = LocalConfig::get();
    match update {
        MaskLayoutUpdate::VerticalMaskHeight(value) => config.vertical_mask_height = value,
        MaskLayoutUpdate::HorizontalMaskWidth(value) => config.horizontal_mask_width = value,
        MaskLayoutUpdate::VerticalPosition(value) => config.vertical_position = value,
        MaskLayoutUpdate::HorizontalPosition(value) => config.horizontal_position = value,
    }

    let msg = if let Some(device_size) = device_size {
        let (device_w, device_h) = device_size;
        let (left, top, right, bottom) = mask_rect_from_config(&config, device_w, device_h)?;
        let content_width = (right - left) as f32;
        let content_height = (bottom - top) as f32;

        // Same off-screen protection as the restore path, so a mask configured
        // outside every monitor stays reachable.
        let scale_factor = window.resolution.scale_factor() as f32;
        let (left, top) = clamp_to_visible_monitor(
            left,
            top,
            content_width,
            content_height,
            scale_factor,
            monitors,
        );

        apply_titlebar_dimensions(
            window,
            mask_size,
            titlebar_visible,
            content_width,
            content_height,
            left,
            top,
        );

        t!(
            "mask.windowMovedAndResized",
            left => left,
            top => top,
            width => mask_size.0.x,
            height => mask_size.0.y
        )
        .to_string()
    } else {
        String::new()
    };

    match update {
        MaskLayoutUpdate::VerticalMaskHeight(value) => LocalConfig::set_vertical_mask_height(value),
        MaskLayoutUpdate::HorizontalMaskWidth(value) => {
            LocalConfig::set_horizontal_mask_width(value)
        }
        MaskLayoutUpdate::VerticalPosition(value) => LocalConfig::set_vertical_position(value),
        MaskLayoutUpdate::HorizontalPosition(value) => LocalConfig::set_horizontal_position(value),
    }

    Ok(msg)
}

fn apply_titlebar_dimensions(
    window: &mut Window,
    mask_size: &mut MaskSize,
    titlebar_visible: bool,
    content_width: f32,
    content_height: f32,
    left: i32,
    top: i32,
) {
    let scale_factor = window.resolution.scale_factor() as f32;

    let win_height = if titlebar_visible {
        content_height + TITLEBAR_HEIGHT
    } else {
        content_height
    };
    let win_top_logical = if titlebar_visible {
        top as f32 - TITLEBAR_HEIGHT
    } else {
        top as f32
    };
    let win_left = logical_to_physical_i32(left as f32, scale_factor);
    let win_top = logical_to_physical_i32(win_top_logical, scale_factor);

    window.resolution.set(content_width, win_height);
    window.position.set((win_left, win_top).into());
    mask_size.0 = Vec2::new(content_width, content_height);
}

pub fn physical_to_logical_i32(value: i32, scale_factor: f32) -> i32 {
    (value as f32 / scale_factor).round() as i32
}

fn logical_to_physical_i32(value: f32, scale_factor: f32) -> i32 {
    (value * scale_factor).round() as i32
}

/// True when the logical rect (content area) intersects at least one monitor.
///
/// Windows reports transient off-screen sentinel coordinates while a window is
/// minimized, hidden or being restored. Persisting or restoring such a position
/// parks the mask window permanently outside the desktop, where it can neither
/// be seen nor clicked. When no monitor is known yet (early startup), accept the
/// position instead of rejecting a possibly valid one.
pub fn rect_intersects_monitor(
    left: i32,
    top: i32,
    width: f32,
    height: f32,
    scale_factor: f32,
    monitors: &Query<&Monitor>,
) -> bool {
    if monitors.is_empty() {
        return true;
    }
    let left_p = logical_to_physical_i32(left as f32, scale_factor);
    let top_p = logical_to_physical_i32(top as f32, scale_factor);
    let right_p = logical_to_physical_i32(left as f32 + width, scale_factor);
    let bottom_p = logical_to_physical_i32(top as f32 + height, scale_factor);

    monitors.iter().any(|m| {
        let (ml, mt) = (m.physical_position.x, m.physical_position.y);
        let (mr, mb) = (ml + m.physical_width as i32, mt + m.physical_height as i32);
        left_p < mr && right_p > ml && top_p < mb && bottom_p > mt
    })
}

/// Clamp a restore position so the window stays reachable on a visible monitor.
///
/// Positions already intersecting a monitor are returned unchanged, so windows
/// intentionally parked on a secondary monitor keep their placement. Off-screen
/// positions fall back to the monitor that contains the desktop origin (the
/// primary monitor in the common case), or to the first monitor, with a small
/// margin. The follow-up `WindowMoved` event then persists the corrected value.
pub fn clamp_to_visible_monitor(
    left: i32,
    top: i32,
    width: f32,
    height: f32,
    scale_factor: f32,
    monitors: &Query<&Monitor>,
) -> (i32, i32) {
    if rect_intersects_monitor(left, top, width, height, scale_factor, monitors) {
        return (left, top);
    }
    let Some(fallback) = monitors.iter().min_by_key(|m| {
        let (ml, mt) = (m.physical_position.x, m.physical_position.y);
        let (mr, mb) = (ml + m.physical_width as i32, mt + m.physical_height as i32);
        let contains_origin = ml <= 0 && mr > 0 && mt <= 0 && mb > 0;
        if contains_origin {
            (0, ml, mt)
        } else {
            (1, ml, mt)
        }
    }) else {
        return (left, top);
    };
    let fallback_left = (fallback.physical_position.x as f32 / scale_factor).round() as i32 + 60;
    let fallback_top = (fallback.physical_position.y as f32 / scale_factor).round() as i32 + 60;
    log::warn!(
        "[Mask] Saved position is off-screen; moving window to a visible monitor at ({}, {})",
        fallback_left,
        fallback_top
    );
    (fallback_left, fallback_top)
}
