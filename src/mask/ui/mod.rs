pub mod basic;
pub mod mapping_label;

use basic::BasicPlugin;
use bevy::app::{App, Plugin};

use crate::mask::ui::mapping_label::MappingLabelPlugin;

pub struct UiPlugins;

impl Plugin for UiPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((BasicPlugin, MappingLabelPlugin));
    }
}
