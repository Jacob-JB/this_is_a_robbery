use bevy::prelude::*;

pub mod gltf_collider;

pub fn build(app: &mut App) {
    gltf_collider::build(app);
}
