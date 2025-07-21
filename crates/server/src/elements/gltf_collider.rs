use bevy::prelude::*;
use common::{
    elements::gltf_collider::{GltfCollider, InitializeGltfCollider},
    networking::StreamHeader,
};
use nevy::*;

use crate::{replicate_despawn::ReplicateDespawn, state::initialize_pairs::InitializePairs};

pub fn build(app: &mut App) {
    app.add_systems(Update, (insert_gltf_colliders, replicate_gltf_colliders));
}

/// Used to load a gltf mesh on the server.
///
/// This is needed so that we can tell clients which path to load.
#[derive(Component)]
#[require(ReplicateDespawn)]
pub struct GltfColliderPath(pub String);

fn insert_gltf_colliders(
    mut commands: Commands,
    collider_q: Query<(Entity, &GltfColliderPath), Without<GltfCollider>>,
    assets: Res<AssetServer>,
) {
    for (entity, path) in collider_q.iter() {
        let handle = assets.load(&path.0);

        commands.entity(entity).insert(GltfCollider(handle));
    }
}

fn replicate_gltf_colliders(
    pairs: InitializePairs<GltfColliderPath>,
    gltf_colliders: Query<&GltfColliderPath>,
    mut messages: LocalMessageSender,
    message_id: Res<MessageId<InitializeGltfCollider>>,
) -> Result {
    messages.flush()?;

    for (client_entity, collider_entity) in pairs.iter() {
        let path = gltf_colliders.get(collider_entity)?;

        messages.write(
            StreamHeader::Messages,
            client_entity,
            *message_id,
            true,
            &InitializeGltfCollider {
                entity: collider_entity.into(),
                path: path.0.clone(),
            },
        )?;
    }

    Ok(())
}
