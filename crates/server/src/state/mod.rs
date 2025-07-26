use bevy::prelude::*;
use common::state::JoinGameRequest;
use nevy::*;

pub mod initialize_pairs;

pub fn build(app: &mut App) {
    app.add_systems(PreUpdate, (accept_join_requests, remove_closed_clients));
}

/// Marker component for connection entities that have joined the game and should receive game updates.
#[derive(Component)]
pub struct JoinedClient;

fn accept_join_requests(
    mut commands: Commands,
    mut connection_q: Query<(Entity, &mut ReceivedMessages<JoinGameRequest>)>,
) {
    for (connection_entity, mut messages) in connection_q.iter_mut() {
        for JoinGameRequest { username } in messages.drain() {
            info!(
                "client {} joined game with username \"{}\"",
                connection_entity, username
            );

            commands.entity(connection_entity).insert(JoinedClient);
        }
    }
}

fn remove_closed_clients(mut commands: Commands, connection_q: Query<(Entity, &ConnectionStatus)>) {
    for (connection_entity, connection_status) in connection_q.iter() {
        let ConnectionStatus::Closed { .. } = connection_status else {
            continue;
        };

        commands.entity(connection_entity).despawn();

        debug!("removed closed client {}", connection_entity);
    }
}
