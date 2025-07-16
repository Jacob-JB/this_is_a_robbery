use bevy::{
    ecs::{
        component::{ComponentHook, HookContext, Immutable, StorageType},
        entity::EntityHashMap,
        world::DeferredWorld,
    },
    prelude::*,
};
use common::ServerEntity;

pub fn build(app: &mut App) {
    app.init_resource::<ServerEntityMap>();
}

/// A resource containing a map of [ServerEntity]s to [LocalServerEntity]s.
///
/// This map is updated using component hooks on [LocalServerEntity].
#[derive(Default, Resource)]
pub struct ServerEntityMap {
    map: EntityHashMap<Entity>,
}

/// Maps an entity to a [ServerEntity].
///
/// When inserted and removed will update the [ServerEntityMap] resource.
#[derive(Deref)]
pub struct LocalServerEntity(pub ServerEntity);

impl Component for LocalServerEntity {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    type Mutability = Immutable;

    fn on_insert() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, context: HookContext| {
            let client_entity = context.entity;

            let server_entity = **world
                .entity(client_entity)
                .get::<LocalServerEntity>()
                .expect("LocalServerEntity should exist");

            let mut map = world.resource_mut::<ServerEntityMap>();

            if let Some(replaced_entity) = map.map.insert(server_entity.into(), client_entity) {
                warn!(
                    "A duplicate `LocalServerEntity` {} replaced {} for {}. Removing `LocalServerEntity` from replaced entity.",
                    client_entity, replaced_entity, server_entity
                );

                world
                    .commands()
                    .entity(replaced_entity)
                    .remove::<LocalServerEntity>();
            }
        })
    }

    fn on_remove() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, context: HookContext| {
            let client_entity = context.entity;

            let server_entity = **world
                .entity(client_entity)
                .get::<LocalServerEntity>()
                .expect("LocalServerEntity should exist");

            let mut map = world.resource_mut::<ServerEntityMap>();

            let bevy::platform::collections::hash_map::Entry::Occupied(entry) =
                map.map.entry(Entity::from(server_entity))
            else {
                return;
            };

            // Only remove if the client entity wasn't replaced
            if *entry.get() == client_entity {
                entry.remove();
            }
        })
    }
}

impl ServerEntityMap {
    /// Gets a client entity from a server entity.
    pub fn get_client_entity(&self, server_entity: ServerEntity) -> Option<Entity> {
        self.map.get(&Entity::from(server_entity)).copied()
    }
}

#[cfg(test)]
mod tests {
    use crate::server_entity_map::{LocalServerEntity, ServerEntityMap};
    use bevy::prelude::*;
    use common::ServerEntity;

    #[test]
    fn server_entity_map() {
        let mut server_world = World::new();
        let mut world = World::new();

        world.init_resource::<ServerEntityMap>();

        let server_entity = ServerEntity::from(server_world.spawn_empty().id());

        // insert once
        let first_local_entity = world.spawn(LocalServerEntity(server_entity)).id();
        assert_eq!(
            world
                .resource::<ServerEntityMap>()
                .get_client_entity(server_entity),
            Some(first_local_entity)
        );

        // insert new entity, despawn old.
        let second_local_entity = world.spawn(LocalServerEntity(server_entity)).id();
        assert_eq!(
            world
                .resource::<ServerEntityMap>()
                .get_client_entity(server_entity),
            Some(second_local_entity)
        );

        let None = world.entity(first_local_entity).get::<LocalServerEntity>() else {
            panic!("Component should have been removed from the first local entity");
        };

        // remove local entity
        world.despawn(second_local_entity);
        assert_eq!(
            world
                .resource::<ServerEntityMap>()
                .get_client_entity(server_entity),
            None
        );
    }
}
