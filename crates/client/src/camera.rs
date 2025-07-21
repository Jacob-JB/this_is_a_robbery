use avian3d::prelude::*;
use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use common::character::controller::CharacterInput;

use crate::character::LocalPlayer;

const PLAYER_CAMERA_OFFSET: Vec3 = Vec3::new(0., 1.8, 0.);

pub fn build(app: &mut App) {
    app.add_systems(Startup, spawn_main_camera);
    app.add_systems(Update, (follow_player, toggle_cursor_lock));
}

#[derive(Component)]
pub struct MainCamera;

fn spawn_main_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 100f32.to_radians(),
            ..default()
        }),
    ));
}

fn follow_player(
    mut camera_q: Query<&mut Transform, With<MainCamera>>,
    player_q: Query<(&Position, &CharacterInput), With<LocalPlayer>>,
) -> Result {
    let Ok((player_position, input)) = player_q.single() else {
        return Ok(());
    };

    let mut camera_transform = camera_q.single_mut()?;

    *camera_transform = Transform::from_translation(player_position.0 + PLAYER_CAMERA_OFFSET)
        .looking_to(input.look_direction, Vec3::Y);

    Ok(())
}

fn toggle_cursor_lock(
    mut window_q: Query<&mut Window, With<PrimaryWindow>>,
    input: Res<ButtonInput<KeyCode>>,
    mut locked: Local<bool>,
) {
    let Ok(mut window) = window_q.single_mut() else {
        return;
    };

    if input.just_pressed(KeyCode::AltLeft) {
        *locked = !*locked;

        if *locked {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
        } else {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        }
    }
}
