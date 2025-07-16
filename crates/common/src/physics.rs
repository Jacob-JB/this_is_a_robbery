use std::time::Duration;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ServerEntity;

/// Physics snapshot sent from server to client.
#[derive(Serialize, Deserialize)]
pub struct PhysicsSnapshot {
    pub bodies: Vec<(ServerEntity, PhysicsBodySnapshot)>,
    pub time: Duration,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct PhysicsBodySnapshot {
    pub position: Vec3,
    pub linear_velocity: Vec3,
    pub rotation: Quat,
}

/// A sample of the server's physics time sent
/// to clients for the clients to average.
///
/// Is it's own message and not part of snapshots
/// to minimize message delay.
#[derive(Serialize, Deserialize)]
pub struct TimeSample {
    pub time: Duration,
}
