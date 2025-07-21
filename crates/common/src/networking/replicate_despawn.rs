use bevy::prelude::*;
use nevy::AddMessage;
use serde::{Deserialize, Serialize};

use crate::ServerEntity;

pub fn build(app: &mut App) {
    app.add_message::<ServerEntityRemoved>();
}

/// Server -> Client message used by many parts of the game
/// when simple despawning logic is all that is needed.
#[derive(Serialize, Deserialize)]
pub struct ServerEntityRemoved {
    pub entity: ServerEntity,
}
