use avian3d::prelude::*;
use bevy::prelude::*;
use common::CommonPlugin;

use crate::networking::ClientConnection;

pub mod agents;
pub mod camera;
pub mod character;
pub mod elements;
pub mod input;
pub mod networking;
pub mod physics_replication;
pub mod server_entity_map;
pub mod state;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
        level: bevy::log::Level::DEBUG,
        filter: bevy::log::DEFAULT_FILTER.to_string()
            + ",bevy_render=info,bevy_app=info,offset_allocator=info,bevy_asset=info,gilrs=info,bevy_winit=info",
        ..default()
    }).set(AssetPlugin {
        file_path: "../../assets".into(),
        ..default()
    }));

    app.add_plugins(CommonPlugin);

    app.add_plugins(PhysicsDebugPlugin::default());

    networking::build(&mut app);
    state::build(&mut app);
    server_entity_map::build(&mut app);
    physics_replication::build(&mut app);
    input::build(&mut app);
    character::build(&mut app);
    camera::build(&mut app);
    elements::build(&mut app);
    agents::build(&mut app);

    app.add_systems(PostStartup, debug_connect_to_server);

    app.run();
}

fn debug_connect_to_server(
    mut commands: Commands,
    endpoint_q: Query<Entity, With<networking::ClientEndpoint>>,
) -> Result {
    let endpoint_entity = endpoint_q.single()?;

    let address = std::env::args()
        .nth(1)
        .expect("Expected server address as first argument")
        .parse()
        .expect("Invalid server address");

    commands.spawn((
        ClientConnection,
        nevy::ConnectionOf(endpoint_entity),
        nevy::QuicConnectionConfig {
            client_config: networking::create_connection_config(),
            address,
            server_name: "example.server".to_string(),
        },
    ));

    Ok(())
}
