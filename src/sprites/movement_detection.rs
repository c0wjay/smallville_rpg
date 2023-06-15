use bevy::prelude::{Changed, Query, Transform};
use bevy_rapier2d::prelude::Velocity;
use seldom_map_nav::prelude::{Nav, Pathfind};

use crate::physics::MoveLock;

use super::{AnimationIndices, AnimationState, FaceDirection, Facing};

pub fn change_animation_state_when_move(
    mut query: Query<(&Velocity, &MoveLock, &mut Facing, &mut AnimationIndices), Changed<Velocity>>,
) {
    for (velocity, move_lock, mut facing, mut indices) in &mut query {
        if !move_lock.0 {
            let x_speed = velocity.linvel.x;
            let y_speed = velocity.linvel.y;

            if !(x_speed.round() == 0. && y_speed.round() == 0.) {
                indices.animation_state = AnimationState::Walk;

                if x_speed.abs() > y_speed.abs() {
                    if x_speed > 0. {
                        facing.direction = FaceDirection::Right;
                    } else {
                        facing.direction = FaceDirection::Left;
                    }
                } else {
                    if y_speed > 0. {
                        facing.direction = FaceDirection::Up;
                    } else {
                        facing.direction = FaceDirection::Down;
                    }
                }
            }
        }
    }
}

pub fn change_animation_state_with_navigation(
    mut query: Query<(
        &Transform,
        &mut Facing,
        &mut AnimationIndices,
        &Nav,
        &Pathfind,
    )>,
) {
    for (transform, mut facing, mut indices, nav, pathfind) in query.iter_mut() {
        if nav.done {
            continue;
        }

        let path = &pathfind.path;
        if path.is_empty() {
            continue;
        }

        let dest = *path.front().unwrap();
        let pos = transform.translation.truncate();
        let dest_diff = dest - pos;

        if !((dest_diff.x.round(), dest_diff.y.round()) == (0., 0.)) {
            indices.animation_state = AnimationState::Walk;

            if dest_diff.x.abs() > dest_diff.y.abs() {
                if dest_diff.x > 0. {
                    facing.direction = FaceDirection::Right;
                } else {
                    facing.direction = FaceDirection::Left;
                }
            } else {
                if dest_diff.y > 0. {
                    facing.direction = FaceDirection::Up;
                } else {
                    facing.direction = FaceDirection::Down;
                }
            }
        }
    }
}

pub fn set_to_idle_when_stop(
    mut query: Query<
        (&Velocity, &MoveLock, &mut AnimationIndices, Option<&Nav>),
        Changed<Velocity>,
    >,
) {
    for (velocity, move_lock, mut indices, nav) in &mut query {
        if !move_lock.0 {
            if let Some(nav) = nav {
                if !nav.done {
                    continue;
                }
            }

            if velocity.linvel.x.round() == 0. && velocity.linvel.y.round() == 0. {
                indices.animation_state = AnimationState::Idle;
            }
        }
    }
}

pub fn change_animation_indices_with_state(
    mut query: Query<&mut AnimationIndices, Changed<AnimationIndices>>,
) {
    for mut indices in &mut query {
        if indices.animation_state == AnimationState::Walk {
            indices.first = 0;
            indices.last = 3;
        } else if indices.animation_state == AnimationState::Attack {
            indices.first = 0;
            indices.last = 4;
        } else {
            indices.first = 0;
            indices.last = 0;
        }
    }
}
