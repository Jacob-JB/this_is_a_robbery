use bevy::{platform::collections::HashMap, prelude::*};

pub fn build(app: &mut App) {}

#[derive(Component)]
pub struct InvestigationTarget;

#[derive(Component, Default)]
pub struct AgentInvestigationState {
    /// A map of all investigation targets to investigation tasks
    targets: HashMap<Entity, Entity>,
}

#[derive(Component)]
pub struct InvestigationTask {}
