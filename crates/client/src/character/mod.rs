use avian3d::prelude::*;
use bevy::prelude::*;
use common::character::Character;

pub mod controls;

pub fn build(app: &mut App) {
    controls::build(app);

    app.add_systems(Startup, debug_spawn_player);
}

/// Marker component for the local player character
#[derive(Component)]
pub struct LocalPlayer;

fn debug_spawn_player(mut commands: Commands) {
    commands.spawn((LocalPlayer, Character, Position(Vec3::Y * 3.)));
}
