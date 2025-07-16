use avian3d::{PhysicsPlugins, prelude::*};
use bevy::prelude::*;
use nevy::AddMessage;
use serde::{Deserialize, Serialize};

pub mod character;
pub mod networking;
pub mod physics;
pub mod state;

/// Plugin that is added to both the client and server
pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());

        networking::build(app);
        character::build(app);

        app.add_message::<state::JoinGameRequest>();
        app.add_message::<physics::PhysicsSnapshot>();
        app.add_message::<physics::TimeSample>();
        app.add_message::<character::SpawnCharacter>();
        app.add_message::<character::SetLocalPlayer>();
        app.add_message::<character::CharacterStateUpdate>();

        app.add_systems(Startup, spawn_debug_floor);
    }
}

/// newtype over an entity from the server's ecs world
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash)]
pub struct ServerEntity(Entity);

impl std::fmt::Display for ServerEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Server({})", Entity::from(*self))
    }
}

impl From<Entity> for ServerEntity {
    fn from(value: Entity) -> Self {
        ServerEntity(value)
    }
}

impl From<ServerEntity> for Entity {
    fn from(value: ServerEntity) -> Self {
        value.0
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

fn spawn_debug_floor(mut commands: Commands) {
    commands.spawn((
        Collider::half_space(Vec3::Y),
        RigidBody::Static,
        Position(Vec3::ZERO),
        CollisionLayers::new(GameLayer::World, 0),
    ));
}
