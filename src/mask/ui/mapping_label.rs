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
    },
};

pub struct MappingLabelPlugin;

impl Plugin for MappingLabelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RedrawMappingLabel(0))
            .add_systems(Startup, init_label_opacity)
            .add_systems(
                Update,
                (
                    redraw_normal_mapping_label.run_if(resource_changed::<ActiveMappingConfig>),
                    update_labels.run_if(
                        resource_changed::<MaskSize>
                            .or(resource_changed::<LabelOpacity>)
                            .or(resource_changed::<RedrawMappingLabel>),
                    ),
                ),
            )
            .add_systems(OnEnter(MappingState::RawInput), redraw_raw_input_label)
            .add_systems(OnExit(MappingState::RawInput), redraw_normal_mapping_label);
    }
}

fn init_label_opacity(mut commands: Commands) {
    commands.insert_resource(LabelOpacity(LocalConfig::get().mapping_label_opacity));
}

fn redraw_raw_input_label(
    mut commands: Commands,
    query: Query<(Entity, &Label)>,
    mask_size: Res<MaskSize>,
    mut redraw_mapping_label: ResMut<RedrawMappingLabel>,
) {
    for (entity, _) in query.iter() {
        commands.entity(entity).despawn();
    }

    create_simple_label(&mut commands, "M-Right", (25., 25.).into(), mask_size.0);
    redraw_mapping_label.0 += 1;
}

#[derive(Resource)]
pub struct RedrawMappingLabel(u32);

fn redraw_normal_mapping_label(
    mut commands: Commands,
    query: Query<(Entity, &Label)>,
    active_mapping: Res<ActiveMappingConfig>,
    mut redraw_mapping_label: ResMut<RedrawMappingLabel>,
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
                            )
                        }
                        _ => {}
                    }
                } else {
                    create_simple_label(&mut commands, &binding, pos, size);
                }
            });
        redraw_mapping_label.0 += 1;
    }
}

fn update_labels(
    mask_size: Res<MaskSize>,
    opacity: Res<LabelOpacity>,
    window: Single<&Window>,
    mut query: Query<(
        &Label,
        &mut BackgroundColor,
        &mut Node,
        &ComputedNode,
        &Children,
    )>,
    mut text_query: Query<&mut TextColor>,
    mut child_query: Query<&Children>,
    mut redraw_mapping_label: ResMut<RedrawMappingLabel>,
) {
    let mut updated_flag = false;

    for (label, mut bg, mut node, cp_node, node_children) in query.iter_mut() {
        let node_size = cp_node.size();
        if node_size != Vec2::ZERO {
            updated_flag = true;
        }

        let scale = window.scale_factor();
        let new_pos =
            label.original_pos / label.original_size * mask_size.0 - node_size / scale / 2.;
        node.left = Val::Px(new_pos.x);
        node.top = Val::Px(new_pos.y);

        bg.0 = Color::linear_rgba(0., 0., 0., opacity.0);
        for child in node_children.iter() {
            if let Ok(mut text_color) = text_query.get_mut(child) {
                text_color.0 = Color::linear_rgba(1., 1., 1., opacity.0);
            }

            if let Ok(children) = child_query.get_mut(child) {
                for child in children.iter() {
                    if let Ok(mut text_color) = text_query.get_mut(child) {
                        text_color.0 = Color::linear_rgba(1., 1., 1., opacity.0);
                    }
                }
            }
        }
    }

    if !updated_flag {
        redraw_mapping_label.0 += 1;
    }
}

#[derive(Resource)]
pub struct LabelOpacity(f32);

#[derive(Component)]
struct Label {
    original_pos: Vec2,
    original_size: Vec2,
}

#[derive(Component)]
enum LabelType {
    Simple,
    Pad,
}

fn create_simple_label(
    commands: &mut Commands,
    binding: &str,
    original_pos: Vec2,
    original_size: Vec2,
) {
    commands.spawn((
        Label {
            original_pos,
            original_size,
        },
        LabelType::Simple,
        Node {
            position_type: PositionType::Absolute,
            padding: UiRect::px(5., 5., 3., 3.),
            ..default()
        },
        BorderRadius::all(Val::Px(3.)),
        BackgroundColor(Color::BLACK),
        children![(
            Text::new(binding),
            TextFont {
                font_size: 12.,
                ..default()
            },
            TextColor(Color::WHITE),
        )],
    ));
}

fn text_node(binding: &str) -> (Text, TextFont, TextColor) {
    (
        Text::new(binding),
        TextFont {
            font_size: 12.,
            ..default()
        },
        TextColor(Color::WHITE),
    )
}

fn create_pad_label(
    commands: &mut Commands,
    bindings: Vec<&str>,
    original_pos: Vec2,
    original_size: Vec2,
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
        ..default()
    };

    let children = match bindings.len() {
        1 => vec![text_node(&bindings[0])],
        2 => {
            pad_node.row_gap = Val::Px(15.);
            bindings.iter().map(|binding| text_node(binding)).collect()
        }
        4 | 5 => {
            pad_node.justify_content = JustifyContent::SpaceBetween;
            bindings.iter().map(|binding| text_node(binding)).collect()
        }
        _ => panic!(
            "{}",
            t!("mask.padLabelNotSupported", count => bindings.len())
        ),
    };

    commands
        .spawn((
            Label {
                original_pos,
                original_size,
            },
            LabelType::Pad,
            pad_node,
            BorderRadius::all(Val::Percent(50.)),
            BackgroundColor(Color::BLACK),
        ))
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
                    let [top, bottom, left, right, center]: [_; 5] = children.try_into().unwrap();
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
}
