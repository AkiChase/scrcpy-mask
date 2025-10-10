pub mod mapping;
pub mod mask_command;
pub mod ui;
pub mod video;

use bevy::{
    app::{App, Plugin, Startup, Update},
    ecs::system::{Commands, Single},
    window::Window,
};

use crate::mask::{
    mask_command::{MaskSize, handle_mask_command},
    video::{VideoAttributes, handle_video_msg, init_video},
};

pub struct MaskPlugins;

impl Plugin for MaskPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((ui::UiPlugins, mapping::MappingPlugins))
            .init_non_send_resource::<VideoAttributes>()
            .add_systems(Startup, (init_mask_size, init_video))
            .add_systems(Update, (handle_mask_command, handle_video_msg));
    }
}

fn init_mask_size(mut commands: Commands, window: Single<&Window>) {
    commands.insert_resource(MaskSize(window.size()));
}
