use avian3d::{PhysicsPlugins, prelude::PhysicsLayer};
use bevy::prelude::*;

pub mod character;
pub mod networking;

/// Plugin that is added to both the client and server
pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());

        networking::build(app);
        character::build(app);
    }
}

#[derive(Default, PhysicsLayer)]
pub enum GameLayer {
    /// Mostly static elements that players collide with
    #[default]
    World,
    /// Players
    Players,
    /// Used on client for colliders that can block
    /// or receive interaction from the player camera
    Interaction,
}
