use avian3d::prelude::*;
use bevy::prelude::*;
use nevy::*;
use serde::{Deserialize, Serialize};

use crate::{GameLayer, ServerEntity};

pub fn build(app: &mut App) {
    app.add_message::<InitializeAgent>();
}

#[derive(Component)]
#[require(
    Collider::capsule_endpoints(0.25, Vec3::Y * 0.25, Vec3::Y * 1.75),
    CollisionLayers::new([GameLayer::Agents, GameLayer::Opaque], 0),
    Transform,
    Position,
    Rotation,
    LinearVelocity,
)]
pub struct Agent;

#[derive(Component, Serialize, Deserialize)]
#[require(Position)]
pub struct PatrolPoint;

#[derive(Serialize, Deserialize)]
pub struct InitializeAgent {
    pub entity: ServerEntity,
}
