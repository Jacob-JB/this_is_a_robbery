use bevy::prelude::*;
use bevy_egui::egui::Ui;
use bevy_inspector_egui::bevy_inspector::ui_for_entity_with_children;

use crate::ui::element_selection::LastSelectedElement;

use super::Tab;

pub fn build(app: &mut App) {
    // add_openable_tab::<BevyInspector>(app, "Entity Inspector");
}

pub struct BevyInspector {
    query_state: QueryState<Entity, With<LastSelectedElement>>,
}

impl FromWorld for BevyInspector {
    fn from_world(world: &mut World) -> Self {
        BevyInspector {
            query_state: world.query_filtered(),
        }
    }
}

impl Tab for BevyInspector {
    fn title(&self) -> &'static str {
        "Bevy Inspector".into()
    }

    fn ui(&mut self, world: &mut World, ui: &mut Ui) {
        if let Ok(entity) = self.query_state.get_single(world) {
            ui_for_entity_with_children(world, entity, ui);
        }
    }
}
