use avian3d::prelude::*;
use bevy::{prelude::*, render::mesh::Indices};
use nevy::AddMessage;
use serde::{Deserialize, Serialize};

use crate::ServerEntity;

pub fn build(app: &mut App) {
    app.add_systems(Update, load_mesh_colliders);

    app.add_message::<InitializeGltfCollider>();
}

/// Server -> Client message to initialize a gltf collider.
#[derive(Serialize, Deserialize)]
pub struct InitializeGltfCollider {
    pub entity: ServerEntity,
    pub path: String,
}

/// Will insert a [Collider] of the first mesh in a gltf asset.
///
/// Will log an error and remove this component if there is an error,
/// no collider will be inserted.
///
/// Will use the first primitive of the gltf mesh.
#[derive(Component)]
#[require(RigidBody::Static)]
pub struct GltfCollider(pub Handle<Mesh>);

fn load_mesh_colliders(
    mut commands: Commands,
    collider_q: Query<(Entity, &GltfCollider), Without<Collider>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
) {
    for (collider_entity, GltfCollider(mesh_handle)) in collider_q.iter() {
        let Some(mesh) = mesh_assets.get_mut(mesh_handle) else {
            continue;
        };

        // dbg!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        // dbg!(mesh.indices().is_some());

        // `trimesh_from_mesh` expects indicies
        if let None = mesh.indices() {
            let Some(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
                error!(
                    "Mesh collider for {} has no position attribute",
                    collider_entity
                );
                commands.entity(collider_entity).remove::<GltfCollider>();
                continue;
            };

            mesh.insert_indices(Indices::U32(
                (0..positions.len()).map(|i| i as u32).collect(),
            ));
        }

        let Some(collider_component) = Collider::trimesh_from_mesh(mesh) else {
            error!(
                "Couldn't convert gltf mesh for {} into a collider",
                collider_entity
            );
            commands.entity(collider_entity).remove::<GltfCollider>();
            continue;
        };

        commands.entity(collider_entity).insert(collider_component);
    }
}
