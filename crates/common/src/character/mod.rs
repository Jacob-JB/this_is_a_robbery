use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ServerEntity;

pub mod controller;

pub fn build(app: &mut App) {
    controller::build(app);
}

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
