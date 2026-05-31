pub mod mapping;
pub mod mask_command;
pub mod ui;
pub mod video;

use std::time::Duration;

use bevy::{
    app::{App, Plugin, Startup, Update},
    ecs::{
        message::MessageReader,
        system::{Commands, Local, Res, ResMut, Single},
    },
    math::Vec2,
    time::{Time, Timer, TimerMode},
    window::{Window, WindowMoved, WindowPosition, WindowResized},
};

use crate::{
    config::LocalConfig,
    mask::{
        mask_command::{MaskSize, TitlebarState, handle_mask_command},
        ui::basic::TITLEBAR_HEIGHT,
        video::{VideoAttributes, handle_video_msg},
    },
    utils::{DeviceOrientation, share::ControlledDevice},
};

pub struct MaskPlugins;

impl Plugin for MaskPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((ui::UiPlugins, mapping::MappingPlugins))
            .init_non_send_resource::<VideoAttributes>()
            .add_systems(Startup, (init_mask_size, init_titlebar_state))
            .add_systems(
                Update,
                (
                    sync_mask_size,
                    sync_mask_position,
                    handle_mask_command,
                    handle_video_msg,
                ),
            );
    }
}

fn init_mask_size(mut commands: Commands, window: Single<&Window>) {
    let config = LocalConfig::get();
    let mask_h = if config.titlebar_visible {
        (window.size().y - TITLEBAR_HEIGHT).max(0.0)
    } else {
        window.size().y
    };
    commands.insert_resource(MaskSize(Vec2::new(window.size().x, mask_h)));
}

fn init_titlebar_state(mut commands: Commands) {
    let config = LocalConfig::get();
    commands.insert_resource(TitlebarState {
        visible: config.titlebar_visible,
    });
}

const DEBOUNCE_MS: u64 = 200;

#[derive(Default)]
struct ResizeDebounce {
    timer: Timer,
    pending: bool,
}

impl ResizeDebounce {
    fn ensure_init(&mut self) {
        if self.timer.duration() == Duration::ZERO {
            self.timer = Timer::new(Duration::from_millis(DEBOUNCE_MS), TimerMode::Once);
        }
    }
}

#[derive(Default)]
struct MoveDebounce {
    timer: Timer,
    pending: bool,
}

impl MoveDebounce {
    fn ensure_init(&mut self) {
        if self.timer.duration() == Duration::ZERO {
            self.timer = Timer::new(Duration::from_millis(DEBOUNCE_MS), TimerMode::Once);
        }
    }
}

fn sync_mask_size(
    mut resize_reader: MessageReader<WindowResized>,
    titlebar_state: Res<TitlebarState>,
    mut mask_size: ResMut<MaskSize>,
    mut window: Single<&mut Window>,
    time: Res<Time>,
    mut debounce: Local<ResizeDebounce>,
) {
    debounce.ensure_init();

    for e in resize_reader.read() {
        let h = (e.height - titlebar_state.offset()).max(0.0);
        mask_size.0 = Vec2::new(e.width, h);
        debounce.timer.reset();
        debounce.pending = true;
    }

    if debounce.pending {
        debounce.timer.tick(time.delta());
        if debounce.timer.just_finished() {
            debounce.pending = false;
            if let Some(device) = ControlledDevice::get_main_device_blocking() {
                let (dw, dh) = device.device_size;
                if dw == 0 || dh == 0 {
                    return;
                }
                let device_w = dw as f32;
                let device_h = dh as f32;
                let orientation = DeviceOrientation::from_size(dw, dh);
                let titlebar_offset = titlebar_state.offset();
                let current_w = mask_size.0.x;
                let current_h = mask_size.0.y;

                match orientation {
                    DeviceOrientation::Landscape => {
                        let target_h = (current_w * (device_h / device_w)).round();
                        if target_h != current_h {
                            window.resolution.set(current_w, target_h + titlebar_offset);
                            mask_size.0 = Vec2::new(current_w, target_h);
                        }
                    }
                    DeviceOrientation::Portrait => {
                        let target_w = (current_h * (device_w / device_h)).round();
                        if target_w != current_w {
                            window.resolution.set(target_w, current_h + titlebar_offset);
                            mask_size.0 = Vec2::new(target_w, current_h);
                        }
                    }
                }

                // Persist size and position after debounce settles
                let content_w = mask_size.0.x.round() as u32;
                let content_h = mask_size.0.y.round() as u32;
                let WindowPosition::At(pos) = window.position else {
                    return;
                };
                let scale_factor = window.resolution.scale_factor() as f32;
                let titlebar_physical = (TITLEBAR_HEIGHT * scale_factor) as i32;
                let content_top = if titlebar_state.visible {
                    pos.y + titlebar_physical
                } else {
                    pos.y
                };
                let content_left = pos.x;

                match orientation {
                    DeviceOrientation::Landscape => {
                        LocalConfig::set_horizontal_mask_width(content_w);
                        LocalConfig::set_horizontal_position((content_left, content_top));
                    }
                    DeviceOrientation::Portrait => {
                        LocalConfig::set_vertical_mask_height(content_h);
                        LocalConfig::set_vertical_position((content_left, content_top));
                    }
                }
            }
        }
    }
}

fn sync_mask_position(
    mut move_reader: MessageReader<WindowMoved>,
    window: Single<&Window>,
    titlebar_state: Res<TitlebarState>,
    time: Res<Time>,
    mut debounce: Local<MoveDebounce>,
) {
    debounce.ensure_init();

    for _ in move_reader.read() {
        debounce.timer.reset();
        debounce.pending = true;
    }

    if debounce.pending {
        debounce.timer.tick(time.delta());
        if debounce.timer.just_finished() {
            debounce.pending = false;
            if let Some(device) = ControlledDevice::get_main_device_blocking() {
                let (dw, dh) = device.device_size;
                if dw == 0 || dh == 0 {
                    return;
                }
                let WindowPosition::At(pos) = window.position else {
                    return;
                };
                let scale_factor = window.resolution.scale_factor() as f32;
                let titlebar_physical = (TITLEBAR_HEIGHT * scale_factor) as i32;
                let content_top = if titlebar_state.visible {
                    pos.y + titlebar_physical
                } else {
                    pos.y
                };
                let content_left = pos.x;

                match DeviceOrientation::from_size(dw, dh) {
                    DeviceOrientation::Landscape => {
                        LocalConfig::set_horizontal_position((content_left, content_top));
                    }
                    DeviceOrientation::Portrait => {
                        LocalConfig::set_vertical_position((content_left, content_top));
                    }
                }
            }
        }
    }
}
