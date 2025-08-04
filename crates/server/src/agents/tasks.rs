use bevy::{platform::collections::HashSet, prelude::*};
use rand::{Rng, rng};

pub fn build(app: &mut App) {
    app.add_systems(Update, assign_tasks);
}

/// Exists on an agent when they have been assigned to a task.
///
/// Use the lifecycle events of this component to trigger state
/// upadates for starting and stopping tasks.
#[derive(Component)]
#[relationship(relationship_target = AssignedAgents)]
pub struct AssignedTo(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = AssignedTo)]
pub struct AssignedAgents(Vec<Entity>);

/// The priority of a task.
/// If the priority of an agents current task is lower than this task,
/// it will be pulled off that task and assigned to this one.
#[derive(Component, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[require(AssignedAgents)]
pub enum TaskPriority {
    #[default]
    Idle,
}

/// A list of which tasks this agent can be assigned to.
#[derive(Component, Default)]
pub struct AvailableTasks {
    pub tasks: HashSet<Entity>,
}

fn assign_tasks(
    mut commands: Commands,
    agent_q: Query<(Entity, &AvailableTasks, Option<&AssignedTo>)>,
    task_q: Query<&TaskPriority>,
) -> Result {
    for (agent_entity, available_tasks, assigned_to) in &agent_q {
        let current_priority = match assigned_to {
            Some(&AssignedTo(task_entity)) => {
                let &priority = task_q.get(task_entity)?;
                Some(priority)
            }
            None => None,
        };

        // Construct an ordered list of tasks that meet the prerequisites to be assigned to.
        let mut possible_tasks = Vec::new();

        for &task_entity in &available_tasks.tasks {
            let &priority = task_q.get(task_entity)?;

            if let Some(current_priority) = current_priority {
                if current_priority >= priority {
                    continue;
                }
            }

            // find the index of the first task with a lower priority, then insert there to keep order
            let mut index = 0;
            for &(_, possible_priority) in &possible_tasks {
                if possible_priority < priority {
                    break;
                }
                index += 1;
            }
            possible_tasks.insert(index, (task_entity, priority));
        }

        // Get the priority of the first task.
        // The list is ordered by priority so this is the highest priority.
        // If the list is empty no task can be assigned so continue to the next agent.
        let Some(&(_, highest_priority)) = possible_tasks.first() else {
            continue;
        };

        // Get the position of the first task with a lower priority.
        // If all tasks have the same priority this is the length of the list.
        // this value will be at least one because we just garunteed that the list isn't empty.
        let range = possible_tasks
            .iter()
            .position(|&(_, p)| p < highest_priority)
            .unwrap_or(possible_tasks.len());

        // Select a random task of the highest priority.
        let index = rng().random_range(0..range);
        let &(task_entity, _) = possible_tasks
            .get(index)
            .expect("Index of selected task should be in range");

        commands
            .entity(agent_entity)
            .insert(AssignedTo(task_entity));
    }

    Ok(())
}
