use std::time::Duration;

use bevy::{prelude::*, winit::{UpdateMode, WinitSettings}};

use crate::{
    config::LocalConfig,
    mask::{mask_command::TitlebarState, video::VideoPlayer},
};

pub const BORDER_THICKNESS: f32 = 1.0;
pub const TITLEBAR_HEIGHT: f32 = 30.0;

#[derive(Component)]
pub struct MaskContentMarker;

#[derive(Component)]
pub struct TitlebarMarker;


#[derive(Resource)]
pub struct MaskContentEntity(pub Entity);

pub struct BasicPlugin;

impl Plugin for BasicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::NONE))
            .insert_resource(WinitSettings {
                focused_mode: UpdateMode::Continuous,
                unfocused_mode: UpdateMode::reactive_low_power(Duration::from_millis(100)),
            })
            .add_systems(Startup, setup_ui)
            .add_systems(Update, sync_titlebar_visibility);
    }
}

fn setup_ui(mut commands: Commands, mut window: Single<&mut Window>) {
    let config = LocalConfig::get();
    let win_h = if config.titlebar_visible {
        600. + TITLEBAR_HEIGHT
    } else {
        600.
    };
    window.resolution.set(800., win_h);

    commands.spawn(Camera2d::default());

    let titlebar_bg = Color::srgba(0.05, 0.05, 0.05, 0.85);
    let border_color = Color::srgba_u8(183, 42, 32, 255);

    // Root container
    let root_entity = commands
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .id();

    // Titlebar
    let titlebar_entity = commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Px(TITLEBAR_HEIGHT),
                padding: UiRect::px(8., 8., 0., 0.),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(titlebar_bg),
            TitlebarMarker,
        ))
        .id();

    commands.entity(titlebar_entity).with_children(|titlebar| {
        titlebar.spawn((
            Text::new("scrcpy-mask"),
            TextFont {
                font_size: 13.,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
    });

    // Mask content container
    let mask_entity = commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                flex_grow: 1.,
                ..default()
            },
            MaskContentMarker,
        ))
        .id();
    commands.insert_resource(MaskContentEntity(mask_entity));

    // Parent hierarchy: root -> titlebar, root -> mask_content
    commands.entity(root_entity).add_children(&[titlebar_entity, mask_entity]);

    // Add children to MaskContent
    commands.entity(mask_entity).with_children(|content| {
        // Video (absolute, behind border)
        content.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                position_type: PositionType::Absolute,
                padding: UiRect::all(Val::Px(BORDER_THICKNESS)),
                box_sizing: BoxSizing::BorderBox,
                ..default()
            },
            ZIndex(-1),
            BackgroundColor(Color::NONE),
            ImageNode::default(),
            VideoPlayer,
        ));

        // Border (absolute to fill the content area)
        content.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                position_type: PositionType::Absolute,
                border: UiRect::all(Val::Px(BORDER_THICKNESS)),
                box_sizing: BoxSizing::BorderBox,
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(border_color),
        ));
    });
}

fn sync_titlebar_visibility(
    titlebar_state: Res<TitlebarState>,
    mut titlebar_query: Query<&mut Node, With<TitlebarMarker>>,
) {
    if !titlebar_state.is_changed() {
        return;
    }
    for mut node in titlebar_query.iter_mut() {
        node.display = if titlebar_state.visible {
            Display::Flex
        } else {
            Display::None
        };
    }
}
