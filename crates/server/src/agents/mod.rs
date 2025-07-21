use std::sync::Arc;

use bevy::prelude::*;
use bevy_landmass::{
    AgentOptions, Archipelago3d, FromAgentRadius, Landmass3dPlugin, NavMesh3d,
    nav_mesh::bevy_mesh_to_landmass_nav_mesh,
};

pub fn build(app: &mut App) {
    app.add_plugins(Landmass3dPlugin::default());

    app.add_systems(Startup, spawn_archipelago);
    app.add_systems(Update, (load_nav_meshes, convert_nav_meshes));
}

#[derive(Component)]
pub struct NavMeshPath(pub String);

#[derive(Component)]
struct ConvertNavMesh {
    mesh: Handle<Mesh>,
    nav_mesh: Handle<NavMesh3d>,
}

fn spawn_archipelago(mut commands: Commands) {
    commands.spawn(Archipelago3d::new(AgentOptions::from_agent_radius(0.25)));
}

fn load_nav_meshes(
    mut commands: Commands,
    mesh_q: Query<(Entity, &NavMeshPath), Without<ConvertNavMesh>>,
    assets: Res<AssetServer>,
    nav_meshes: ResMut<Assets<NavMesh3d>>,
) {
    for (entity, NavMeshPath(path)) in mesh_q.iter() {
        let mesh = assets.load(path);
        let nav_mesh = nav_meshes.reserve_handle();

        commands
            .entity(entity)
            .insert(ConvertNavMesh { mesh, nav_mesh });
    }
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

// https://github.com/andriyDev/landmass/blob/main/crates/bevy_landmass/example/main.rs
