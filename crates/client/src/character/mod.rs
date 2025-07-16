use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;
use common::character::{
    Character, CharacterStateUpdate, SetLocalPlayer, SpawnCharacter,
    controller::{CharacterController, CharacterInput},
};
use nevy::*;

use crate::{
    networking::params::{ClientMessages, LocalClientMessageSender},
    physics_replication::SnapshotInterpolation,
    server_entity_map::{LocalServerEntity, ServerEntityMap},
};

pub mod controls;

const PLAYER_UPDATE_INTERVAL: Duration = Duration::from_millis(100);

pub fn build(app: &mut App) {
    controls::build(app);

    app.add_systems(
        Update,
        (
            send_character_updates,
            (initialize_players, set_local_player).chain(),
        ),
    );
}

/// Marker component for the local player character
#[derive(Component)]
pub struct LocalPlayer;

fn initialize_players(mut commands: Commands, mut messages: ClientMessages<SpawnCharacter>) {
    for SpawnCharacter { server_entity } in messages.drain() {
        debug!("Spawed character {}", server_entity);

        commands.spawn((
            LocalServerEntity(server_entity),
            Character,
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
            .insert((LocalPlayer, CharacterController));
    }
}

fn send_character_updates(
    character_q: Query<(&Position, &LinearVelocity, &Rotation, &CharacterInput), With<LocalPlayer>>,
    mut messages: LocalClientMessageSender,
    message_id: Res<MessageId<CharacterStateUpdate>>,
    time: Res<Time>,
    mut last_update: Local<Duration>,
) -> Result {
    if time.elapsed() - *last_update > PLAYER_UPDATE_INTERVAL {
        *last_update = time.elapsed();

        let Ok((&Position(position), &LinearVelocity(velocity), &Rotation(rotation), &input)) =
            character_q.single()
        else {
            return Ok(());
        };

        messages.write(
            *message_id,
            false,
            &CharacterStateUpdate {
                position,
                velocity,
                rotation,
                input,
            },
        )?;
    }

    Ok(())
}
