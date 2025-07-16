use bevy::{prelude::*, render::mesh::MeshPlugin, scene::ScenePlugin};
use common::CommonPlugin;

pub mod character;
pub mod networking;
pub mod physics_replication;
pub mod state;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        bevy::log::LogPlugin {
            level: bevy::log::Level::DEBUG,
            filter: bevy::log::DEFAULT_FILTER.to_string()
                + ",bevy_render=info,bevy_app=info,offset_allocator=info,bevy_asset=info,gilrs=info,bevy_winit=info",
            ..default()
        },
        AssetPlugin::default(),
        MeshPlugin,
        ScenePlugin,
    ));

    app.add_plugins(CommonPlugin);

    networking::build(&mut app);
    state::build(&mut app);
    physics_replication::build(&mut app);
    character::build(&mut app);

    app.run();
}
