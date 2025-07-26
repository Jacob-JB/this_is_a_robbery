use std::sync::Arc;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{
    Agent3d, AgentDesiredVelocity3d, AgentOptions, AgentSettings, AgentTarget3d, Archipelago3d,
    ArchipelagoRef3d, FromAgentRadius, Island, Landmass3dPlugin, NavMesh3d, NavMeshHandle,
    Velocity3d, nav_mesh::bevy_mesh_to_landmass_nav_mesh,
};

use common::agents::Agent;

const AGENT_RADIUS: f32 = 0.25;
const AGENT_DESIRED_SPEED: f32 = 2.0;
const AGENT_MAX_SPEED: f32 = 3.0;
const AGENT_MAX_ACCELERATION: f32 = 50.0;

pub fn build(app: &mut App) {
    app.add_plugins(Landmass3dPlugin::default());

    app.add_systems(Startup, spawn_archipelago);
    app.add_systems(
        Update,
        (
            load_nav_meshes,
            convert_nav_meshes,
            insert_agent_nav,
            (update_agent_velocities, integrate_agents).chain(),
        ),
    );
}

#[derive(Component)]
pub struct NavMeshPath(pub String);

#[derive(Component)]
struct ConvertNavMesh {
    mesh: Handle<Mesh>,
    nav_mesh: Handle<NavMesh3d>,
}

#[derive(Component)]
struct MainArchipelago;

fn spawn_archipelago(mut commands: Commands) {
    commands.spawn((
        MainArchipelago,
        Archipelago3d::new(AgentOptions::from_agent_radius(AGENT_RADIUS)),
    ));
}

fn load_nav_meshes(
    mut commands: Commands,
    mesh_q: Query<(Entity, &NavMeshPath), Without<ConvertNavMesh>>,
    archipelago_q: Query<Entity, With<MainArchipelago>>,
    assets: Res<AssetServer>,
    nav_meshes: ResMut<Assets<NavMesh3d>>,
) -> Result {
    for (entity, NavMeshPath(path)) in mesh_q.iter() {
        let archipelago_entity = archipelago_q.single()?;

        let mesh = assets.load(path);
        let nav_mesh = nav_meshes.reserve_handle();

        commands.entity(entity).insert((
            ArchipelagoRef3d::new(archipelago_entity),
            Island,
            NavMeshHandle(nav_mesh.clone()),
            ConvertNavMesh { mesh, nav_mesh },
            Transform::default(),
        ));
    }

    Ok(())
}

fn convert_nav_meshes(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut nav_meshes: ResMut<Assets<NavMesh3d>>,
    mesh_q: Query<(Entity, &ConvertNavMesh)>,
) {
    for (entity, convert) in mesh_q.iter() {
        let Some(mesh) = meshes.get(&convert.mesh) else {
            continue;
        };

        let nav_mesh = bevy_mesh_to_landmass_nav_mesh(mesh).unwrap();

        match nav_mesh.validate() {
            Ok(valid_nav_mesh) => {
                nav_meshes.insert(
                    &convert.nav_mesh,
                    NavMesh3d {
                        nav_mesh: Arc::new(valid_nav_mesh),
                        type_index_to_node_type: default(),
                    },
                );
            }
            Err(err) => {
                error!("Failed to validate nav mesh on {}: {}", entity, err);
            }
        }

        commands.entity(entity).remove::<ConvertNavMesh>();
    }
}

fn insert_agent_nav(
    mut commands: Commands,
    agent_q: Query<Entity, (With<Agent>, Without<Agent3d>)>,
    archipelago_q: Query<Entity, With<MainArchipelago>>,
) -> Result {
    for agent_entity in agent_q.iter() {
        let archipelago_entity = archipelago_q.single()?;

        commands.entity(agent_entity).insert((
            Agent3d::default(),
            AgentSettings {
                radius: AGENT_RADIUS,
                desired_speed: AGENT_DESIRED_SPEED,
                max_speed: AGENT_MAX_SPEED,
            },
            ArchipelagoRef3d::new(archipelago_entity),
            AgentTarget3d::None,
        ));
    }

    Ok(())
}

fn update_agent_velocities(
    mut agent_q: Query<(
        &mut LinearVelocity,
        &mut Velocity3d,
        &AgentDesiredVelocity3d,
    )>,
    time: Res<Time>,
) {
    for (mut linear_velocity, mut velocity, target_velocity) in agent_q.iter_mut() {
        debug!("target velocity is {}", target_velocity.velocity());

        let difference = target_velocity.velocity() - **linear_velocity;

        **linear_velocity +=
            difference.clamp_length_max(AGENT_MAX_ACCELERATION * time.delta_secs());
        velocity.velocity = **linear_velocity;
    }
}

fn integrate_agents(mut agent_q: Query<(&mut Transform, &LinearVelocity)>, time: Res<Time>) {
    for (mut transform, linear_velocity) in agent_q.iter_mut() {
        transform.translation += **linear_velocity * time.delta_secs();
    }
}
