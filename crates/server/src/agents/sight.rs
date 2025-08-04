use std::iter::once;

use avian3d::prelude::*;
use bevy::{platform::collections::HashSet, prelude::*};
use common::GameLayer;

pub fn build(app: &mut App) {
    app.add_systems(Update, cast_sight);
}

/// Contains a list of [SightTarget]s that an agent can see.
#[derive(Component, Default)]
pub struct AgentSight {
    targets: HashSet<Entity>,
}

#[derive(Component)]
#[require(AgentSight)]
pub struct AgentEyes {
    /// The translation of the agents eyes.
    pub offset: Vec3,
    /// The field of view of the agent's eyes measured in radians from the direction they are looking.
    pub fov: f32,
    /// The farthest distance the agent can see.
    pub range: f32,
}

/// Something that an agent can see.
///
/// Must be [GameLayer::Opaque] in order to be seen.
#[derive(Component, Default)]
pub struct SightTarget;

/// A target that agent's will cast rays at and record if they can see a [SightTarget].
#[derive(Component, Default)]
pub struct SightCastTarget;

fn cast_sight(
    mut agent_q: Query<(Entity, &GlobalTransform, &AgentEyes, &mut AgentSight)>,
    sight_targets: Query<(), With<SightTarget>>,
    cast_targets: Query<&GlobalTransform, With<SightCastTarget>>,
    spatial_query: SpatialQuery,
) {
    agent_q.par_iter_mut().for_each(
        |(agent_entity, agent_transform, agent_eyes, mut agent_sight)| {
            let look_origin = *agent_transform * agent_eyes.offset;
            let look_direction = agent_transform.forward();

            agent_sight.targets.clear();

            for cast_transform in &cast_targets {
                let (cast_direction, _) =
                    Dir3::new_and_length(cast_transform.translation() - look_origin)
                        .unwrap_or((Dir3::NEG_Z, 0.));

                let angle_to = cast_direction.angle_between(look_direction.into());

                if angle_to > agent_eyes.fov {
                    continue;
                }

                let Some(RayHitData { entity, .. }) = spatial_query.cast_ray(
                    look_origin,
                    cast_direction,
                    agent_eyes.range,
                    false,
                    &SpatialQueryFilter::from_mask([GameLayer::Opaque])
                        .with_excluded_entities(once(agent_entity)),
                ) else {
                    continue;
                };

                if sight_targets.contains(entity) {
                    agent_sight.targets.insert(entity);
                }
            }
        },
    );
}
