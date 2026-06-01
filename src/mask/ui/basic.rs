use std::time::Duration;

use bevy::{
    math::CompassOctant,
    prelude::*,
    window::WindowLevel,
    winit::{UpdateMode, WinitSettings},
};

use crate::{
    config::LocalConfig,
    mask::{mask_command::TitlebarState, video::VideoPlayer},
    scrcpy::controller::ControllerCommand,
    utils::{ChannelSenderD, DeviceOrientation, share::ControlledDevice},
};

pub const BORDER_THICKNESS: f32 = 1.0;
pub const TITLEBAR_HEIGHT: f32 = 30.0;

const EDGE_HANDLE_SIZE: f32 = 6.0;
const CORNER_HANDLE_SIZE: f32 = 12.0;

#[derive(Component)]
struct ResizeHandle(CompassOctant);

#[derive(Component)]
pub struct MaskContentMarker;

#[derive(Component)]
pub struct TitlebarMarker;

#[derive(Component)]
struct MinimizeButton;

#[derive(Component)]
struct CloseButton;

#[derive(Component)]
struct PushpinButton;

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
            .add_systems(Update, (button_interaction, handle_titlebar_buttons, handle_titlebar_drag, handle_resize, sync_titlebar_visibility, sync_pushpin_style));
    }
}

fn setup_ui(mut commands: Commands, mut window: Single<&mut Window>, asset_server: Res<AssetServer>) {
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

    let minimize_icon: Handle<Image> = asset_server.load("icons/minus.png");
    let pushpin_icon: Handle<Image> = asset_server.load("icons/pushpin.png");
    let close_icon: Handle<Image> = asset_server.load("icons/close.png");

    let initial_pin_bg = if config.always_on_top {
        PIN_NORMAL_BG
    } else {
        NORMAL_BG
    };

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
            Interaction::default(),
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

        // Button container (right side)
        titlebar
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.),
                ..default()
            })
            .with_children(|buttons| {
                buttons
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(20.),
                            height: Val::Px(20.),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(NORMAL_BG),
                        MinimizeButton,
                    ))
                    .with_child((
                        Node {
                            width: Val::Px(14.),
                            height: Val::Px(14.),
                            ..default()
                        },
                        ImageNode::new(minimize_icon),
                    ));

                buttons
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(20.),
                            height: Val::Px(20.),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(initial_pin_bg),
                        PushpinButton,
                    ))
                    .with_child((
                        Node {
                            width: Val::Px(14.),
                            height: Val::Px(14.),
                            ..default()
                        },
                        ImageNode::new(pushpin_icon),
                    ));

                buttons
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(20.),
                            height: Val::Px(20.),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(CLOSE_NORMAL_BG),
                        CloseButton,
                    ))
                    .with_child((
                        Node {
                            width: Val::Px(14.),
                            height: Val::Px(14.),
                            ..default()
                        },
                        ImageNode::new(close_icon),
                    ));
            });
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

        // Resize handles (invisible, layered on top of border)
        let edge_z = ZIndex(10);
        let corner_z = ZIndex(11);

        // Edge handles
        content.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.),
                left: Val::Px(0.),
                width: Val::Percent(100.),
                height: Val::Px(EDGE_HANDLE_SIZE),
                ..default()
            },
            edge_z,
            BackgroundColor(Color::NONE),
            Interaction::default(),
            ResizeHandle(CompassOctant::North),
        ));
        content.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.),
                left: Val::Px(0.),
                width: Val::Percent(100.),
                height: Val::Px(EDGE_HANDLE_SIZE),
                ..default()
            },
            edge_z,
            BackgroundColor(Color::NONE),
            Interaction::default(),
            ResizeHandle(CompassOctant::South),
        ));
        content.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.),
                left: Val::Px(0.),
                width: Val::Px(EDGE_HANDLE_SIZE),
                height: Val::Percent(100.),
                ..default()
            },
            edge_z,
            BackgroundColor(Color::NONE),
            Interaction::default(),
            ResizeHandle(CompassOctant::West),
        ));
        content.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.),
                right: Val::Px(0.),
                width: Val::Px(EDGE_HANDLE_SIZE),
                height: Val::Percent(100.),
                ..default()
            },
            edge_z,
            BackgroundColor(Color::NONE),
            Interaction::default(),
            ResizeHandle(CompassOctant::East),
        ));

        // Corner handles (higher z-index to capture clicks over edges)
        content.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.),
                left: Val::Px(0.),
                width: Val::Px(CORNER_HANDLE_SIZE),
                height: Val::Px(CORNER_HANDLE_SIZE),
                ..default()
            },
            corner_z,
            BackgroundColor(Color::NONE),
            Interaction::default(),
            ResizeHandle(CompassOctant::NorthWest),
        ));
        content.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.),
                right: Val::Px(0.),
                width: Val::Px(CORNER_HANDLE_SIZE),
                height: Val::Px(CORNER_HANDLE_SIZE),
                ..default()
            },
            corner_z,
            BackgroundColor(Color::NONE),
            Interaction::default(),
            ResizeHandle(CompassOctant::NorthEast),
        ));
        content.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.),
                left: Val::Px(0.),
                width: Val::Px(CORNER_HANDLE_SIZE),
                height: Val::Px(CORNER_HANDLE_SIZE),
                ..default()
            },
            corner_z,
            BackgroundColor(Color::NONE),
            Interaction::default(),
            ResizeHandle(CompassOctant::SouthWest),
        ));
        content.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.),
                right: Val::Px(0.),
                width: Val::Px(CORNER_HANDLE_SIZE),
                height: Val::Px(CORNER_HANDLE_SIZE),
                ..default()
            },
            corner_z,
            BackgroundColor(Color::NONE),
            Interaction::default(),
            ResizeHandle(CompassOctant::SouthEast),
        ));
    });
}

fn handle_titlebar_drag(
    mut window: Single<&mut Window>,
    interaction_query: Query<&Interaction, (With<TitlebarMarker>, Changed<Interaction>)>,
    button_query: Query<&Interaction, Or<(With<MinimizeButton>, With<PushpinButton>, With<CloseButton>)>>,
) {
    let button_pressed = button_query.iter().any(|i| *i == Interaction::Pressed);
    if !button_pressed && interaction_query.iter().any(|i| *i == Interaction::Pressed) {
        window.start_drag_move();
    }
}

fn handle_titlebar_buttons(
    mut window: Single<&mut Window>,
    minimize_query: Query<&Interaction, (With<MinimizeButton>, Changed<Interaction>)>,
    pushpin_query: Query<&Interaction, (With<PushpinButton>, Changed<Interaction>)>,
    close_query: Query<&Interaction, (With<CloseButton>, Changed<Interaction>)>,
    d_tx: Res<ChannelSenderD>,
) {
    for interaction in minimize_query.iter() {
        if *interaction == Interaction::Pressed {
            window.set_minimized(true);
        }
    }
    for interaction in pushpin_query.iter() {
        if *interaction == Interaction::Pressed {
            let top = window.window_level != WindowLevel::AlwaysOnTop;
            if top {
                window.window_level = WindowLevel::AlwaysOnTop;
            } else {
                window.window_level = WindowLevel::Normal;
            }
            LocalConfig::set_always_on_top(top);
        }
    }
    for interaction in close_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Some(device) = ControlledDevice::get_main_device_blocking() {
                let _ = d_tx.0.send(ControllerCommand::ShutdownMain(device.scid.clone()));
            }
        }
    }
}

const NORMAL_BG: Color = Color::srgba(0.25, 0.25, 0.25, 0.6);
const HOVERED_BG: Color = Color::srgba(0.38, 0.38, 0.38, 0.7);
const PRESSED_BG: Color = Color::srgba(0.15, 0.15, 0.15, 0.85);
const CLOSE_NORMAL_BG: Color = Color::srgba(0.82, 0.25, 0.2, 0.7);
const CLOSE_HOVER_BG: Color = Color::srgba(0.95, 0.3, 0.25, 0.85);
const CLOSE_PRESSED_BG: Color = Color::srgba(0.65, 0.12, 0.08, 0.9);
const PIN_NORMAL_BG: Color = Color::srgba(0.7, 0.55, 0.15, 0.65);
const PIN_HOVER_BG: Color = Color::srgba(0.8, 0.65, 0.2, 0.75);
const PIN_PRESSED_BG: Color = Color::srgba(0.55, 0.42, 0.1, 0.85);

fn button_interaction(
    window: Single<&Window>,
    minimize_query: Query<(Entity, &Interaction), (With<MinimizeButton>, Changed<Interaction>)>,
    pushpin_query: Query<(Entity, &Interaction), (With<PushpinButton>, Changed<Interaction>)>,
    close_query: Query<(Entity, &Interaction), (With<CloseButton>, Changed<Interaction>)>,
    mut bg_query: Query<&mut BackgroundColor>,
) {
    for (entity, interaction) in minimize_query.iter() {
        if let Ok(mut bg) = bg_query.get_mut(entity) {
            *bg = match *interaction {
                Interaction::Pressed => PRESSED_BG,
                Interaction::Hovered => HOVERED_BG,
                Interaction::None => NORMAL_BG,
            }
            .into();
        }
    }
    let pinned = window.window_level == WindowLevel::AlwaysOnTop;
    for (entity, interaction) in pushpin_query.iter() {
        if let Ok(mut bg) = bg_query.get_mut(entity) {
            *bg = if pinned {
                match *interaction {
                    Interaction::Pressed => PIN_PRESSED_BG,
                    Interaction::Hovered => PIN_HOVER_BG,
                    Interaction::None => PIN_NORMAL_BG,
                }
            } else {
                match *interaction {
                    Interaction::Pressed => PRESSED_BG,
                    Interaction::Hovered => HOVERED_BG,
                    Interaction::None => NORMAL_BG,
                }
            }
            .into();
        }
    }
    for (entity, interaction) in close_query.iter() {
        if let Ok(mut bg) = bg_query.get_mut(entity) {
            *bg = match *interaction {
                Interaction::Pressed => CLOSE_PRESSED_BG,
                Interaction::Hovered => CLOSE_HOVER_BG,
                Interaction::None => CLOSE_NORMAL_BG,
            }
            .into();
        }
    }
}

fn map_handle_for_device(handle: CompassOctant, orientation: DeviceOrientation) -> Option<CompassOctant> {
    match orientation {
        DeviceOrientation::Landscape => match handle {
            CompassOctant::NorthWest | CompassOctant::West | CompassOctant::SouthWest => Some(CompassOctant::West),
            CompassOctant::NorthEast | CompassOctant::East | CompassOctant::SouthEast => Some(CompassOctant::East),
            CompassOctant::North | CompassOctant::South => None,
        },
        DeviceOrientation::Portrait => match handle {
            CompassOctant::NorthWest | CompassOctant::North | CompassOctant::NorthEast => Some(CompassOctant::North),
            CompassOctant::SouthWest | CompassOctant::South | CompassOctant::SouthEast => Some(CompassOctant::South),
            CompassOctant::East | CompassOctant::West => None,
        },
    }
}

fn handle_resize(
    mut window: Single<&mut Window>,
    query: Query<(&ResizeHandle, &Interaction), Changed<Interaction>>,
) {
    let device = ControlledDevice::get_main_device_blocking();

    for (handle, interaction) in query.iter() {
        if *interaction == Interaction::Pressed {
            let direction = match &device {
                Some(dev) if dev.device_size.0 > 0 && dev.device_size.1 > 0 => {
                    let orientation = DeviceOrientation::from_size(dev.device_size.0, dev.device_size.1);
                    map_handle_for_device(handle.0, orientation)
                }
                _ => None, // no device or unknown size: block resize
            };
            if let Some(dir) = direction {
                window.start_drag_resize(dir);
            }
        }
    }
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

fn sync_pushpin_style(
    window: Single<&Window, Changed<Window>>,
    mut pushpin_query: Query<(&Interaction, &mut BackgroundColor), With<PushpinButton>>,
) {
    let pinned = window.window_level == WindowLevel::AlwaysOnTop;
    for (interaction, mut bg) in pushpin_query.iter_mut() {
        if *interaction == Interaction::None {
            *bg = if pinned { PIN_NORMAL_BG } else { NORMAL_BG }.into();
        }
    }
}
