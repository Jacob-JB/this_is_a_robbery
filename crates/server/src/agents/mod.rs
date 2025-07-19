use bevy::prelude::*;
use bevy_landmass::{AgentOptions, Archipelago3d, FromAgentRadius, Landmass3dPlugin, NavMesh3d};

pub fn build(app: &mut App) {
    app.add_plugins(Landmass3dPlugin::default());

    app.add_systems(Startup, spawn_archipelago);
}

#[derive(Component)]
struct LoadNavMesh {
    mesh: Handle<Mesh>,
    nav_mesh: Handle<NavMesh3d>,
}

fn spawn_archipelago(mut commands: Commands) {
    commands.spawn(Archipelago3d::new(AgentOptions::from_agent_radius(0.25)));
}

fn load_mesh(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    nav_meshes: Res<Assets<NavMesh3d>>,
) {
    let mesh: Handle<Mesh> = asset_server.load("test_nav_mesh.gltf#Plane");
    let nav_mesh = nav_meshes.reserve_handle();

    commands.spawn(LoadNavMesh { mesh, nav_mesh });
}

fn convert_nav_meshes(mut meshe_q: Query<&LoadNavMesh>) {}

// https://github.com/andriyDev/landmass/blob/main/crates/bevy_landmass/example/main.rs
