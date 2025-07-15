use avian3d::prelude::*;
use bevy::{input::mouse::MouseMotion, prelude::*};
use common::{GameLayer, character::CharacterInput};

use crate::{character::LocalPlayer, input::ControlScheme};

const MAX_VERTICAL_CAMERA_ANGLE: f32 = std::f32::consts::FRAC_PI_2 * 0.9;
const ON_GROUND_TOLERANCE: f32 = 0.02;
pub const PLAYER_JUMP_SPEED: f32 = 3.;

pub fn build(app: &mut App) {
    app.add_systems(Update, (get_movement_input, get_camera_input, jump_players));
}

fn get_movement_input(
    controls: Res<ControlScheme>,
    input: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<&mut CharacterInput, With<LocalPlayer>>,
) {
    let Ok(mut character_input) = player_q.single_mut() else {
        return;
    };

    character_input.move_forward = input.pressed(controls.move_forward);
    character_input.move_backward = input.pressed(controls.move_backward);
    character_input.move_left = input.pressed(controls.move_left);
    character_input.move_right = input.pressed(controls.move_right);
}

fn get_camera_input(
    mut mouse: EventReader<MouseMotion>,
    controls: Res<ControlScheme>,
    mut player_q: Query<&mut CharacterInput, With<LocalPlayer>>,
    mut rotation: Local<Vec2>,
) {
    let delta = mouse.read().map(|e| e.delta).sum::<Vec2>() * -controls.mouse_sensitivity;

    rotation.x += delta.x;
    rotation.y =
        (rotation.y + delta.y).clamp(-MAX_VERTICAL_CAMERA_ANGLE, MAX_VERTICAL_CAMERA_ANGLE);

    let Ok(mut player_input) = player_q.single_mut() else {
        return;
    };

    player_input.look_direction = Dir3::new(
        Quat::from_euler(EulerRot::YXZ, rotation.x, rotation.y, 0.).mul_vec3(Vec3::NEG_Z),
    )
    .unwrap();
}

fn jump_players(
    controls: Res<ControlScheme>,
    input: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<(&Position, &mut LinearVelocity), With<LocalPlayer>>,
    spatial_query: SpatialQuery,
) {
    let Ok((&Position(position), mut velocity)) = player_q.single_mut() else {
        return;
    };

    let on_ground = spatial_query
        .cast_ray(
            position,
            Dir3::NEG_Y,
            ON_GROUND_TOLERANCE,
            true,
            &SpatialQueryFilter::from_mask([GameLayer::World]),
        )
        .is_some();

    if on_ground && input.just_pressed(controls.jump) {
        velocity.0 += Vec3::Y * PLAYER_JUMP_SPEED;
    }
}
