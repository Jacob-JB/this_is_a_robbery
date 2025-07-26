use avian3d::prelude::*;
use bevy::prelude::*;
use common::{
    character::{
        CharacterStateUpdate, SetLocalPlayer, SpawnCharacter,
        controller::{CharacterController, CharacterInput},
    },
    networking::StreamHeader,
};
use nevy::*;

use crate::{
    physics_replication::ReplicateBody,
    state::{JoinedClient, initialize_pairs::InitializePairs},
};

pub fn build(app: &mut App) {
    app.add_systems(Update, (receive_character_updates, spawn_characters));

    app.add_systems(PostUpdate, initliaze_characters.before(UpdateEndpoints));
}

#[derive(Component, Deref)]
#[relationship(relationship_target = ClientOfCharacter)]
pub struct CharacterOfClient(pub Entity);

#[derive(Component, Deref)]
#[relationship_target(relationship = CharacterOfClient)]
pub struct ClientOfCharacter(Entity);

fn spawn_characters(mut commands: Commands, new_clients: Query<Entity, Added<JoinedClient>>) {
    for client_entity in new_clients.iter() {
        commands.spawn((
            CharacterController,
            CharacterOfClient(client_entity),
            ReplicateBody,
        ));
    }
}

fn initliaze_characters(
    pairs: InitializePairs<CharacterController>,
    character_q: Query<&CharacterOfClient>,
    mut messages: LocalMessageSender,
    spawn_character: Res<MessageId<SpawnCharacter>>,
    set_local_player: Res<MessageId<SetLocalPlayer>>,
) -> Result {
    messages.flush()?;

    for (client_entity, character_entity) in pairs.iter() {
        messages.write(
            StreamHeader::Messages,
            client_entity,
            *spawn_character,
            true,
            &SpawnCharacter {
                server_entity: character_entity.into(),
            },
        )?;

        let character_of_client = character_q.get(character_entity)?;

        if **character_of_client == client_entity {
            messages.write(
                StreamHeader::Messages,
                client_entity,
                *set_local_player,
                true,
                &SetLocalPlayer {
                    server_entity: character_entity.into(),
                },
            )?;
        }
    }

    Ok(())
}

fn receive_character_updates(
    mut client_q: Query<(
        Entity,
        &mut ReceivedMessages<CharacterStateUpdate>,
        Option<&ClientOfCharacter>,
    )>,
    mut character_q: Query<(
        &mut Position,
        &mut LinearVelocity,
        &mut Rotation,
        &mut CharacterInput,
    )>,
) -> Result {
    for (client_entity, mut messages, character_of) in client_q.iter_mut() {
        for state_update in messages.drain() {
            let Some(character_of) = character_of else {
                warn!(
                    "Client {} sent a character state update when they don't have a character",
                    client_entity
                );
                continue;
            };

            let (mut position, mut velocity, mut rotation, mut input) =
                character_q.get_mut(**character_of)?;

            position.0 = state_update.position;
            velocity.0 = state_update.velocity;
            rotation.0 = state_update.rotation;
            *input = state_update.input;
        }
    }

    Ok(())
}
