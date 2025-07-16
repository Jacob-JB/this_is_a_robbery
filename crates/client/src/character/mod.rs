use bevy::prelude::*;
use common::character::{SetLocalPlayer, SpawnCharacter, controller::Character};

use crate::{
    networking::ClientMessages,
    physics_replication::SnapshotInterpolation,
    server_entity_map::{LocalServerEntity, ServerEntityMap},
};

pub mod controls;

pub fn build(app: &mut App) {
    controls::build(app);

    app.add_systems(Update, (initialize_players, set_local_player).chain());
}

/// Marker component for the local player character
#[derive(Component)]
pub struct LocalPlayer;

fn initialize_players(mut commands: Commands, mut messages: ClientMessages<SpawnCharacter>) {
    for SpawnCharacter { server_entity } in messages.drain() {
        debug!("Spawed character {}", server_entity);

        commands.spawn((
            LocalServerEntity(server_entity),
            SnapshotInterpolation::default(),
        ));
    }
}

fn set_local_player(
    mut commands: Commands,
    mut messages: ClientMessages<SetLocalPlayer>,
    map: Res<ServerEntityMap>,
) {
    for SetLocalPlayer { server_entity } in messages.drain() {
        let Some(character_entity) = map.get_client_entity(server_entity) else {
            error!("Expected local player character to exist");
            continue;
        };

        debug!("Set local player character to {}", character_entity);

        commands
            .entity(character_entity)
            .remove::<SnapshotInterpolation>()
            .insert((LocalPlayer, Character));
    }
}
