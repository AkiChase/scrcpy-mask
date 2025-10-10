use std::{
    collections::HashMap,
    fs::{File, create_dir_all},
    io::Write,
    path::Path,
    str::FromStr,
};

use bevy::{ecs::resource::Resource, math::Vec2};
use bevy_ineffable::{
    config::InputConfig,
    phantom::IAWrp,
    prelude::{InputAction, InputBinding},
};
use paste::paste;
use rust_i18n::t;
use seq_macro::seq;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use strum_macros::{AsRefStr, Display, EnumString};

use crate::{
    mask::mapping::{
        binding::ValidateMappingConfig,
        cast_spell::{
            BindMappingCancelCast, BindMappingMouseCastSpell, BindMappingPadCastSpell,
            MappingCancelCast, MappingMouseCastSpell, MappingPadCastSpell,
        },
        direction_pad::{BindMappingDirectionPad, MappingDirectionPad},
        fire::{BindMappingFire, BindMappingFps, MappingFire, MappingFps},
        observation::{BindMappingObservation, MappingObservation},
        raw_input::{BindMappingRawInput, MappingRawInput},
        script::{BindMappingScript, MappingScript},
        swipe::{BindMappingSwipe, MappingSwipe},
        tap::{
            BindMappingMultipleTap, BindMappingRepeatTap, BindMappingSingleTap, MappingMultipleTap,
            MappingRepeatTap, MappingSingleTap,
        },
        utils::Size,
    },
    utils::{is_safe_file_name, relate_to_data_path},
};

// declare 32 actions for each kind of key mapping
seq!(N in 1..=32 {
    #[derive(InputAction, Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, AsRefStr, Display, EnumString)]
    pub enum MappingAction {
        #(
            #[ineffable(continuous)]
            SingleTap~N,
            #[ineffable(continuous)]
            RepeatTap~N,
            #[ineffable(pulse)]
            MultipleTap~N,
            #[ineffable(pulse)]
            Swipe~N,
            #[ineffable(dual_axis)]
            DirectionPad~N,
            #[ineffable(continuous)]
            MouseCastSpell~N,
            #[ineffable(continuous)]
            PadCastSpell~N,
            #[ineffable(dual_axis)]
            PadCastDirection~N,
            #[ineffable(pulse)]
            CancelCast~N,
            #[ineffable(continuous)]
            Observation~N,
            #[ineffable(pulse)]
            Fps~N,
            #[ineffable(continuous)]
            Fire~N,
            #[ineffable(pulse)]
            RawInput~N,
            #[ineffable(continuous)]
            Script~N,
        )*
    }

    impl MappingAction {
        pub fn ineff_continuous(&self) -> IAWrp<MappingAction, bevy_ineffable::phantom::Continuous> {
            match self {
                #(
                    MappingAction::SingleTap~N => self.clone()._singletap~N(),
                    MappingAction::RepeatTap~N => self.clone()._repeattap~N(),
                    MappingAction::MouseCastSpell~N => self.clone()._mousecastspell~N(),
                    MappingAction::PadCastSpell~N => self.clone()._padcastspell~N(),
                    MappingAction::Observation~N => self.clone()._observation~N(),
                    MappingAction::Fire~N => self.clone()._fire~N(),
                    MappingAction::Script~N => self.clone()._script~N(),
                )*
                _ => panic!("ineff_continuous called on non-continuous variant"),
            }
        }

        pub fn ineff_pulse(&self) -> IAWrp<MappingAction, bevy_ineffable::phantom::Pulse> {
            match self {
                #(
                    MappingAction::MultipleTap~N => self.clone()._multipletap~N(),
                    MappingAction::Swipe~N => self.clone()._swipe~N(),
                    MappingAction::CancelCast~N => self.clone()._cancelcast~N(),
                    MappingAction::Fps~N => self.clone()._fps~N(),
                    MappingAction::RawInput~N => self.clone()._rawinput~N(),
                )*
                _ => panic!("ineff_pulse called on non-pulse variant"),
            }
        }

        pub fn ineff_dual_axis(&self) -> IAWrp<MappingAction, bevy_ineffable::phantom::DualAxis> {
            match self {
                #(
                    MappingAction::DirectionPad~N => self.clone()._directionpad~N(),
                    MappingAction::PadCastDirection~N => self.clone()._padcastdirection~N(),
                )*
                _ => panic!("ineff_dual_axis called on non-dual_axis variant"),
            }
        }
    }
});

macro_rules! impl_mapping_related {
    ( $($variant:ident),* $(,)? ) => {
        paste! {
            #[derive(Serialize, Deserialize, Debug, Clone, AsRefStr)]
            #[serde(tag = "type")]
            pub enum MappingType {
                $(
                    $variant([<Mapping $variant>]),
                )*
            }

            #[derive(Debug, Clone)]
            pub enum BindMappingType {
                $(
                    $variant([<BindMapping $variant>]),
                )*
            }
        }


        impl ValidateMappingConfig for MappingType {
            fn validate(&self) -> Result<(), String> {
                match self {
                    $(
                        MappingType::$variant(v) => v.validate(),
                    )*
                }
            }
        }

        impl From<MappingType> for BindMappingType {
            fn from(value: MappingType) -> Self {
                match value {
                    $(
                        MappingType::$variant(v) => Self::$variant(v.into()),
                    )*
                }
            }
        }

        impl BindMappingType {
            pub fn get_input_binding(&self) -> InputBinding {
                match self {
                    $(
                        BindMappingType::$variant(inner) => inner.input_binding.clone(),
                    )*
                }
            }

            $(
                paste! {
                    pub fn [<as_ref_ $variant:lower>](&self) -> & [<BindMapping $variant>] {
                        match self {
                            BindMappingType::$variant(inner) => inner,
                            _ => panic!(concat!("Not a ", stringify!($variant), " mapping")),
                        }
                    }
                }
            )*
        }
    };
}

impl_mapping_related! {
    SingleTap,
    RepeatTap,
    MultipleTap,
    Swipe,
    DirectionPad,
    MouseCastSpell,
    PadCastSpell,
    CancelCast,
    Observation,
    Fps,
    Fire,
    RawInput,
    Script
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingConfig {
    pub version: String,
    pub original_size: Size,
    pub mappings: Vec<MappingType>,
}

#[derive(Debug, Clone)]
pub struct BindMappingConfig {
    pub version: String,
    pub original_size: Size,
    pub mappings: HashMap<MappingAction, BindMappingType>,
}

impl From<MappingConfig> for BindMappingConfig {
    fn from(value: MappingConfig) -> Self {
        let mut mappings = HashMap::<MappingAction, BindMappingType>::new();
        let mut mapping_type_map = HashMap::<String, u32>::new();
        for mapping in value.mappings.into_iter() {
            let name = mapping.as_ref();
            let count = *mapping_type_map
                .entry(name.to_string())
                .and_modify(|c| *c += 1)
                .or_insert(1);
            let action_name = format!("{}{}", name, count);
            let action = MappingAction::from_str(&action_name).unwrap();

            if let MappingType::PadCastSpell(mapping_pad_cast_spell) = mapping {
                let pad_action_name = format!("PadCastDirection{count}");
                let mut bind_mapping: BindMappingPadCastSpell = mapping_pad_cast_spell.into();
                bind_mapping.pad_action = MappingAction::from_str(&pad_action_name).unwrap();
                mappings.insert(action, BindMappingType::PadCastSpell(bind_mapping));
            } else {
                mappings.insert(action, mapping.into());
            }
        }

        Self {
            version: value.version,
            original_size: value.original_size,
            mappings,
        }
    }
}

impl BindMappingConfig {
    pub fn get_mapping_label_info(&self) -> Vec<(&BindMappingType, String, Vec2, Vec2)> {
        let size: Vec2 = self.original_size.into();
        self.mappings
            .iter()
            .map(|(_, mapping)| {
                let (binding, pos): (String, Vec2) = match mapping {
                    BindMappingType::SingleTap(m) => (m.bind.to_string(), m.position.into()),
                    BindMappingType::RepeatTap(m) => (m.bind.to_string(), m.position.into()),
                    BindMappingType::MultipleTap(m) => {
                        (m.bind.to_string(), m.items[0].position.into())
                    }
                    BindMappingType::Swipe(m) => (m.bind.to_string(), m.positions[0].into()),
                    BindMappingType::DirectionPad(m) => (String::new(), m.position.into()),
                    BindMappingType::MouseCastSpell(m) => (m.bind.to_string(), m.position.into()),
                    BindMappingType::PadCastSpell(m) => (String::new(), m.position.into()),
                    BindMappingType::CancelCast(m) => (m.bind.to_string(), m.position.into()),
                    BindMappingType::Observation(m) => (m.bind.to_string(), m.position.into()),
                    BindMappingType::Fps(m) => (m.bind.to_string(), m.position.into()),
                    BindMappingType::Fire(m) => (m.bind.to_string(), m.position.into()),
                    BindMappingType::RawInput(m) => (m.bind.to_string(), m.position.into()),
                    BindMappingType::Script(m) => (m.bind.to_string(), m.position.into()),
                };
                (mapping, binding, pos, size)
            })
            .collect()
    }
}

impl From<&BindMappingConfig> for InputConfig {
    fn from(mapping_config: &BindMappingConfig) -> Self {
        let mut all_bindings: HashMap<String, Vec<InputBinding>> = HashMap::new();

        for (action, mapping) in &mapping_config.mappings {
            if let BindMappingType::PadCastSpell(m) = mapping {
                all_bindings.insert(action.to_string(), vec![m.input_binding.clone()]);
                all_bindings.insert(m.pad_action.to_string(), vec![m.pad_input_binding.clone()]);
            } else {
                all_bindings.insert(action.to_string(), vec![mapping.get_input_binding()]);
            }
        }

        let binding_config: HashMap<String, HashMap<String, Vec<InputBinding>>> =
            HashMap::from([("MappingAction".to_string(), all_bindings)]);
        let mut input_config = InputConfig::new();
        input_config.bindings = binding_config;
        input_config
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ActiveMappingConfig(pub Option<BindMappingConfig>, pub String);

pub fn default_mapping_config() -> MappingConfig {
    MappingConfig {
        version: "0.0.1".to_string(),
        original_size: Size {
            width: 2560,
            height: 1440,
        },
        mappings: vec![],
    }
}

pub fn validate_mapping_config(mapping_config: &MappingConfig) -> Result<(), String> {
    let mut validate_errors = Vec::<String>::new();

    let mut mapping_type_map = HashMap::<String, u32>::new();
    for mapping in mapping_config.mappings.iter() {
        let name = mapping.as_ref();
        let count = *mapping_type_map
            .entry(name.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);
        if count > 32 {
            validate_errors.push(
                t!(
                    "mask.mapping.mappingActionExceedsMaxCount",
                    name => name,
                    count => count,
                    max => 32
                )
                .to_string(),
            );
        }

        if let Err(e) = mapping.validate() {
            validate_errors.push(format!("[{name}-{count}] {e}"));
        }
    }

    if !validate_errors.is_empty() {
        let mut validate_errors: Vec<String> = validate_errors
            .into_iter()
            .enumerate()
            .map(|(i, err)| format!("{}. {}", i + 1, err))
            .collect();
        validate_errors.insert(
            0,
            t!("mask.mapping.mappingConfigValidationFailed").to_string(),
        );
        return Err(validate_errors.join("\n"));
    }
    Ok(())
}

pub fn load_mapping_config(
    file_name: impl AsRef<str>,
) -> Result<(BindMappingConfig, InputConfig), String> {
    if !is_safe_file_name(file_name.as_ref()) {
        return Err(format!(
            "{}: {}",
            t!("mask.mapping.fileNameNotSafe"),
            file_name.as_ref()
        ));
    }

    // load from file
    let path = relate_to_data_path(["mapping", file_name.as_ref()]);
    if !path.exists() {
        return Err(format!(
            "{}: {}",
            t!("mask.mapping.mappingConfigNotFound"),
            file_name.as_ref()
        ));
    }

    let config_string = std::fs::read_to_string(path)
        .map_err(|e| format!("{}: {}", t!("web.mapping.cannotReadMappingConfig"), e))?;
    let mapping_config: MappingConfig = serde_json::from_str(&config_string)
        .map_err(|e| format!("{}: {}", t!("web.mapping.cannotDeserializeConfig"), e))?;

    validate_mapping_config(&mapping_config)?;

    let bind_mapping_config: BindMappingConfig = mapping_config.into();
    let input_config: InputConfig = InputConfig::from(&bind_mapping_config);
    Ok((bind_mapping_config, input_config))
}

pub fn save_mapping_config(config: &MappingConfig, path: &Path) -> Result<(), String> {
    let json_string = to_string_pretty(config)
        .map_err(|e| format!("{}: {}", t!("web.mapping.cannotDeserializeConfig"), e))?;
    if let Some(parent) = path.parent() {
        create_dir_all(parent)
            .map_err(|e| format!("{}: {}", t!("mask.mapping.cannotCreateConfigDir"), e))?;
    }

    let mut file = File::create(path)
        .map_err(|e| format!("{}: {}", t!("mask.mapping.cannotCreateMappingConfig"), e))?;
    file.write_all(json_string.as_bytes())
        .map_err(|e| format!("{}: {}", t!("mask.mapping.cannotWriteMappingConfig"), e))?;

    Ok(())
}
