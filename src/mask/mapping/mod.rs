pub mod binding;
pub mod cast_spell;
pub mod config;
pub mod cursor;
pub mod direction_pad;
pub mod fire;
pub mod observation;
pub mod raw_input;
pub mod script;
pub mod script_helper;
pub mod swipe;
pub mod tap;
pub mod utils;

use bevy::prelude::*;
use bevy_ineffable::prelude::*;
use rust_i18n::t;

use crate::{
    config::LocalConfig,
    mask::mapping::{
        config::{
            ActiveMappingConfig, BindMappingConfig, MappingAction, default_mapping_config,
            load_mapping_config, save_mapping_config,
        },
        cursor::{CursorPlugins, CursorState},
    },
    utils::{relate_to_data_path},
};

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
pub enum MappingState {
    #[default]
    Stop,
    Normal,
    RawInput,
}

pub struct MappingPlugins;

impl Plugin for MappingPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((IneffablePlugin, CursorPlugins))
            .insert_state(MappingState::Stop)
            .insert_resource(ActiveMappingConfig(None, String::new()))
            .register_input_action::<MappingAction>()
            .add_systems(
                Startup,
                (
                    init,
                    tap::tap_init,
                    direction_pad::direction_pad_init,
                    fire::fire_init,
                    cast_spell::cast_spell_init,
                    observation::init_observation,
                    raw_input::raw_input_init,
                    script::script_init,
                ),
            )
            // normal mapping mode
            .add_systems(
                Update,
                (
                    tap::handle_single_tap,
                    tap::handle_repeat_tap,
                    tap::handle_repeat_tap_trigger,
                    tap::handle_multiple_tap,
                    swipe::handle_swipe,
                    direction_pad::handle_direction_pad,
                    cast_spell::handle_mouse_cast_spell,
                    cast_spell::handle_mouse_cast_spell_trigger,
                    cast_spell::handle_cancel_cast,
                    cast_spell::handle_pad_cast_spell,
                    cast_spell::handle_pad_cast_spell_trigger,
                    observation::handle_observation,
                    observation::handle_observation_trigger,
                    fire::handle_fps,
                    // raw input won't work in fps mode
                    raw_input::handle_raw_input.run_if(not(in_state(CursorState::Fps))),
                    // fire only works in fps mode
                    (fire::handle_fire, fire::handle_fire_trigger)
                        .run_if(in_state(CursorState::Fps)),
                    script::handle_script,
                    script::handle_script_trigger,
                )
                    .run_if(in_state(MappingState::Normal)),
            )
            // handlers in raw input mode
            .add_systems(
                Update,
                (
                    raw_input::handle_raw_input_trigger,
                    raw_input::handle_exit_raw_input_mode,
                )
                    .run_if(in_state(MappingState::RawInput).and(not(in_state(CursorState::Fps)))),
            )
            .add_systems(
                OnEnter(MappingState::RawInput),
                raw_input::on_enter_raw_input_mode,
            )
            .add_systems(
                OnExit(MappingState::RawInput),
                raw_input::on_exit_raw_input_mode,
            );
    }
}

fn init(mut ineffable: IneffableCommands, mut active_mapping: ResMut<ActiveMappingConfig>) {
    let config = LocalConfig::get();

    let (bind_mapping_config, input_config, file) =
        match load_mapping_config(&config.active_mapping_file) {
            Ok((mapping_config, input_config)) => {
                log::info!(
                    "[Mask] {}: {}",
                    t!("mask.mapping.usingMappingConfig"),
                    config.active_mapping_file,
                );
                (mapping_config, input_config, config.active_mapping_file)
            }
            Err(e) => {
                log::error!("{}", e);
                log::info!(
                    "[Mask] {}: default.json",
                    t!("mask.mapping.useDefaultMapping")
                );
                let default_mapping = default_mapping_config();
                let config_path = relate_to_data_path(["mapping", "default.json"]);
                save_mapping_config(&default_mapping, &config_path).unwrap();
                LocalConfig::set_active_mapping_file("default.json".to_string());
                let default_bind_mapping: BindMappingConfig = default_mapping.into();
                let input_config: InputConfig = InputConfig::from(&default_bind_mapping);
                (
                    default_bind_mapping,
                    input_config,
                    "default.json".to_string(),
                )
            }
        };
    active_mapping.0 = Some(bind_mapping_config);
    active_mapping.1 = file;
    ineffable.set_config(&input_config);
}
