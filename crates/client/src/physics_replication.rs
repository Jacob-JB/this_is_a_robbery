use std::{collections::VecDeque, time::Duration};

use avian3d::prelude::*;
use bevy::prelude::*;
use common::physics::*;

use crate::{networking::ClientMessages, server_entity_map::ServerEntityMap};

pub fn build(app: &mut App) {
    app.init_resource::<PhysicsTimeEstimate>();
    app.init_resource::<PhysicsSnapshots>();
    app.init_resource::<InterpolateValues>();
    app.insert_resource(SnapshotPlayoutDelay {
        delay: Duration::from_millis(200),
    });

    app.add_systems(
        Update,
        (
            receive_time_samples,
            calculate_time_estimate,
            receive_physics_snapshots,
        ),
    );

    app.add_systems(
        PostUpdate,
        (clear_interpolation, queue_interpolation, interpolate_bodies).chain(),
    );
}

/// how many samples to keep to make the time estimate
const TIME_ESTIMATE_SAMPLES: usize = 30;

#[derive(Resource, Default)]
pub struct PhysicsTimeEstimate {
    /// list of samples and when they were received
    samples: VecDeque<(TimeSample, Duration)>,
    pub time_estimate: Duration,
}

fn receive_time_samples(
    mut messages: ClientMessages<TimeSample>,
    mut physics_time: ResMut<PhysicsTimeEstimate>,
    time: Res<Time>,
) {
    for sample in messages.drain() {
        physics_time.samples.push_back((sample, time.elapsed()));

        if physics_time.samples.len() > TIME_ESTIMATE_SAMPLES {
            physics_time.samples.pop_front();
        }
    }
}

fn calculate_time_estimate(mut physics_time: ResMut<PhysicsTimeEstimate>, time: Res<Time>) {
    if let Some(time_estimate) = physics_time
        .samples
        .iter()
        .map(|(sample, received_time)| sample.time + (time.elapsed() - *received_time))
        .sum::<Duration>()
        .checked_div(physics_time.samples.len() as u32)
    {
        physics_time.time_estimate = time_estimate;
    }
}

#[derive(Resource, Default)]
pub struct PhysicsSnapshots {
    /// ordered list of physics snapshots sent from the server
    pub snapshots: VecDeque<PhysicsSnapshot>,
}

impl PhysicsSnapshots {
    /// binary searches for the index of a specific time
    ///
    /// if there is an exact match returns that index, otherwise the index directly after
    pub fn search(&self, time: Duration) -> usize {
        let (Ok(index) | Err(index)) = self
            .snapshots
            .binary_search_by(|snapshot| snapshot.time.cmp(&time));

        index
    }
}

fn receive_physics_snapshots(
    mut messages: ClientMessages<PhysicsSnapshot>,
    mut snapshots: ResMut<PhysicsSnapshots>,
) {
    for snapshot in messages.drain() {
        let index = snapshots.search(snapshot.time);

        snapshots.snapshots.insert(index, snapshot);
    }
}

/// Insert onto a mapped server entity to
/// enable physics snapshot interpolation.
///
/// Will only apply interpolation if the entity
/// has a [Position] and [Rotation].
#[derive(Component, Default)]
pub struct SnapshotInterpolation {
    start: Option<PhysicsBodySnapshot>,
    end: Option<PhysicsBodySnapshot>,
}

#[derive(Resource)]
pub struct SnapshotPlayoutDelay {
    pub delay: Duration,
}

#[derive(Resource, Default)]
struct InterpolateValues {
    interpolate_percent: f32,
    velocity_scale: f32,
}

fn clear_interpolation(mut body_q: Query<&mut SnapshotInterpolation>) {
    for mut interpolation in body_q.iter_mut() {
        interpolation.start = None;
        interpolation.end = None;
    }
}

fn queue_interpolation(
    mut snapshots: ResMut<PhysicsSnapshots>,
    physics_time: Res<PhysicsTimeEstimate>,
    playout_delay: Res<SnapshotPlayoutDelay>,
    map: Res<ServerEntityMap>,
    mut interpolate_values: ResMut<InterpolateValues>,
    mut body_q: Query<&mut SnapshotInterpolation>,
) {
    // calculate playout time based on time estimate and delay
    let playout_time = physics_time
        .time_estimate
        .saturating_sub(playout_delay.delay);

    // get the indicies of the snapshots before and after the playout time
    let end_index = snapshots.search(playout_time);

    let Some(start_index) = end_index.checked_sub(1) else {
        // warn!("Have no snapshot to start interpolation at");
        return;
    };

    // get the snapshots
    let Some(end_snapshot) = snapshots.snapshots.get(end_index) else {
        // warn!("Have no snapshot to end interpolation at");
        return;
    };

    let start_snapshot = snapshots
        .snapshots
        .get(start_index)
        .expect("index should be valid if control reaches here");

    // calculate the interpolation percent
    let snapshot_difference = end_snapshot.time - start_snapshot.time;
    let interpolate_distance = playout_time - start_snapshot.time;
    interpolate_values.interpolate_percent =
        interpolate_distance.as_secs_f32() / snapshot_difference.as_secs_f32();

    // scale velocities based on time difference between snapshots
    interpolate_values.velocity_scale = snapshot_difference.as_secs_f32();

    // update the bodies that appear in the two snapshots
    for &(server_entity, snapshot) in start_snapshot.bodies.iter() {
        let Some(body_entity) = map.get_client_entity(server_entity) else {
            continue;
        };

        let Ok(mut interpolation) = body_q.get_mut(body_entity) else {
            continue;
        };

        interpolation.start = Some(snapshot);
    }

    for &(server_entity, snapshot) in end_snapshot.bodies.iter() {
        let Some(body_entity) = map.get_client_entity(server_entity) else {
            continue;
        };

        let Ok(mut interpolation) = body_q.get_mut(body_entity) else {
            continue;
        };

        interpolation.end = Some(snapshot);
    }

    // // snapshot and playout time debug info
    // let earliest = snapshots
    //     .snapshots
    //     .front()
    //     .map(|s| s.time)
    //     .unwrap_or_default();
    // let latest = snapshots
    //     .snapshots
    //     .back()
    //     .map(|s| s.time)
    //     .unwrap_or_default();
    // debug!(
    //     "playout: {:#?} start: {:#?} end: {:#?} earliest: {:#?} latest: {:#?}",
    //     playout_time, start_snapshot.time, end_snapshot.time, earliest, latest
    // );

    // forget snapshots older than the two used
    for _ in 0..start_index {
        snapshots.snapshots.pop_front();
    }
}

fn interpolate_bodies(
    interpolate_values: Res<InterpolateValues>,
    mut body_q: Query<(&SnapshotInterpolation, &mut Position, &mut Rotation)>,
) {
    for (interpolation, mut position, mut rotation) in body_q.iter_mut() {
        // only interpolate if the body is in the start and end snapshot
        let (Some(start), Some(end)) = (interpolation.start, interpolation.end) else {
            continue;
        };

        // hermite interpolation for position
        position.0 = hermite(
            interpolate_values.interpolate_percent,
            start.position,
            start.linear_velocity * interpolate_values.velocity_scale,
            end.position,
            end.linear_velocity * interpolate_values.velocity_scale,
        );

        // spherical linear interpolation for rotation
        rotation.0 = start
            .rotation
            .slerp(end.rotation, interpolate_values.interpolate_percent)
    }
}

/// Computes a hermite spline between two points,
///
/// The start and end velocities are with respect to `p`.
fn hermite(
    p: f32,
    start_position: Vec3,
    start_velocity: Vec3,
    end_position: Vec3,
    end_velocity: Vec3,
) -> Vec3 {
    (start_position * (2. * p.powi(3) - 3. * p.powi(2) + 1.))
        + (end_position * (-2. * p.powi(3) + 3. * p.powi(2)))
        + (start_velocity * (p.powi(3) - 2. * p.powi(2) + p))
        + (end_velocity * (p.powi(3) - p.powi(2)))
}
