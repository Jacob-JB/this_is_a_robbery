use bevy::{
    ecs::{
        query::QueryFilter,
        system::{EntityCommand, EntityCommands, SystemParam},
    },
    prelude::*,
};
use rand::{Rng, rng};

pub mod patrolling;

/// sets used to stop race conditions with shared resources when starting and stopping tasks
///
/// tasks should stop during [StopTasks](AgentTaskSet::StopTasks),
/// then tasks should start in [StartTasks](AgentTaskSet::StartTasks)
#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum AgentTaskSet {
    /// task stopping logic should run here
    StopTasks,
    /// task starting logic should run here
    StartTasks,
}

pub fn build(app: &mut App) {
    app.add_event::<StartedTask>();
    app.add_event::<StoppedTask>();

    app.configure_sets(
        Update,
        (AgentTaskSet::StopTasks, AgentTaskSet::StartTasks).chain(),
    );

    app.add_systems(PreUpdate, select_active_task);

    patrolling::build(app);
}

#[derive(Component, Default)]
pub struct AgentTaskState {
    /// a list of tasks registered as available to the agent
    assigned_tasks: Vec<Entity>,
    active_task: Option<Entity>,
}

#[derive(Component)]
pub struct AgentTask {
    /// the priority of the task
    priority: AgentTaskPriority,
    /// a list of agents that this task has been assigned to
    assigned_agents: Vec<Entity>,
    /// points to the active agent when it is active
    active_agent: Option<Entity>,
}

/// ordered task priorities
#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum AgentTaskPriority {
    /// the lowest priority for tasks that fall under normal behavior
    IdleTask,
    /// tasks that cause a guard to deviate from normal behavior to investigate something that might be of concern
    Investigate,
}

impl AgentTask {
    pub fn new(priority: AgentTaskPriority) -> Self {
        AgentTask {
            priority,
            assigned_agents: Vec::new(),
            active_agent: None,
        }
    }

    pub fn assigned_agents(&self) -> impl Iterator<Item = Entity> + '_ {
        self.assigned_agents.iter().copied()
    }
}

impl AgentTaskState {
    pub fn assigned_tasks(&self) -> impl Iterator<Item = Entity> + '_ {
        self.assigned_tasks.iter().copied()
    }
}

struct AssignToAgentCommand {
    agent: Entity,
}

impl EntityCommand for AssignToAgentCommand {
    fn apply(self, entity: EntityWorldMut) {
        let agent_entity = self.agent;
        let task_entity = entity.id();
        let world = entity.into_world_mut();

        let Ok([mut agent, mut task]) = world.get_entity_mut([agent_entity, task_entity]) else {
            error!(
                "could not assign task {:?} to agent {:?}, one or both of them doesn't exist",
                task_entity, agent_entity
            );
            return;
        };

        let Some(mut agent_task_state) = agent.get_mut::<AgentTaskState>() else {
            error!(
                "could not assign task {:?} to agent {:?}, {0:?} is not an agent",
                task_entity, agent_entity
            );
            return;
        };

        let Some(mut agent_task) = task.get_mut::<AgentTask>() else {
            error!(
                "could not assign task {:?} to agent {:?}, {1:?} is not a task",
                task_entity, agent_entity
            );
            return;
        };

        if !agent_task_state.assigned_tasks.contains(&task_entity) {
            agent_task_state.assigned_tasks.push(task_entity);
        }

        if !agent_task.assigned_agents.contains(&agent_entity) {
            agent_task.assigned_agents.push(agent_entity);
        }
    }
}

pub trait AgentTaskEntityCommands {
    /// assigns a task to an agent, making that agent able to complete the task
    fn assign_to_agent(&mut self, agent: Entity) -> &mut Self;
}

impl AgentTaskEntityCommands for EntityCommands<'_> {
    fn assign_to_agent(&mut self, agent: Entity) -> &mut Self {
        self.queue(AssignToAgentCommand { agent })
    }
}

/// fired when an agent is assigned a new task
#[derive(Event)]
struct StartedTask {
    agent: Entity,
    task: Entity,
}

/// fired when an agent stops doing a task because another is higher priority
///
/// note that if you stopped the task by despawning it, this event will not be fired
#[derive(Event)]
struct StoppedTask {
    agent: Entity,
    task: Entity,
}

/// responsible for changing the active task during [PreUpdate]
fn select_active_task(
    mut agent_q: Query<(Entity, &mut AgentTaskState)>,
    mut task_q: Query<&mut AgentTask>,

    mut started_task_w: EventWriter<StartedTask>,
    mut stopped_task_w: EventWriter<StoppedTask>,
) {
    for (agent_entity, mut agent_task_state) in agent_q.iter_mut() {
        // create a list of all the tasks of the highest priority
        // at the same time filter out tasks that have been despawned
        let mut highest_priority = AgentTaskPriority::IdleTask;
        let mut highest_priority_tasks = Vec::new();

        agent_task_state.assigned_tasks.retain(|&task_entity| {
            if let Ok(task) = task_q.get(task_entity) {
                if task.priority > highest_priority {
                    highest_priority = task.priority;
                    highest_priority_tasks = vec![task_entity];
                } else if task.priority == highest_priority {
                    highest_priority_tasks.push(task_entity);
                }

                true
            } else {
                false
            }
        });

        if let Some(active_task_entity) = agent_task_state.active_task {
            if let Ok(mut active_task) = task_q.get_mut(active_task_entity) {
                if active_task.priority >= highest_priority {
                    // already doing a task of highest priority
                    continue;
                }

                // unassign active task
                agent_task_state.active_task = None;
                active_task.active_agent = None;

                stopped_task_w.write(StoppedTask {
                    agent: agent_entity,
                    task: active_task_entity,
                });
            } else {
                // task no longer exists
                agent_task_state.active_task = None;
            }
        }

        // assign a new task

        if highest_priority_tasks.len() == 0 {
            continue;
        }

        let new_task_entity =
            highest_priority_tasks[rng().random_range(0..highest_priority_tasks.len())];

        let mut new_task = task_q
            .get_mut(new_task_entity)
            .expect("failed query gets should already be filtered out");

        agent_task_state.active_task = Some(new_task_entity);
        new_task.active_agent = Some(agent_entity);

        started_task_w.write(StartedTask {
            agent: agent_entity,
            task: new_task_entity,
        });
    }
}

#[derive(SystemParam)]
pub struct StartedTasks<'w, 's, F: QueryFilter + 'static> {
    started_task_r: EventReader<'w, 's, StartedTask>,
    query: Query<'w, 's, (), F>,
}

impl<'w, 's, F: QueryFilter + 'static> StartedTasks<'w, 's, F> {
    pub fn read(&mut self) -> impl Iterator<Item = (Entity, Entity)> + '_ {
        self.started_task_r
            .read()
            .filter_map(|&StartedTask { agent, task }| {
                if self.query.contains(task) {
                    Some((agent, task))
                } else {
                    None
                }
            })
    }
}

#[derive(SystemParam)]
pub struct StoppedTasks<'w, 's, F: QueryFilter + 'static> {
    stopped_task_r: EventReader<'w, 's, StoppedTask>,
    query: Query<'w, 's, (), F>,
}

impl<'w, 's, F: QueryFilter + 'static> StoppedTasks<'w, 's, F> {
    pub fn read(&mut self) -> impl Iterator<Item = (Entity, Entity)> + '_ {
        self.stopped_task_r
            .read()
            .filter_map(|&StoppedTask { agent, task }| {
                if self.query.contains(task) {
                    Some((agent, task))
                } else {
                    None
                }
            })
    }
}

#[derive(SystemParam)]
pub struct ActiveTasks<'w, 's, F: QueryFilter + 'static> {
    agent_q: Query<'w, 's, (Entity, &'static AgentTaskState)>,
    task_q: Query<'w, 's, (), F>,
}

impl<'w, 's, F: QueryFilter + 'static> ActiveTasks<'w, 's, F> {
    pub fn iter(&self) -> impl Iterator<Item = (Entity, Entity)> + '_ {
        self.agent_q
            .iter()
            .filter_map(|(agent_entity, task_state)| {
                if let Some(active_entity) = task_state.active_task {
                    if self.task_q.contains(active_entity) {
                        Some((agent_entity, active_entity))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
    }
}
