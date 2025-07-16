use avian3d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::character::Character;

const PLAYER_ACCELERATION: f32 = 75.;
const PLAYER_MOVE_SPEED: f32 = 5.;
const MAX_INTEGRATE_ITERATIONS: usize = 20;
const PLAYER_COLLISION_MARGIN: f32 = 0.0005;

pub fn build(app: &mut App) {
    app.add_systems(
        FixedPostUpdate,
        (
            rotate_players,
            accelerate_players,
            integrate_character,
            apply_integrated_positions,
        )
            .chain()
            .in_set(PhysicsSet::StepSimulation),
    );
}

#[derive(Component, Default)]
#[require(
    Character,
    CharacterInput,
    RigidBody::Kinematic,
    IntegratedCharacterPosition
)]
pub struct CharacterController;

/// The input state for a character
///
/// Used to simulate a character both on the client and for prediction on the server
#[derive(Clone, Copy, Component, Serialize, Deserialize)]
pub struct CharacterInput {
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub look_direction: Dir3,
}

impl Default for CharacterInput {
    fn default() -> Self {
        CharacterInput {
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            look_direction: Dir3::NEG_Z,
        }
    }
}

fn rotate_players(mut player_q: Query<(&CharacterInput, &mut Rotation)>) {
    for (input, mut rotation) in player_q.iter_mut() {
        let face_direction = Vec3 {
            y: 0.,
            ..input.look_direction.into()
        }
        .normalize();

        rotation.0 = Transform::default()
            .looking_to(face_direction, Vec3::Y)
            .rotation;
    }
}

fn accelerate_players(
    mut player_q: Query<(&CharacterInput, &mut LinearVelocity, &Rotation)>,
    gravity: Res<Gravity>,
    time: Res<Time>,
) {
    for (input, mut velocity, rotation) in player_q.iter_mut() {
        let target_velocity = Vec2 {
            x: match (input.move_left, input.move_right) {
                (true, false) => 1.,
                (false, true) => -1.,
                _ => 0.,
            },
            y: match (input.move_backward, input.move_forward) {
                (true, false) => 1.,
                (false, true) => -1.,
                _ => 0.,
            },
        }
        .normalize_or_zero();

        let target_velociy = Vec2::from_angle(-rotation.to_euler(EulerRot::YXZ).0)
            .rotate(target_velocity * PLAYER_MOVE_SPEED);

        let difference = target_velociy - velocity.0.xz();
        let max_acceleration = PLAYER_ACCELERATION * time.delta_secs();
        let delta = difference.clamp_length_max(max_acceleration);
        **velocity += Vec3::new(delta.x, 0., delta.y);

        **velocity += gravity.0 * time.delta_secs();
    }
}

/// Used by the kinematic integrator
#[derive(Component, Default)]
struct IntegratedCharacterPosition(Vec3);

/// Integrates kinematic character positions.
/// Performs collision detection and slides characters along obstacles.
fn integrate_character(
    mut character_q: Query<
        (
            Entity,
            &mut LinearVelocity,
            &Position,
            &mut IntegratedCharacterPosition,
            &Rotation,
            &Collider,
            &CollisionLayers,
        ),
        With<CharacterController>,
    >,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    for (
        player_entity,
        mut velocity,
        position,
        mut position_update,
        rotation,
        collider,
        collision_layers,
    ) in character_q.iter_mut()
    {
        let mut position = **position;
        let mut remaining_time = time.delta_secs();

        for _ in 0..MAX_INTEGRATE_ITERATIONS {
            let Ok(direction) = Dir3::new(**velocity) else {
                break;
            };

            let max_distance = remaining_time * velocity.length();

            let hit = spatial_query
                .shape_hits(
                    collider,
                    position,
                    **rotation,
                    direction,
                    u32::MAX,
                    &ShapeCastConfig {
                        max_distance,
                        ..default()
                    },
                    // &SpatialQueryFilter::from_mask([GameLayer::World])
                    &SpatialQueryFilter::from_mask(collision_layers.filters)
                        .with_excluded_entities(std::iter::once(player_entity)),
                )
                .into_iter()
                .filter(|hit| -hit.normal1.dot(direction.into()) > 0.)
                .next();

            let Some(hit) = hit else {
                position += direction * max_distance;
                break;
            };

            let hit_normal = rotation.mul_vec3(-hit.normal2);

            remaining_time -= hit.distance / velocity.length();

            position += direction * hit.distance;
            position += hit_normal * PLAYER_COLLISION_MARGIN;

            **velocity = velocity.reject_from(hit_normal);
        }

        position_update.0 = position;
    }
}

/// Updates character positions after [integrate_characters].
fn apply_integrated_positions(
    mut character_q: Query<(&mut Position, &IntegratedCharacterPosition)>,
) {
    for (mut position, integrated_position) in character_q.iter_mut() {
        **position = integrated_position.0;
    }
}
