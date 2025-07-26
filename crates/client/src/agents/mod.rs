use bevy::prelude::*;
use common::agents::{Agent, InitializeAgent};

use crate::{
    networking::params::ClientMessages, physics_replication::SnapshotInterpolation,
    server_entity_map::LocalServerEntity,
};

pub fn build(app: &mut App) {
    app.add_systems(Update, initialize_agents);
}

fn initialize_agents(mut commands: Commands, mut messages: ClientMessages<InitializeAgent>) {
    for InitializeAgent { entity } in messages.drain() {
        commands.spawn((
            LocalServerEntity(entity),
            Agent,
            SnapshotInterpolation::default(),
        ));
    }
}
