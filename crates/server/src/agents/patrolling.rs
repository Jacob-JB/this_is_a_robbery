use std::time::Duration;

use bevy::prelude::*;
use bevy_landmass::{AgentState, AgentTarget, AgentTarget3d};
use rand::{Rng, rng};

use crate::agents::tasks::{AssignedAgents, AssignedTo};

const POINT_WAIT_DURATION: Duration = Duration::from_secs(3);

pub fn build(app: &mut App) {
    app.add_observer(start_patroling);

    app.add_systems(
        Update,
        (
            assign_patrol_points,
            reach_patrol_points,
            leave_patrol_points,
        ),
    );
}

#[derive(Component)]
#[require(AssignedAgents)]
pub struct PatrolTask {
    pub points: Vec<Entity>,
}

#[derive(Component, Default)]
#[require(Transform)]
pub struct PatrolPoint {
    assigned_agent: Option<Entity>,
}

#[derive(Component, Default)]
enum AgentPatrolState {
    #[default]
    Unassigned,
    MovingTo {
        point_entity: Entity,
    },
    Arrived {
        point_entity: Entity,
        leave_at: Duration,
    },
}

fn start_patroling(
    trigger: Trigger<OnInsert, AssignedTo>,
    mut commands: Commands,
    agent_q: Query<&AssignedTo>,
    task_q: Query<(), With<PatrolTask>>,
) -> Result {
    let agent_entity = trigger.target();
    let &AssignedTo(task_entity) = agent_q.get(agent_entity)?;

    let Ok(()) = task_q.get(task_entity) else {
        return Ok(());
    };

    debug!("Agent {} started patroling", agent_entity);

    commands
        .entity(agent_entity)
        .insert(AgentPatrolState::default());

    Ok(())
}

// fn debug_set_agent_target(
//     mut agent_q: Query<&mut AgentTarget3d>,
//     character_q: Query<&Transform, With<Character>>,
// ) {
//     let Ok(character_transform) = character_q.single() else {
//         return;
//     };

//     for mut agent_target in agent_q.iter_mut() {
//         *agent_target = AgentTarget3d::Point(character_transform.translation);
//     }
// }

fn assign_patrol_points(
    mut agent_q: Query<(
        Entity,
        &AssignedTo,
        &mut AgentPatrolState,
        &mut AgentTarget3d,
    )>,
    task_q: Query<&PatrolTask>,
    mut point_q: Query<&mut PatrolPoint>,
) -> Result {
    for (agent_entity, &AssignedTo(task_entity), mut patrol_state, mut agent_target) in &mut agent_q
    {
        let AgentPatrolState::Unassigned = *patrol_state else {
            continue;
        };

        let task = task_q.get(task_entity)?;

        let available_points: Vec<Entity> = task
            .points
            .iter()
            .filter_map(|&point_entity| {
                point_q
                    .get(point_entity)
                    .ok()
                    .and_then(|point| point.assigned_agent.is_none().then_some(point_entity))
            })
            .collect();

        if available_points.is_empty() {
            warn!(
                "Unable to assign agent {} to a point in patrol task {}, out of points",
                agent_entity, task_entity
            );

            continue;
        }

        let index = rng().random_range(0..available_points.len());
        let &point_entity = available_points
            .get(index)
            .expect("Sampled index should be in range");

        let mut point = point_q.get_mut(point_entity)?;

        debug_assert!(
            point.assigned_agent.is_none(),
            "Tried to assign agent {} to point {} which already has an agent",
            agent_entity,
            point_entity,
        );

        *patrol_state = AgentPatrolState::MovingTo { point_entity };
        point.assigned_agent = Some(agent_entity);

        *agent_target = AgentTarget::Entity(point_entity);
    }

    Ok(())
}

fn reach_patrol_points(
    mut agent_q: Query<(Entity, &mut AgentPatrolState, &AgentState)>,
    time: Res<Time>,
) {
    for (agent_entity, mut agent_state, nav_state) in &mut agent_q {
        let AgentPatrolState::MovingTo { point_entity } = *agent_state else {
            continue;
        };

        match nav_state {
            AgentState::Idle | AgentState::Moving => (),
            AgentState::AgentNotOnNavMesh => warn!("Agent {} is not on the nav mesh", agent_entity),
            AgentState::TargetNotOnNavMesh => warn!(
                "Agent {}'s patrol point {} is not on the  mesh",
                agent_entity, point_entity
            ),
            AgentState::NoPath => warn!(
                "Agent {} couldn't find a path to it's patrol point {}",
                agent_entity, point_entity
            ),
            AgentState::ReachedTarget => {
                *agent_state = AgentPatrolState::Arrived {
                    point_entity,
                    leave_at: time.elapsed() + POINT_WAIT_DURATION,
                };
            }
        }
    }
}

fn leave_patrol_points(
    mut agent_q: Query<(&mut AgentPatrolState, &mut AgentTarget3d)>,
    mut point_q: Query<&mut PatrolPoint>,
    time: Res<Time>,
) -> Result {
    for (mut agent_state, mut agent_target) in &mut agent_q {
        let AgentPatrolState::Arrived {
            point_entity,
            leave_at,
        } = *agent_state
        else {
            continue;
        };

        if time.elapsed() < leave_at {
            continue;
        }

        point_q.get_mut(point_entity)?.assigned_agent = None;
        *agent_state = AgentPatrolState::Unassigned;
        *agent_target = AgentTarget::None;
    }

    Ok(())
}
