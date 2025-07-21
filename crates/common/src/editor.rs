//! scene metadata for the editor

use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::*,
};

/// points to the group an element is contained in
#[derive(Component, Reflect, MapEntities)]
#[reflect(Component, MapEntities)]
#[relationship(relationship_target = ElementGroup)]
pub struct ParentGroup(pub Entity);

/// used for grouping elements in the editor
#[derive(Component, Reflect, Default, MapEntities)]
#[reflect(Component, MapEntities)]
#[relationship_target(relationship = ParentGroup)]
pub struct ElementGroup(Vec<Entity>);

// impl MapEntities for ElementGroup {
//     fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
//         for element in self.elements.iter_mut() {
//             *element = entity_mapper.map_entity(*element);
//         }
//     }
// }

impl ElementGroup {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.0.iter().copied()
    }
}

// impl MapEntities for ParentGroup {
//     fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
//         self.group = entity_mapper.map_entity(self.group);
//     }
// }

// impl Default for ParentGroup {
//     fn default() -> Self {
//         ParentGroup {
//             // best we have for an "invalid" entity
//             group: Entity::from_bits(u64::MAX),
//         }
//     }
// }

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct SaveMeta {
    pub save_type: SaveType,
}

#[derive(Reflect, Default, Clone, Copy)]
pub enum SaveType {
    #[default]
    Level,
    Item,
}
