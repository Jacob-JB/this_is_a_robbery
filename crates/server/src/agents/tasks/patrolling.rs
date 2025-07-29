use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{AgentState, AgentTarget, AgentTarget3d};
use common::agents::PatrolPoint;
use rand::Rng;

use crate::agents::tasks::{AgentTaskSet, StartedTasks, StoppedTasks};

pub fn build(app: &mut App) {
    app.add_systems(
        Update,
        (
            start_patrol_task.in_set(AgentTaskSet::StartTasks),
            stop_patrol_task.in_set(AgentTaskSet::StopTasks),
            pick_new_point,
            finish_patrolling,
            reach_patrol_points,
        ),
    );
}

/// stored on the agent to tell the [PatrolTask] which points the agent uses
#[derive(Component)]
pub struct AgentPoints {
    pub points: Vec<Entity>,
}

#[derive(Component)]
pub struct PatrolTask;

#[derive(Component)]
pub struct PatrolPointState {
    pub assigned_agent: Option<Entity>,
}

/// starts the patrol task
fn start_patrol_task(mut commands: Commands, mut started: StartedTasks<With<PatrolTask>>) {
    for (agent_entity, _) in started.read() {
        // debug!("agent {:?} has started patrolling", agent_entity);

        commands.entity(agent_entity).insert(PatrolTaskState::Idle);
    }
}

/// stops the patrol task
fn stop_patrol_task(
    mut commands: Commands,
    mut stopped_tasks: StoppedTasks<With<PatrolTask>>,
    mut agent_q: Query<(&mut AgentTarget3d, &PatrolTaskState)>,
    mut patrol_point_q: Query<&mut PatrolPointState>,
) {
    for (agent_entity, _) in stopped_tasks.read() {
        let Ok((mut agent_target, patrol_state)) = agent_q.get_mut(agent_entity) else {
            error!("agent {:?} didn't have AgentPathfinding", agent_entity);
            continue;
        };

        // debug!("agent {:?} has stopped patrolling", agent_entity);

        if let &PatrolTaskState::WalkingPatrolPoint { point }
        | &PatrolTaskState::AtPatrolPoint { point, .. } = patrol_state
        {
            if let Ok(mut point) = patrol_point_q.get_mut(point) {
                point.assigned_agent = None;
            } else {
                warn!(
                    "tried to unassign point {:?} when agent {:?} stopped patrolling but couldn't find it",
                    point, agent_entity
                );
            }
        }

        *agent_target = AgentTarget::None;
        commands.entity(agent_entity).remove::<PatrolTaskState>();
    }
}

#[derive(Component)]
enum PatrolTaskState {
    Idle,
    WalkingPatrolPoint { point: Entity },
    AtPatrolPoint { point: Entity, move_time: Duration },
}

/// responsible for starting the agent towards it's next patrol point
fn pick_new_point(
    mut agent_q: Query<(
        Entity,
        &mut PatrolTaskState,
        &AgentPoints,
        &mut AgentTarget3d,
    )>,
    mut patrol_point_q: Query<&mut PatrolPointState>,
    position_q: Query<&GlobalTransform>,
) {
    for (agent_entity, mut agent_state, agent_points, mut agent_target) in agent_q.iter_mut() {
        let PatrolTaskState::Idle = *agent_state else {
            continue;
        };

        // get a list of eligible points that the agent can move to
        // points must not already be assigned to another agent and can't be the old point

        let eligible_points =
            Vec::from_iter(agent_points.points.iter().copied().filter(|&point_entity| {
                patrol_point_q
                    .get(point_entity)
                    .map(|point| point.assigned_agent.is_none())
                    .unwrap_or(false)
            }));

        if eligible_points.len() == 0 {
            warn!(
                "there were zero eligible points available for {:?} to patrol",
                agent_entity
            );
            continue;
        }

        let new_point_entity = eligible_points[rand::rng().random_range(0..eligible_points.len())];

        // unwrap has already been checked in the eligible points filter
        patrol_point_q
            .get_mut(new_point_entity)
            .unwrap()
            .assigned_agent = Some(agent_entity);

        let end_transform = position_q.get(new_point_entity).unwrap();

        *agent_target = AgentTarget::Point(end_transform.translation());

        *agent_state = PatrolTaskState::WalkingPatrolPoint {
            point: new_point_entity,
        };

        // debug!("agent {:?} is now moving to point {:?}", agent_entity, new_point_entity);
    }
}

/// creats the random delay for an agent moving
///
/// add to the current time to get when the agent should move again
pub fn create_agent_move_delay() -> Duration {
    // between 5 and 10 seconds

    Duration::from_secs_f32(rand::rng().random::<f32>() * 5. + 5.)
}

/// responsible for registering that an agent has reached a patrol point and starting the timer for it to move later
fn reach_patrol_points(
    mut agent_q: Query<(&AgentState, &mut PatrolTaskState, &mut Rotation), Without<PatrolPoint>>,
    point_q: Query<&Rotation, With<PatrolPoint>>,
    time: Res<Time>,
) {
    for (agent_state, mut task, mut rotation) in agent_q.iter_mut() {
        let PatrolTaskState::WalkingPatrolPoint { point } = *task else {
            continue;
        };

        if let AgentState::ReachedTarget = agent_state {
            *task = PatrolTaskState::AtPatrolPoint {
                point,
                move_time: time.elapsed() + create_agent_move_delay(),
            };

            if let Ok(&point_rotation) = point_q.get(point) {
                *rotation = point_rotation;
            }

            // debug!("an agent has reached point {:?}", point);
        }
    }
}

/// responsible for setting agents back to idle once they've been at a point long enough
fn finish_patrolling(
    mut agent_q: Query<&mut PatrolTaskState>,
    mut patrol_point_q: Query<&mut PatrolPointState>,
    time: Res<Time>,
) {
    for mut agent_state in agent_q.iter_mut() {
        let &PatrolTaskState::AtPatrolPoint {
            point: old_point_entity,
            move_time,
        } = agent_state.as_ref()
        else {
            continue;
        };

        if time.elapsed() < move_time {
            continue;
        }

        let Ok(mut old_point) = patrol_point_q.get_mut(old_point_entity) else {
            error!("tried to move an agent but they were assigned to a point that doesn't exist");
            continue;
        };

        // free the old point
        old_point.assigned_agent = None;

        *agent_state = PatrolTaskState::Idle;

        // debug!("an agent has finished patrolling point {:?}", old_point_entity);
    }
}
