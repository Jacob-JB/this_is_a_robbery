use bevy::prelude::*;
use common::elements::gltf_collider::{GltfCollider, InitializeGltfCollider};

use crate::{networking::params::ClientMessages, server_entity_map::LocalServerEntity};

pub fn build(app: &mut App) {
    app.add_systems(Update, initialize_gltf_colliders);
}

fn initialize_gltf_colliders(
    mut commands: Commands,
    mut messages: ClientMessages<InitializeGltfCollider>,
    assets: Res<AssetServer>,
) {
    for InitializeGltfCollider { entity, path } in messages.drain() {
        commands.spawn((LocalServerEntity(entity), GltfCollider(assets.load(path))));
    }
}
