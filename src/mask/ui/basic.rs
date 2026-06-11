use std::time::Duration;

use bevy::{
    math::CompassOctant,
    prelude::IntoScheduleConfigs,
    prelude::*,
    window::WindowLevel,
    winit::{UpdateMode, WinitSettings},
};
use bevy_ui_render::prelude::MaterialNode;

use crate::{
    config::LocalConfig,
    mask::{
        MaskFrameSet, MaskResizeState,
        mask_command::TitlebarState,
        video::{VideoPlayer, YuvVideoMaterial, create_initial_yuv_material},
    },
    scrcpy::{constant::Keycode, controller::ControllerCommand, device_action},
    utils::{ChannelSenderCS, ChannelSenderD, DeviceOrientation, share::ControlledDevice},
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

#[derive(Component)]
pub struct DeviceButton(pub DeviceAction);

pub enum DeviceAction {
    Back,
    Home,
    AppSwitch,
    ScreenOff,
    ScreenOn,
    VolumeUp,
    VolumeDown,
}

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
            .add_systems(
                Update,
                (
                    button_interaction,
                    handle_titlebar_buttons,
                    handle_device_buttons,
                    handle_titlebar_drag,
                    handle_resize.in_set(MaskFrameSet::Resize),
                    sync_titlebar_visibility,
                    sync_pushpin_style,
                ),
            );
    }
}

fn setup_ui(
    mut commands: Commands,
    mut window: Single<&mut Window>,
    mut images: ResMut<Assets<Image>>,
    mut yuv_materials: ResMut<Assets<YuvVideoMaterial>>,
    asset_server: Res<AssetServer>,
) {
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
    let video_material = create_initial_yuv_material(&mut images, &mut yuv_materials);

    let minimize_icon: Handle<Image> = asset_server.load("icons/minus.png");
    let pushpin_icon: Handle<Image> = asset_server.load("icons/pushpin.png");
    let close_icon: Handle<Image> = asset_server.load("icons/close.png");
    let screen_off_icon: Handle<Image> = asset_server.load("icons/bulb.png");
    let screen_on_icon: Handle<Image> = asset_server.load("icons/bulb-fill.png");
    let volume_up_icon: Handle<Image> = asset_server.load("icons/up.png");
    let volume_down_icon: Handle<Image> = asset_server.load("icons/down.png");
    let back_icon: Handle<Image> = asset_server.load("icons/enter.png");
    let home_icon: Handle<Image> = asset_server.load("icons/border.png");
    let menu_icon: Handle<Image> = asset_server.load("icons/menu.png");

    let initial_pin_bg = if config.always_on_top {
        MAC_PIN_BG
    } else {
        MAC_PIN_INACTIVE_BG
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
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(titlebar_bg),
            TitlebarMarker,
            Interaction::default(),
        ))
        .id();

    commands.entity(titlebar_entity).with_children(|titlebar| {
        // Left group: window buttons + app name
        titlebar
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.),
                ..default()
            })
            .with_children(|left| {
                left.spawn((
                    Button,
                    Node {
                        width: Val::Px(14.),
                        height: Val::Px(14.),
                        border_radius: BorderRadius::all(Val::Px(7.)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(MAC_CLOSE_BG),
                    CloseButton,
                ))
                .with_child((
                    Node {
                        width: Val::Px(10.),
                        height: Val::Px(10.),
                        ..default()
                    },
                    ImageNode::new(close_icon.clone()),
                ));

                left.spawn((
                    Button,
                    Node {
                        width: Val::Px(14.),
                        height: Val::Px(14.),
                        border_radius: BorderRadius::all(Val::Px(7.)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(MAC_MINIMIZE_BG),
                    MinimizeButton,
                ))
                .with_child((
                    Node {
                        width: Val::Px(10.),
                        height: Val::Px(10.),
                        ..default()
                    },
                    ImageNode::new(minimize_icon),
                ));

                left.spawn((
                    Button,
                    Node {
                        width: Val::Px(14.),
                        height: Val::Px(14.),
                        border_radius: BorderRadius::all(Val::Px(7.)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(initial_pin_bg),
                    PushpinButton,
                ))
                .with_child((
                    Node {
                        width: Val::Px(10.),
                        height: Val::Px(10.),
                        ..default()
                    },
                    ImageNode::new(pushpin_icon),
                ));

                left.spawn((
                    Text::new("scrcpy-mask"),
                    TextFont {
                        font_size: FontSize::Px(14.),
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::px(5., 0., 0., 0.),
                        ..default()
                    },
                ));
            });

        // Spacer
        titlebar.spawn(Node {
            flex_grow: 1.,
            ..default()
        });

        // Right group: display/volume | navigation
        titlebar
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.),
                ..default()
            })
            .with_children(|right| {
                // Display & volume buttons
                for action in [
                    DeviceAction::ScreenOff,
                    DeviceAction::ScreenOn,
                    DeviceAction::VolumeDown,
                    DeviceAction::VolumeUp,
                ] {
                    let icon = match action {
                        DeviceAction::ScreenOff => screen_off_icon.clone(),
                        DeviceAction::ScreenOn => screen_on_icon.clone(),
                        DeviceAction::VolumeDown => volume_down_icon.clone(),
                        DeviceAction::VolumeUp => volume_up_icon.clone(),
                        _ => unreachable!(),
                    };
                    right
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
                            DeviceButton(action),
                        ))
                        .with_child((
                            Node {
                                width: Val::Px(14.),
                                height: Val::Px(14.),
                                ..default()
                            },
                            ImageNode::new(icon),
                        ));
                }

                // Separator
                right
                    .spawn(Node {
                        width: Val::Px(1.),
                        height: Val::Px(16.),
                        margin: UiRect::px(2., 2., 0., 0.),
                        ..default()
                    })
                    .insert(BackgroundColor(Color::srgba(0.4, 0.4, 0.4, 0.5)));

                // Navigation buttons
                for action in [
                    DeviceAction::Back,
                    DeviceAction::Home,
                    DeviceAction::AppSwitch,
                ] {
                    let icon = match action {
                        DeviceAction::Back => back_icon.clone(),
                        DeviceAction::Home => home_icon.clone(),
                        DeviceAction::AppSwitch => menu_icon.clone(),
                        _ => unreachable!(),
                    };
                    right
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
                            DeviceButton(action),
                        ))
                        .with_child((
                            Node {
                                width: Val::Px(14.),
                                height: Val::Px(14.),
                                ..default()
                            },
                            ImageNode::new(icon),
                        ));
                }
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
    commands
        .entity(root_entity)
        .add_children(&[titlebar_entity, mask_entity]);

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
            MaterialNode(video_material),
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
    button_query: Query<
        &Interaction,
        Or<(
            With<MinimizeButton>,
            With<PushpinButton>,
            With<CloseButton>,
            With<DeviceButton>,
        )>,
    >,
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
                let _ = d_tx
                    .0
                    .send(ControllerCommand::ShutdownMain(device.scid.clone()));
            }
        }
    }
}

fn handle_device_buttons(
    query: Query<(&DeviceButton, &Interaction), Changed<Interaction>>,
    cs_tx: Res<ChannelSenderCS>,
) {
    for (btn, interaction) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match btn.0 {
            DeviceAction::Back => device_action::inject_keycode(&cs_tx.0, Keycode::Back),
            DeviceAction::Home => device_action::inject_keycode(&cs_tx.0, Keycode::Home),
            DeviceAction::AppSwitch => device_action::inject_keycode(&cs_tx.0, Keycode::AppSwitch),
            DeviceAction::ScreenOff => device_action::set_display_power(&cs_tx.0, false),
            DeviceAction::ScreenOn => device_action::set_display_power(&cs_tx.0, true),
            DeviceAction::VolumeUp => device_action::inject_keycode(&cs_tx.0, Keycode::VolumeUp),
            DeviceAction::VolumeDown => {
                device_action::inject_keycode(&cs_tx.0, Keycode::VolumeDown)
            }
        }
    }
}

const NORMAL_BG: Color = Color::srgba(0.25, 0.25, 0.25, 0.6);
const HOVERED_BG: Color = Color::srgba(0.38, 0.38, 0.38, 0.7);
const PRESSED_BG: Color = Color::srgba(0.15, 0.15, 0.15, 0.85);

const MAC_CLOSE_BG: Color = Color::srgba(1.0, 0.373, 0.341, 1.0);
const MAC_CLOSE_HOVER_BG: Color = Color::srgba(1.0, 0.52, 0.49, 1.0);
const MAC_CLOSE_PRESSED_BG: Color = Color::srgba(0.85, 0.23, 0.20, 1.0);
const MAC_MINIMIZE_BG: Color = Color::srgba(1.0, 0.737, 0.180, 1.0);
const MAC_MINIMIZE_HOVER_BG: Color = Color::srgba(1.0, 0.82, 0.35, 1.0);
const MAC_MINIMIZE_PRESSED_BG: Color = Color::srgba(0.85, 0.60, 0.10, 1.0);
const MAC_PIN_BG: Color = Color::srgba(0.157, 0.784, 0.251, 1.0);
const MAC_PIN_HOVER_BG: Color = Color::srgba(0.28, 0.86, 0.35, 1.0);
const MAC_PIN_PRESSED_BG: Color = Color::srgba(0.10, 0.65, 0.18, 1.0);
const MAC_PIN_INACTIVE_BG: Color = Color::srgba(0.157, 0.784, 0.251, 0.4);

fn button_interaction(
    window: Single<&Window>,
    minimize_query: Query<(Entity, &Interaction), (With<MinimizeButton>, Changed<Interaction>)>,
    pushpin_query: Query<(Entity, &Interaction), (With<PushpinButton>, Changed<Interaction>)>,
    close_query: Query<(Entity, &Interaction), (With<CloseButton>, Changed<Interaction>)>,
    device_btn_query: Query<(Entity, &Interaction), (With<DeviceButton>, Changed<Interaction>)>,
    mut bg_query: Query<&mut BackgroundColor>,
) {
    for (entity, interaction) in minimize_query.iter() {
        if let Ok(mut bg) = bg_query.get_mut(entity) {
            *bg = match *interaction {
                Interaction::Pressed => MAC_MINIMIZE_PRESSED_BG,
                Interaction::Hovered => MAC_MINIMIZE_HOVER_BG,
                Interaction::None => MAC_MINIMIZE_BG,
            }
            .into();
        }
    }
    let pinned = window.window_level == WindowLevel::AlwaysOnTop;
    for (entity, interaction) in pushpin_query.iter() {
        if let Ok(mut bg) = bg_query.get_mut(entity) {
            *bg = if pinned {
                match *interaction {
                    Interaction::Pressed => MAC_PIN_PRESSED_BG,
                    Interaction::Hovered => MAC_PIN_HOVER_BG,
                    Interaction::None => MAC_PIN_BG,
                }
            } else {
                match *interaction {
                    Interaction::Pressed => MAC_PIN_PRESSED_BG,
                    Interaction::Hovered => MAC_PIN_HOVER_BG,
                    Interaction::None => MAC_PIN_INACTIVE_BG,
                }
            }
            .into();
        }
    }
    for (entity, interaction) in close_query.iter() {
        if let Ok(mut bg) = bg_query.get_mut(entity) {
            *bg = match *interaction {
                Interaction::Pressed => MAC_CLOSE_PRESSED_BG,
                Interaction::Hovered => MAC_CLOSE_HOVER_BG,
                Interaction::None => MAC_CLOSE_BG,
            }
            .into();
        }
    }
    for (entity, interaction) in device_btn_query.iter() {
        if let Ok(mut bg) = bg_query.get_mut(entity) {
            *bg = match *interaction {
                Interaction::Pressed => PRESSED_BG,
                Interaction::Hovered => HOVERED_BG,
                Interaction::None => NORMAL_BG,
            }
            .into();
        }
    }
}

fn map_handle_for_device(
    handle: CompassOctant,
    orientation: DeviceOrientation,
) -> Option<CompassOctant> {
    match orientation {
        DeviceOrientation::Landscape => match handle {
            CompassOctant::NorthWest | CompassOctant::West | CompassOctant::SouthWest => {
                Some(CompassOctant::West)
            }
            CompassOctant::NorthEast | CompassOctant::East | CompassOctant::SouthEast => {
                Some(CompassOctant::East)
            }
            CompassOctant::North | CompassOctant::South => None,
        },
        DeviceOrientation::Portrait => match handle {
            CompassOctant::NorthWest | CompassOctant::North | CompassOctant::NorthEast => {
                Some(CompassOctant::North)
            }
            CompassOctant::SouthWest | CompassOctant::South | CompassOctant::SouthEast => {
                Some(CompassOctant::South)
            }
            CompassOctant::East | CompassOctant::West => None,
        },
    }
}

fn handle_resize(
    mut window: Single<&mut Window>,
    mut resize_state: ResMut<MaskResizeState>,
    query: Query<(&ResizeHandle, &Interaction), Changed<Interaction>>,
) {
    let device = ControlledDevice::get_main_device_blocking();

    for (handle, interaction) in query.iter() {
        if *interaction == Interaction::Pressed {
            let direction = match &device {
                Some(dev) if dev.device_size.0 > 0 && dev.device_size.1 > 0 => {
                    let orientation =
                        DeviceOrientation::from_size(dev.device_size.0, dev.device_size.1);
                    map_handle_for_device(handle.0, orientation)
                }
                _ => None, // no device or unknown size: block resize
            };
            if let Some(dir) = direction {
                resize_state.begin_interaction();
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
            *bg = if pinned {
                MAC_PIN_BG
            } else {
                MAC_PIN_INACTIVE_BG
            }
            .into();
        }
    }
}
