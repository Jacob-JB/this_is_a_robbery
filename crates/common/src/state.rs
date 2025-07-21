use bevy::prelude::*;
use nevy::AddMessage;
use serde::{Deserialize, Serialize};

pub fn build(app: &mut App) {
    app.add_message::<JoinGameRequest>();
}

#[derive(Serialize, Deserialize)]
pub struct JoinGameRequest {
    pub username: String,
}
