use bevy::prelude::*;
use rust_i18n::t;

use crate::{
    config::LocalConfig,
    mask::{
        mapping::{
            MappingState,
            config::{ActiveMappingConfig, BindMappingType},
        },
        mask_command::MaskSize,
        ui::basic::MaskContentEntity,
    },
};

pub struct MappingLabelPlugin;

impl Plugin for MappingLabelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_label_opacity)
            .add_systems(
                Update,
                (
                    sync_label_opacity,
                    redraw_normal_mapping_label.run_if(resource_changed::<ActiveMappingConfig>),
                    update_labels,
                ),
            )
            .add_systems(OnEnter(MappingState::RawInput), redraw_raw_input_label)
            .add_systems(OnExit(MappingState::RawInput), redraw_normal_mapping_label);
    }
}

fn init_label_opacity(mut commands: Commands) {
    commands.insert_resource(LabelOpacity(LocalConfig::get().mapping_label_opacity));
}

fn sync_label_opacity(mut opacity: ResMut<LabelOpacity>) {
    let current = LocalConfig::get().mapping_label_opacity;
    if (opacity.0 - current).abs() > f32::EPSILON {
        opacity.0 = current;
    }
}

fn redraw_raw_input_label(
    mut commands: Commands,
    query: Query<(Entity, &MappingLabel)>,
    mask_size: Res<MaskSize>,
    mask_content: Res<MaskContentEntity>,
    opacity: Res<LabelOpacity>,
) {
    for (entity, _) in query.iter() {
        commands.entity(entity).despawn();
    }

    create_simple_label(
        &mut commands,
        "M-Right",
        (25., 25.).into(),
        mask_size.0,
        mask_content.0,
        opacity.0,
    );
}

fn redraw_normal_mapping_label(
    mut commands: Commands,
    query: Query<(Entity, &MappingLabel)>,
    active_mapping: Res<ActiveMappingConfig>,
    mask_content: Res<MaskContentEntity>,
    opacity: Res<LabelOpacity>,
) {
    for (entity, _) in query.iter() {
        commands.entity(entity).despawn();
    }

    if let Some(config) = &active_mapping.0 {
        config
            .get_mapping_label_info()
            .into_iter()
            .for_each(|(mapping, binding, pos, size)| {
                if binding.is_empty() {
                    match mapping {
                        BindMappingType::DirectionPad(mapping_direction_pad) => {
                            create_pad_label(
                                &mut commands,
                                mapping_direction_pad
                                    .bind
                                    .to_string_vec()
                                    .iter()
                                    .map(|s| s.as_str())
                                    .collect(),
                                pos,
                                size,
                                mask_content.0,
                                opacity.0,
                            );
                        }
                        BindMappingType::PadCastSpell(mapping_pad_cast_spell) => {
                            let mut bindings = mapping_pad_cast_spell.pad_bind.to_string_vec();
                            bindings.push(mapping_pad_cast_spell.bind.to_string());
                            create_pad_label(
                                &mut commands,
                                bindings.iter().map(|s| s.as_ref()).collect(),
                                pos,
                                size,
                                mask_content.0,
                                opacity.0,
                            )
                        }
                        _ => {}
                    }
                } else {
                    create_simple_label(
                        &mut commands,
                        &binding,
                        pos,
                        size,
                        mask_content.0,
                        opacity.0,
                    );
                }
            });
    }
}

fn update_labels(
    mask_size: Res<MaskSize>,
    opacity: Res<LabelOpacity>,
    window: Single<&Window>,
    mut query: Query<(
        &MappingLabel,
        &mut BackgroundColor,
        &mut Node,
        &ComputedNode,
        &Children,
    )>,
    mut text_query: Query<&mut TextColor>,
    mut child_query: Query<&Children>,
) {
    for (label, mut bg, mut node, cp_node, node_children) in query.iter_mut() {
        let node_size = cp_node.size();

        let scale = window.scale_factor();
        let new_pos =
            label.original_pos / label.original_size * mask_size.0 - node_size / scale / 2.;
        node.left = Val::Px(new_pos.x);
        node.top = Val::Px(new_pos.y);

        bg.0 = label_background_color(opacity.0);
        for child in node_children.iter() {
            if let Ok(mut text_color) = text_query.get_mut(child) {
                text_color.0 = label_text_color(opacity.0);
            }

            if let Ok(children) = child_query.get_mut(child) {
                for child in children.iter() {
                    if let Ok(mut text_color) = text_query.get_mut(child) {
                        text_color.0 = label_text_color(opacity.0);
                    }
                }
            }
        }
    }
}

#[derive(Resource)]
pub struct LabelOpacity(f32);

#[derive(Component)]
struct MappingLabel {
    original_pos: Vec2,
    original_size: Vec2,
}

#[derive(Component)]
enum MappingLabelType {
    Simple,
    Pad,
}

#[derive(Bundle)]
struct MappingLabelBundle {
    label: MappingLabel,
    label_type: MappingLabelType,
    node: Node,
    background_color: BackgroundColor,
}

fn create_simple_label(
    commands: &mut Commands,
    binding: &str,
    original_pos: Vec2,
    original_size: Vec2,
    parent_entity: Entity,
    opacity: f32,
) {
    commands.entity(parent_entity).with_children(|parent| {
        parent.spawn((
            MappingLabelBundle {
                label: MappingLabel {
                    original_pos,
                    original_size,
                },
                label_type: MappingLabelType::Simple,
                node: Node {
                    position_type: PositionType::Absolute,
                    padding: UiRect::px(5., 5., 3., 3.),
                    border_radius: BorderRadius::all(Val::Px(3.)),
                    ..default()
                },
                background_color: BackgroundColor(label_background_color(opacity)),
            },
            children![(
                Text::new(binding),
                TextFont {
                    font_size: FontSize::Px(12.),
                    ..default()
                },
                TextColor(label_text_color(opacity)),
            )],
        ));
    });
}

fn text_node(binding: &str, opacity: f32) -> (Text, TextFont, TextColor) {
    (
        Text::new(binding),
        TextFont {
            font_size: FontSize::Px(12.),
            ..default()
        },
        TextColor(label_text_color(opacity)),
    )
}

fn label_background_color(opacity: f32) -> Color {
    Color::linear_rgba(0., 0., 0., opacity)
}

fn label_text_color(opacity: f32) -> Color {
    Color::linear_rgba(1., 1., 1., opacity)
}

fn create_pad_label(
    commands: &mut Commands,
    bindings: Vec<&str>,
    original_pos: Vec2,
    original_size: Vec2,
    parent_entity: Entity,
    opacity: f32,
) {
    let mut pad_node = Node {
        position_type: PositionType::Absolute,
        width: Val::Px(125.),
        height: Val::Px(125.),
        padding: UiRect::all(Val::Px(5.)),
        box_sizing: BoxSizing::BorderBox,
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border_radius: BorderRadius::all(Val::Percent(50.)),
        ..default()
    };

    let children = match bindings.len() {
        1 => vec![text_node(&bindings[0], opacity)],
        2 => {
            pad_node.row_gap = Val::Px(15.);
            bindings
                .iter()
                .map(|binding| text_node(binding, opacity))
                .collect()
        }
        4 | 5 => {
            pad_node.justify_content = JustifyContent::SpaceBetween;
            bindings
                .iter()
                .map(|binding| text_node(binding, opacity))
                .collect()
        }
        _ => panic!(
            "{}",
            t!("mask.padLabelNotSupported", count => bindings.len())
        ),
    };

    commands.entity(parent_entity).with_children(|parent| {
        parent
            .spawn(MappingLabelBundle {
                label: MappingLabel {
                    original_pos,
                    original_size,
                },
                label_type: MappingLabelType::Pad,
                node: pad_node,
                background_color: BackgroundColor(label_background_color(opacity)),
            })
            .with_children(|parent| {
                match children.len() {
                    4 => {
                        let [top, bottom, left, right]: [_; 4] = children.try_into().unwrap();
                        parent.spawn((
                            Node {
                                width: Val::Percent(100.),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            children![top],
                        ));
                        parent.spawn((
                            Node {
                                width: Val::Percent(100.),
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },
                            children![left, right],
                        ));
                        parent.spawn((
                            Node {
                                width: Val::Percent(100.),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            children![bottom],
                        ));
                    }
                    5 => {
                        let [top, bottom, left, right, center]: [_; 5] =
                            children.try_into().unwrap();
                        parent.spawn((
                            Node {
                                width: Val::Percent(100.),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            children![top],
                        ));
                        parent.spawn((
                            Node {
                                width: Val::Percent(100.),
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },
                            children![left, center, right],
                        ));
                        parent.spawn((
                            Node {
                                width: Val::Percent(100.),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            children![bottom],
                        ));
                    }
                    _ => {
                        for child in children {
                            parent.spawn(child);
                        }
                    }
                };
            });
    });
}
