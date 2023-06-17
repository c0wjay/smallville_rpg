use bevy::{
    prelude::{
        debug, warn, Added, Commands, Component, Entity, EventWriter, Query, Res, Transform, With,
    },
    reflect::Reflect,
};
use big_brain::{
    prelude::{ActionBuilder, ActionState, Highest, ScorerBuilder},
    scorers::Score,
    thinker::{ActionSpan, Actor, ScorerSpan, Thinker},
};
use seldom_map_nav::prelude::PathTarget;

use crate::{
    constants::GRID_SIZE,
    maps::{Coordinate, EntityGridMap},
    units::NPC,
};

use super::OrderMovementEvent;

// TODO: Distance & Approach should be refactored. Redesign Component to be more suitable at big-brain pattern.
// Score
#[derive(Component, Clone, Debug, Reflect)]
pub struct Distance {
    // TODO: target can be Static and Dinamic. Static is coordinate, Dinamic is Entity.
    pub target: Entity,
    // TODO: Range should be independent Component Later. (This can be used for not only movement but also attack or other events.)
    // Use actor's Coordinate + range in EntityGridMap.
    pub range: i32,
    // Calculated distance between actor and target's transform.translation
    pub distance: f32,
}

impl Distance {
    pub fn new(range: i32) -> Self {
        let target = Entity::from_raw(0);
        let distance = -1.0;
        Self {
            target,
            range,
            distance,
        }
    }

    pub fn set_target(&mut self, target: Entity) {
        if self.distance < 0. {
            self.target = target;
        }
    }

    pub fn update_distance(&mut self, target: Entity, distance: f32) {
        if self.target == target {
            self.distance = distance;
        }
    }

    pub fn reset(&mut self) {
        self.target = Entity::from_raw(0);
        self.distance = -1.0;
    }

    pub fn has_target(&self) -> bool {
        self.target != Entity::from_raw(0) && self.distance >= 0.
    }
}

pub fn push_target_in_range(
    entity_map: Res<EntityGridMap>,
    mut actor_query: Query<(Entity, &Coordinate, &Transform, &mut Distance)>,
    transform_query: Query<&Transform>,
) {
    for (actor_entity, actor_coordinate, actor_transform, mut distance) in actor_query.iter_mut() {
        // If I use this checking code, target will not be changed until it is out of range.
        // If I not use this code, target will be changed to most closest one every frame.
        if distance.has_target() {
            continue;
        }

        // TODO: need to be refactored.
        for x in actor_coordinate.min_x - distance.range..=actor_coordinate.max_x + distance.range {
            for y in
                actor_coordinate.min_y - distance.range..=actor_coordinate.max_y + distance.range
            {
                if let Some(entity_vec) = entity_map.get((x, y)) {
                    for entity in entity_vec {
                        if entity.eq(&actor_entity) {
                            continue;
                        }

                        if let Ok(transform) = transform_query.get(*entity) {
                            let new_distance =
                                actor_transform.translation.distance(transform.translation);

                            if distance.distance < 0. || new_distance < distance.distance {
                                distance.set_target(*entity);
                                distance.update_distance(*entity, new_distance);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn update_distance_from_target(
    mut actor_query: Query<(&Transform, &mut Distance)>,
    target_query: Query<&Transform>,
) {
    for (actor_transform, mut distance) in actor_query.iter_mut() {
        if !distance.has_target() {
            continue;
        }

        if let Ok(target_transform) = target_query.get(distance.target) {
            distance.distance = actor_transform
                .translation
                .distance(target_transform.translation);
        } else {
            distance.reset();
        }
    }
}

pub fn remove_target_if_out_of_range(
    mut actor_query: Query<(&Transform, &mut Distance)>,
    target_query: Query<&Transform>,
) {
    for (actor_transform, mut distance) in actor_query.iter_mut() {
        if !distance.has_target() {
            continue;
        }

        if let Ok(target_transform) = target_query.get(distance.target) {
            let new_distance = actor_transform
                .translation
                .distance(target_transform.translation);

            if new_distance > (distance.range as f32) * GRID_SIZE {
                distance.reset();
            }
        } else {
            distance.reset();
        }
    }
}

// Action 1
#[derive(Clone, Component, Debug, ActionBuilder, Reflect)]
pub struct Approach {
    pub speed: f32,
}

pub fn move_toward_target(
    mut actor_query: Query<(&Actor, &mut ActionState, &ActionSpan, &Approach)>,
    distances: Query<&Distance>,
    mut movement_writer: EventWriter<OrderMovementEvent>,
) {
    for (Actor(actor), mut state, span, approach) in actor_query.iter_mut() {
        // This sets up the tracing scope. Any `debug` calls here will be
        // spanned together in the output.
        let _guard = span.span().enter();

        if let Ok(distance) = distances.get(*actor) {
            match *state {
                ActionState::Requested => {
                    debug!("Move Start!");
                    if !distance.has_target() {
                        *state = ActionState::Cancelled;
                        continue;
                    }
                    movement_writer.send(OrderMovementEvent {
                        mover: *actor,
                        destination: PathTarget::Dynamic(distance.target),
                        speed: approach.speed,
                    });
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if !distance.has_target() {
                        *state = ActionState::Cancelled;
                        continue;
                    }
                    if distance.distance < 8. {
                        debug!("Move End!");
                        *state = ActionState::Success;
                    } else {
                        *state = ActionState::Requested;
                    }
                }
                ActionState::Cancelled => {
                    debug!("Move Cancelled!");
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

// TODO: Maybe merge with combat::Attack component
// Action 2
#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct Attack {
    pub target: Entity,
    pub damage: f32,
}

// Scorer
#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct DistanceChecker;

pub fn distance_scorer(
    distances: Query<&Distance>,
    mut scorer_query: Query<(&Actor, &mut Score, &ScorerSpan), With<DistanceChecker>>,
) {
    for (Actor(actor), mut score, span) in scorer_query.iter_mut() {
        if let Ok(distance) = distances.get(*actor) {
            if !distance.has_target() {
                continue;
            }

            // score increases as target is closer.
            score.set(1. - (distance.distance / (distance.range as f32 * GRID_SIZE)).clamp(0., 1.));
            if distance.distance < 16.0 {
                span.span()
                    .in_scope(|| debug!("Target has been closer {}", distance.distance));
            }
        }
    }
}

// Init Thinkers
pub fn setup_thinkers(mut commands: Commands, npc: Query<Entity, Added<NPC>>) {
    for entity in npc.iter() {
        commands.entity(entity).insert((
            Distance::new(3),
            Thinker::build()
                .label("NPC Brain")
                .picker(Highest)
                .when(DistanceChecker, Approach { speed: 100. }),
        ));
    }
}
