use bevy::prelude::*;
use common::networking::{StreamHeader, replicate_despawn::ServerEntityRemoved};
use nevy::*;

use crate::state::JoinedClient;

pub fn build(app: &mut App) {
    app.add_systems(PostUpdate, send_despawned_messages.before(UpdateEndpoints));
}

/// When this component is removed from an entity (the entity is despawned)
/// a [ServerEntityRemoved] message will be sent to all [JoinedClient]s.
#[derive(Component, Default)]
pub struct ReplicateDespawn;

fn send_despawned_messages(
    mut components: RemovedComponents<ReplicateDespawn>,
    clients: Query<Entity, With<JoinedClient>>,
    mut messages: LocalMessageSender,
    message_id: Res<MessageId<ServerEntityRemoved>>,
) -> Result {
    messages.flush()?;

    for despawned_entity in components.read() {
        for client_entity in clients.iter() {
            messages.write(
                StreamHeader::Messages,
                client_entity,
                *message_id,
                true,
                &ServerEntityRemoved {
                    entity: despawned_entity.into(),
                },
            )?;
        }
    }

    Ok(())
}
