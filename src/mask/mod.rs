pub mod mapping;
pub mod mask_command;
pub mod ui;
pub mod video;

use bevy::{
    app::{App, Plugin, Startup, Update},
    ecs::system::{Commands, Single},
    math::Vec2,
    window::Window,
};

use crate::{
    config::LocalConfig,
    mask::{
        mask_command::{MaskSize, TitlebarState, handle_mask_command},
        ui::basic::TITLEBAR_HEIGHT,
        video::{VideoAttributes, handle_video_msg},
    },
};

pub struct MaskPlugins;

impl Plugin for MaskPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((ui::UiPlugins, mapping::MappingPlugins))
            .init_non_send_resource::<VideoAttributes>()
            .add_systems(
                Startup,
                (init_mask_size, init_titlebar_state),
            )
            .add_systems(Update, (handle_mask_command, handle_video_msg));
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
