use std::path::PathBuf;

use bevy::prelude::*;
use common::DebugStartLevel;
use nevy::ReceivedMessages;

pub fn build(app: &mut App) {
    app.add_event::<LoadGameLevel>();

    app.add_systems(Update, load_levels);

    app.add_systems(Update, debug_start_level);
}

#[derive(Event)]
pub struct LoadGameLevel {
    pub level_scene: String,
}

#[derive(Component)]
pub struct GameLevelRoot;

fn load_levels(
    mut commands: Commands,
    level_q: Query<(), With<GameLevelRoot>>,
    mut load_level_r: EventReader<LoadGameLevel>,
    asset_server: Res<AssetServer>,
) {
    let mut consecutive = false;

    for LoadGameLevel {
        level_scene: level_name,
    } in load_level_r.read()
    {
        if !level_q.is_empty() || consecutive {
            error!(
                "Attempted to load level \"{}\" while another level was already loaded",
                level_name
            );

            continue;
        }

        consecutive = true;

        let mut path = PathBuf::from("levels");
        path.push("level_name");

        info!("Loading level at \"{:?}\"", path);

        let scene = asset_server.load(path);

        commands.spawn((GameLevelRoot, DynamicSceneRoot(scene)));
    }
}

fn debug_start_level(
    mut client_q: Query<&mut ReceivedMessages<DebugStartLevel>>,
    mut load_level_w: EventWriter<LoadGameLevel>,
) {
    for mut messages in client_q.iter_mut() {
        for DebugStartLevel in messages.drain() {
            load_level_w.write(LoadGameLevel {
                level_scene: "bank.scn.ron".into(),
            });
        }
    }
}
