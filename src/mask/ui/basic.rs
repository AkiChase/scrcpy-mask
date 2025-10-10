use std::time::Duration;

/// Bevy basic plugin that creates a window with a transparent background and a border.
use bevy::{
    prelude::*,
    winit::{UpdateMode, WinitSettings},
};

pub const BORDER_THICKNESS: f32 = 1.0; // logical size

pub struct BasicPlugin;

impl Plugin for BasicPlugin {
    fn build(&self, app: &mut App) {
        app
            // ClearColor resource: The color used to clear the screen at the beginning of each frame
            .insert_resource(ClearColor(Color::NONE))
            .insert_resource(WinitSettings {
                focused_mode: UpdateMode::Continuous,
                unfocused_mode: UpdateMode::reactive_low_power(Duration::from_millis(100)),
            })
            .add_systems(Startup, (setup, border.after(setup)));
    }
}

fn setup(mut commands: Commands, mut window: Single<&mut Window>) {
    window.resolution.set(800., 600.);
    commands.spawn(Camera2d::default());
}

fn border(mut commands: Commands) {
    let border_color = Color::srgba_u8(183, 42, 32, 255);
    commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            border: UiRect::all(Val::Px(BORDER_THICKNESS)),
            box_sizing: BoxSizing::BorderBox,
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor(border_color),
    ));
}
