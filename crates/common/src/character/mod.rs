use avian3d::prelude::*;
use bevy::prelude::*;
use nevy::AddMessage;
use serde::{Deserialize, Serialize};

use crate::{GameLayer, ServerEntity, character::controller::CharacterInput};

pub mod controller;

pub fn build(app: &mut App) {
    controller::build(app);

    app.add_message::<SpawnCharacter>();
    app.add_message::<SetLocalPlayer>();
    app.add_message::<CharacterStateUpdate>();
}

#[derive(Component, Default)]
#[require(
    Collider::capsule_endpoints(0.25, Vec3::Y * 0.25, Vec3::Y * 1.75),
    CollisionLayers::new(GameLayer::Players, GameLayer::World),
)]
pub struct Character;

/// Server -> Client message to initialize a new character.
#[derive(Serialize, Deserialize)]
pub struct SpawnCharacter {
    pub server_entity: ServerEntity,
}

/// Server -> Client message to set an existing character as the local player.
///
/// Ordering with [SpawnCharacter] is important.
#[derive(Serialize, Deserialize)]
pub struct SetLocalPlayer {
    pub server_entity: ServerEntity,
}

/// Client -> Server message to update a character's transform and input state.
#[derive(Serialize, Deserialize, Default)]
pub struct CharacterStateUpdate {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Quat,
    pub input: CharacterInput,
}
