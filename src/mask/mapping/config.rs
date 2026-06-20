use std::{
    collections::{HashMap, HashSet},
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
        cursor::FPS_MARGIN,
        direction_pad::{BindMappingDirectionPad, MappingDirectionPad},
        fire::{BindMappingFire, BindMappingFps, MappingFire, MappingFps},
        observation::{BindMappingObservation, MappingObservation},
        raw_input::{BindMappingRawInput, MappingRawInput},
        script::{BindMappingScript, MappingScript, MappingScriptHooks},
        script_helper::{ScriptAST, ScriptDiagnostic},
        swipe::{BindMappingSwipe, MappingSwipe},
        tap::{
            BindMappingMultipleTap, BindMappingRepeatTap, BindMappingSingleTap, MappingMultipleTap,
            MappingRepeatTap, MappingSingleTap,
        },
        utils::Size,
    },
    utils::{is_safe_file_name, relate_to_data_path},
};

pub fn default_mapping_id() -> String {
    format!("{:08x}", rand::random::<u32>())
}

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

        impl MappingType {
            pub fn id(&self) -> &str {
                match self {
                    $(
                        MappingType::$variant(inner) => inner.id.as_str(),
                    )*
                }
            }
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_diagnostic: Option<ScriptDiagnostic>,
}

impl MappingDiagnostic {
    fn config(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: "error".to_string(),
            code: code.into(),
            message: message.into(),
            mapping_type: None,
            mapping_index: None,
            mapping_id: None,
            field: None,
            script_diagnostic: None,
        }
    }

    fn mapping(
        code: impl Into<String>,
        message: impl Into<String>,
        mapping_type: &str,
        mapping_index: usize,
        mapping_id: &str,
    ) -> Self {
        Self {
            severity: "error".to_string(),
            code: code.into(),
            message: message.into(),
            mapping_type: Some(mapping_type.to_string()),
            mapping_index: Some(mapping_index),
            mapping_id: Some(mapping_id.to_string()),
            field: None,
            script_diagnostic: None,
        }
    }

    fn script(
        mapping_type: &str,
        mapping_index: usize,
        mapping_id: &str,
        field: &str,
        diagnostic: ScriptDiagnostic,
    ) -> Self {
        Self {
            severity: "error".to_string(),
            code: "mapping.script.invalid".to_string(),
            message: format!("Script field '{field}' has errors."),
            mapping_type: Some(mapping_type.to_string()),
            mapping_index: Some(mapping_index),
            mapping_id: Some(mapping_id.to_string()),
            field: Some(field.to_string()),
            script_diagnostic: Some(diagnostic),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BindMappingConfig {
    pub version: String,
    pub original_size: Size,
    pub mappings: HashMap<MappingAction, BindMappingType>,
    pub mapping_id_actions: HashMap<String, MappingAction>,
}

impl From<MappingConfig> for BindMappingConfig {
    fn from(value: MappingConfig) -> Self {
        let mut mappings = HashMap::<MappingAction, BindMappingType>::new();
        let mut mapping_id_actions = HashMap::<String, MappingAction>::new();
        let mut mapping_type_map = HashMap::<String, u32>::new();
        for mapping in value.mappings.into_iter() {
            let name = mapping.as_ref();
            let id = mapping.id().to_string();
            let count = *mapping_type_map
                .entry(name.to_string())
                .and_modify(|c| *c += 1)
                .or_insert(1);
            let action_name = format!("{}{}", name, count);
            let action = MappingAction::from_str(&action_name).unwrap();
            mapping_id_actions.insert(id, action.clone());

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
            mapping_id_actions,
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

// Validate mapping config:
pub fn validate_mapping_config_diagnostics(
    mapping_config: &MappingConfig,
) -> Vec<MappingDiagnostic> {
    let mut diagnostics = Vec::<MappingDiagnostic>::new();
    let mut mapping_ids = HashSet::<String>::new();

    if mapping_config.original_size.width == 0 || mapping_config.original_size.height == 0 {
        diagnostics.push(MappingDiagnostic::config(
            "mapping.config.invalidSize",
            format!(
                "{}: {}x{}",
                t!("web.mapping.invalidSize"),
                mapping_config.original_size.width,
                mapping_config.original_size.height
            ),
        ));
    }

    let mut mapping_type_map = HashMap::<String, u32>::new();
    let fps_touch_pointer_ids: HashSet<u64> = mapping_config
        .mappings
        .iter()
        .filter_map(|mapping| match mapping {
            MappingType::Fps(mapping) => Some(mapping),
            _ => None,
        })
        .flat_map(|mapping| {
            std::iter::once(mapping.pointer_id).chain(mapping.touch_mode.another_pointer_id())
        })
        .collect();
    for mapping in mapping_config.mappings.iter() {
        let mapping_type = mapping.as_ref();
        let count = *mapping_type_map
            .entry(mapping_type.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);
        let mapping_index = count as usize;
        let id = mapping.id();

        if count > 32 {
            diagnostics.push(MappingDiagnostic::mapping(
                "mapping.config.tooManyMappings",
                t!(
                    "mask.mapping.mappingActionExceedsMaxCount",
                    name => mapping_type,
                    count => count,
                    max => 32
                )
                .to_string(),
                mapping_type,
                mapping_index,
                id,
            ));
        }

        if id.trim().is_empty() {
            diagnostics.push(MappingDiagnostic::mapping(
                "mapping.config.emptyId",
                "Mapping id cannot be empty.",
                mapping_type,
                mapping_index,
                id,
            ));
        } else if !mapping_ids.insert(id.to_string()) {
            diagnostics.push(MappingDiagnostic::mapping(
                "mapping.config.duplicateId",
                format!("Duplicate mapping id: {id}."),
                mapping_type,
                mapping_index,
                id,
            ));
        }

        collect_mapping_specific_diagnostics(
            &mut diagnostics,
            mapping,
            &fps_touch_pointer_ids,
            mapping_type,
            mapping_index,
            id,
        );
    }

    diagnostics
}

pub fn validate_mapping_config(mapping_config: &MappingConfig) -> Result<(), String> {
    let validate_errors = validate_mapping_config_diagnostics(mapping_config);

    if !validate_errors.is_empty() {
        let mut validate_errors: Vec<String> = validate_errors
            .into_iter()
            .enumerate()
            .map(|(i, err)| format!("{}. {}", i + 1, format_mapping_diagnostic(&err)))
            .collect();
        validate_errors.insert(
            0,
            t!("mask.mapping.mappingConfigValidationFailed").to_string(),
        );
        return Err(validate_errors.join("\n"));
    }
    Ok(())
}

fn format_mapping_diagnostic(diagnostic: &MappingDiagnostic) -> String {
    let mut prefix = String::new();
    if let (Some(mapping_type), Some(index)) = (&diagnostic.mapping_type, diagnostic.mapping_index)
    {
        prefix = format!("[{mapping_type}-{index}] ");
    }
    if let Some(field) = &diagnostic.field {
        if let Some(script_diagnostic) = &diagnostic.script_diagnostic {
            return format!("{prefix}{field}: {}", script_diagnostic.message);
        }
    }
    format!("{prefix}{}", diagnostic.message)
}

fn collect_mapping_specific_diagnostics(
    diagnostics: &mut Vec<MappingDiagnostic>,
    mapping: &MappingType,
    fps_touch_pointer_ids: &HashSet<u64>,
    mapping_type: &str,
    mapping_index: usize,
    mapping_id: &str,
) {
    match mapping {
        MappingType::SingleTap(mapping) => collect_script_hook_diagnostics(
            diagnostics,
            mapping_type,
            mapping_index,
            mapping_id,
            &mapping.script_hooks,
        ),
        MappingType::RepeatTap(mapping) => collect_script_hook_diagnostics(
            diagnostics,
            mapping_type,
            mapping_index,
            mapping_id,
            &mapping.script_hooks,
        ),
        MappingType::MultipleTap(mapping) => {
            if mapping.items.is_empty() {
                diagnostics.push(MappingDiagnostic::mapping(
                    "mapping.multipleTap.emptyItems",
                    "MultipleTap's operation item list is empty.",
                    mapping_type,
                    mapping_index,
                    mapping_id,
                ));
            }
            collect_script_hook_diagnostics(
                diagnostics,
                mapping_type,
                mapping_index,
                mapping_id,
                &mapping.script_hooks,
            );
        }
        MappingType::Swipe(mapping) => {
            if mapping.positions.is_empty() {
                diagnostics.push(MappingDiagnostic::mapping(
                    "mapping.swipe.emptyPositions",
                    "Swipe's position list is empty.",
                    mapping_type,
                    mapping_index,
                    mapping_id,
                ));
            }
            collect_script_hook_diagnostics(
                diagnostics,
                mapping_type,
                mapping_index,
                mapping_id,
                &mapping.script_hooks,
            );
        }
        MappingType::DirectionPad(mapping) => collect_script_hook_diagnostics(
            diagnostics,
            mapping_type,
            mapping_index,
            mapping_id,
            &mapping.script_hooks,
        ),
        MappingType::MouseCastSpell(mapping) => collect_script_hook_diagnostics(
            diagnostics,
            mapping_type,
            mapping_index,
            mapping_id,
            &mapping.script_hooks,
        ),
        MappingType::PadCastSpell(mapping) => collect_script_hook_diagnostics(
            diagnostics,
            mapping_type,
            mapping_index,
            mapping_id,
            &mapping.script_hooks,
        ),
        MappingType::CancelCast(mapping) => collect_script_hook_diagnostics(
            diagnostics,
            mapping_type,
            mapping_index,
            mapping_id,
            &mapping.script_hooks,
        ),
        MappingType::Observation(mapping) => collect_script_hook_diagnostics(
            diagnostics,
            mapping_type,
            mapping_index,
            mapping_id,
            &mapping.script_hooks,
        ),
        MappingType::Fps(mapping) => {
            if mapping.position.x <= FPS_MARGIN as i32 || mapping.position.y <= FPS_MARGIN as i32 {
                diagnostics.push(MappingDiagnostic::mapping(
                    "mapping.fps.invalidPosition",
                    t!(
                        "mask.mapping.invalidPosition",
                        x => mapping.position.x,
                        y => mapping.position.y,
                        margin => FPS_MARGIN
                    )
                    .to_string(),
                    mapping_type,
                    mapping_index,
                    mapping_id,
                ));
            }
            if mapping.max_offset_x < 0.0 || mapping.max_offset_y < 0.0 {
                diagnostics.push(MappingDiagnostic::mapping(
                    "mapping.fps.invalidMaxOffset",
                    "FPS max_offset_x/max_offset_y must be 0 or greater".to_string(),
                    mapping_type,
                    mapping_index,
                    mapping_id,
                ));
            }
        }
        MappingType::Fire(mapping) => {
            if mapping.preserve_fps_control && fps_touch_pointer_ids.contains(&mapping.pointer_id) {
                diagnostics.push(MappingDiagnostic::mapping(
                    "mapping.fire.pointerConflictsWithFps",
                    "Fire preserve_fps_control requires a pointer_id different from FPS touch pointer ids.",
                    mapping_type,
                    mapping_index,
                    mapping_id,
                ));
            }
            collect_script_hook_diagnostics(
                diagnostics,
                mapping_type,
                mapping_index,
                mapping_id,
                &mapping.script_hooks,
            );
        }
        MappingType::RawInput(_) => {}
        MappingType::Script(mapping) => {
            collect_script_field_diagnostics(
                diagnostics,
                mapping_type,
                mapping_index,
                mapping_id,
                "pressed_script",
                &mapping.pressed_script,
            );
            collect_script_field_diagnostics(
                diagnostics,
                mapping_type,
                mapping_index,
                mapping_id,
                "held_script",
                &mapping.held_script,
            );
            collect_script_field_diagnostics(
                diagnostics,
                mapping_type,
                mapping_index,
                mapping_id,
                "released_script",
                &mapping.released_script,
            );
        }
    }
}

fn collect_script_hook_diagnostics(
    diagnostics: &mut Vec<MappingDiagnostic>,
    mapping_type: &str,
    mapping_index: usize,
    mapping_id: &str,
    hooks: &MappingScriptHooks,
) {
    collect_script_field_diagnostics(
        diagnostics,
        mapping_type,
        mapping_index,
        mapping_id,
        "script_hooks.before_script",
        &hooks.before_script,
    );
    collect_script_field_diagnostics(
        diagnostics,
        mapping_type,
        mapping_index,
        mapping_id,
        "script_hooks.after_script",
        &hooks.after_script,
    );
}

fn collect_script_field_diagnostics(
    diagnostics: &mut Vec<MappingDiagnostic>,
    mapping_type: &str,
    mapping_index: usize,
    mapping_id: &str,
    field: &str,
    script: &str,
) {
    for diagnostic in ScriptAST::validate_diagnostics(script) {
        diagnostics.push(MappingDiagnostic::script(
            mapping_type,
            mapping_index,
            mapping_id,
            field,
            diagnostic,
        ));
    }
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
