use bevy::prelude::*;

/// a marker component of all entities that are considered part of the level
///
/// these are the entities that get saved and can be copied and deleted
#[derive(Component)]
pub struct LevelElement;
