use bevy::prelude::*;
use bevy_landmass::AgentTarget3d;
use common::{
    agents::{Agent, InitializeAgent},
    character::Character,
    networking::StreamHeader,
};
use nevy::*;

use crate::{physics_replication::ReplicateBody, state::initialize_pairs::InitializePairs};

pub mod nav_mesh;

pub fn build(app: &mut App) {
    nav_mesh::build(app);

    app.add_systems(Update, init_agents);
    app.add_systems(PostUpdate, initialize_agents.before(UpdateEndpoints));

    app.add_systems(Startup, debug_spawn_agents);
    app.add_systems(Update, debug_set_agent_target);
}

fn init_agents(mut commands: Commands, agent_q: Query<Entity, Added<Agent>>) {
    for agent_entity in agent_q.iter() {
        commands.entity(agent_entity).insert(ReplicateBody);
    }
}

fn initialize_agents(
    pairs: InitializePairs<Agent>,
    mut messages: LocalMessageSender,
    message_id: Res<MessageId<InitializeAgent>>,
) -> Result {
    messages.flush()?;

    for (client_entity, agent_entity) in pairs.iter() {
        messages.write(
            StreamHeader::Messages,
            client_entity,
            *message_id,
            true,
            &InitializeAgent {
                entity: agent_entity.into(),
            },
        )?;
    }

    Ok(())
}

fn debug_spawn_agents(mut commands: Commands) {
    commands.spawn(Agent);
}

fn debug_set_agent_target(
    mut agent_q: Query<&mut AgentTarget3d>,
    character_q: Query<&Transform, With<Character>>,
) {
    let Ok(character_transform) = character_q.single() else {
        return;
    };

    for mut agent_target in agent_q.iter_mut() {
        *agent_target = AgentTarget3d::Point(character_transform.translation);
    }
}
