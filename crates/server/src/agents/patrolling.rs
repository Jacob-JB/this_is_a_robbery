use bevy::prelude::*;

use crate::agents::tasks::AssignedAgents;

pub fn build(app: &mut App) {}

#[derive(Component)]
#[require(AssignedAgents)]
pub struct PatrolTask;

#[derive(Component)]
#[require(Transform)]
pub struct PatrolPoint {
    pub assigned_agent: Entity,
}
