use bevy::prelude::*;
use common::{
    character::{SetLocalPlayer, SpawnCharacter, controller::Character},
    networking::StreamHeader,
};
use nevy::*;

use crate::state::{JoinedClient, initialize_pairs::InitializePairs};

pub fn build(app: &mut App) {
    app.add_systems(Update, (spawn_characters, initliaze_characters).chain());
}

#[derive(Component, Deref)]
#[relationship(relationship_target = ClientOfCharacter)]
pub struct CharacterOfClient(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = CharacterOfClient)]
pub struct ClientOfCharacter(Entity);

fn spawn_characters(mut commands: Commands, new_clients: Query<Entity, Added<JoinedClient>>) {
    for client_entity in new_clients.iter() {
        commands.spawn((Character, CharacterOfClient(client_entity)));
    }
}

fn initliaze_characters(
    pairs: InitializePairs<Character>,
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
