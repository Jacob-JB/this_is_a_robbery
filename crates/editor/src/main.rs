use bevy::prelude::*;

pub mod level;
pub mod ui;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: "../../assets".into(),
                ..default()
            })
            .set(bevy::log::LogPlugin {
                level: bevy::log::Level::DEBUG,
                filter:
                    "wgpu=error,naga=warn,bevy_xpbd_3d::plugins::setup=info,bevy=info,gilrs=info"
                        .into(),
                ..default()
            }),
    );

    ui::build(&mut app);

    app.run();
}
