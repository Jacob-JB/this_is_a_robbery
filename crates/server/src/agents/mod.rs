use bevy::{platform::collections::HashSet, prelude::*};
use common::{
    agents::{Agent, InitializeAgent},
    networking::StreamHeader,
};
use nevy::*;

use crate::{
    agents::{
        investigation::AgentInvestigationState,
        navigation::NavMeshPath,
        patrolling::{PatrolPoint, PatrolTask},
        sight::AgentEyes,
        tasks::{AvailableTasks, TaskPriority},
    },
    physics_replication::ReplicateBody,
    state::initialize_pairs::InitializePairs,
};

pub mod investigation;
pub mod navigation;
pub mod patrolling;
pub mod sight;
pub mod tasks;

pub fn build(app: &mut App) {
    navigation::build(app);
    sight::build(app);
    tasks::build(app);
    patrolling::build(app);
    investigation::build(app);

    app.add_systems(Update, init_agents);
    app.add_systems(PostUpdate, initialize_agents.before(UpdateEndpoints));

    app.add_systems(Startup, (debug_spawn_nav_mesh, debug_spawn_agents));
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

fn debug_spawn_nav_mesh(mut commands: Commands) {
    commands.spawn(NavMeshPath("bank_nav_mesh.gltf#Mesh0/Primitive0".into()));
}

fn debug_spawn_agents(mut commands: Commands) {
    // -2 0 0
    // -1.5 0 -7.5
    // 3.5 0 -5

    let points = vec![
        commands
            .spawn((PatrolPoint::default(), Transform::from_xyz(-2., 0., 0.)))
            .id(),
        commands
            .spawn((PatrolPoint::default(), Transform::from_xyz(-1.5, 0., -7.5)))
            .id(),
        commands
            .spawn((PatrolPoint::default(), Transform::from_xyz(3.5, 0., -5.)))
            .id(),
    ];

    let task_entity = commands
        .spawn((PatrolTask { points }, TaskPriority::Idle))
        .id();

    commands.spawn((
        Agent,
        AvailableTasks {
            tasks: HashSet::from_iter([task_entity].into_iter()),
        },
        AgentEyes {
            offset: Vec3::Y * 1.8,
            fov: 45f32.to_radians(),
            range: f32::MAX,
        },
        AgentInvestigationState::default(),
    ));
}
