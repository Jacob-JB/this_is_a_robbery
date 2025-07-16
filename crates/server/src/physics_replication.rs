use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;
use nevy::*;

use common::{networking::StreamHeader, physics::*};

use crate::state::JoinedClient;

pub const SNAPSHOT_INTERVAL: Duration = Duration::from_millis(150);
pub const TIME_SAMPLE_INTERVAL: Duration = Duration::from_millis(100);

pub fn build(app: &mut App) {
    app.add_systems(Update, (send_time_samples, send_physics_snapshots));
}

/// Marker component for which physics bodies to replicate.
#[derive(Component)]
pub struct ReplicateBody;

fn send_time_samples(
    client_q: Query<Entity, With<JoinedClient>>,
    mut messages: LocalMessageSender,
    message_id: Res<MessageId<TimeSample>>,
    time: Res<Time>,
    mut last_sample: Local<Duration>,
) -> Result {
    if time.elapsed() > *last_sample + TIME_SAMPLE_INTERVAL {
        *last_sample = time.elapsed();

        for client_entity in client_q.iter() {
            // if out of bandwidth don't send
            messages.write(
                StreamHeader::Messages,
                client_entity,
                *message_id,
                false,
                &TimeSample {
                    time: time.elapsed(),
                },
            )?;
        }
    }

    Ok(())
}

fn send_physics_snapshots(
    body_q: Query<(Entity, &Position, &LinearVelocity, &Rotation), With<ReplicateBody>>,
    client_q: Query<Entity, With<JoinedClient>>,
    mut messages: LocalMessageSender,
    message_id: Res<MessageId<PhysicsSnapshot>>,
    time: Res<Time>,
    mut last_snapshot: Local<Duration>,
) -> Result {
    if time.elapsed() > *last_snapshot + SNAPSHOT_INTERVAL {
        *last_snapshot = time.elapsed();

        let snapshot = PhysicsSnapshot {
            bodies: body_q
                .iter()
                .map(
                    |(
                        body_entity,
                        &Position(position),
                        &LinearVelocity(linear_velocity),
                        &Rotation(rotation),
                    )| {
                        (
                            body_entity.into(),
                            PhysicsBodySnapshot {
                                position,
                                linear_velocity,
                                rotation,
                            },
                        )
                    },
                )
                .collect(),
            time: time.elapsed(),
        };

        for client_entity in client_q.iter() {
            // if out of bandwidth don't send
            messages.write(
                StreamHeader::Messages,
                client_entity,
                *message_id,
                false,
                &snapshot,
            )?;
        }
    }

    Ok(())
}
