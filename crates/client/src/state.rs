use bevy::prelude::*;
use common::{networking::StreamHeader, state::JoinGameRequest};
use nevy::*;

use crate::networking::ClientConnection;

pub fn build(app: &mut App) {
    app.init_state::<ClientState>();

    app.add_systems(
        Update,
        send_join_request.run_if(in_state(ClientState::Disconnected)),
    );
}

/// The state of the client.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientState {
    #[default]
    Disconnected,
    Joined,
}

fn send_join_request(
    connection_q: Query<
        (Entity, &ConnectionStatus),
        (Changed<ConnectionStatus>, With<ClientConnection>),
    >,
    mut sender: LocalMessageSender,
    message_id: Res<MessageId<JoinGameRequest>>,
) -> Result {
    sender.flush()?;
    sender.finish_all_if_uncongested()?;

    if let Ok((connection_entity, ConnectionStatus::Established)) = connection_q.single() {
        sender.write(
            StreamHeader::Messages,
            connection_entity,
            *message_id,
            true,
            &JoinGameRequest {
                username: "Player".into(),
            },
        )?;

        info!("Requested to join game");
    }

    Ok(())
}
