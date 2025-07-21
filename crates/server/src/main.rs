use bevy::{gltf::GltfPlugin, prelude::*, render::mesh::MeshPlugin, scene::ScenePlugin};
use common::CommonPlugin;

use crate::{config::ServerConfig, elements::gltf_collider::GltfColliderPath};

pub mod agents;
pub mod character;
pub mod config;
pub mod elements;
pub mod level;
pub mod networking;
pub mod physics_replication;
pub mod replicate_despawn;
pub mod state;

fn main() {
    let mut app = App::new();

    match ServerConfig::load() {
        Ok(config) => {
            app.insert_resource(config);
        }
        Err(err) => {
            println!("Failed to load server config: {}", err);
            return;
        }
    }

    app.add_plugins((
        MinimalPlugins,
        bevy::log::LogPlugin {
            level: bevy::log::Level::DEBUG,
            filter: bevy::log::DEFAULT_FILTER.to_string()
                + ",bevy_render=info,bevy_app=info,offset_allocator=info,bevy_asset=info,gilrs=info,bevy_winit=info",
            ..default()
        },
        AssetPlugin {
            file_path: "../../assets".into(),
            ..default()
        },
        MeshPlugin,
        ScenePlugin,
    ));

    app.init_asset::<Shader>();

    app.add_plugins((
        GltfPlugin::default(),
        MaterialPlugin::<StandardMaterial>::default(),
    ));

    app.add_plugins(CommonPlugin);

    networking::build(&mut app);
    state::build(&mut app);
    physics_replication::build(&mut app);
    character::build(&mut app);
    agents::build(&mut app);
    level::build(&mut app);
    elements::build(&mut app);
    replicate_despawn::build(&mut app);

    app.add_systems(Startup, debug_level_setup);

    app.run();
}

fn debug_level_setup(mut commands: Commands) {
    commands.spawn(GltfColliderPath(
        "bank_collider.gltf#Mesh0/Primitive0".into(),
    ));
}
