use bevy::{
    prelude::{Component, Entity, Input, KeyCode, Query, Res, With},
    reflect::Reflect,
};
use bevy_rapier2d::prelude::Velocity;

use crate::{
    sprites::{AnimationIndices, AnimationState, FaceDirection, Facing},
    units::Player,
};

#[derive(Component, Default, Clone, Reflect, Debug)]

pub struct MoveLock(pub bool);

#[derive(Debug)]
pub struct MovementEvent {
    pub moved_entity: Entity,
}

pub fn movement(
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Facing, &mut AnimationIndices, &MoveLock), With<Player>>,
) {
    for (mut velocity, mut facing, mut indices, move_lock) in &mut query {
        velocity.linvel.x = 0.;
        velocity.linvel.y = 0.;
        if !move_lock.0 {
            let right = if input.pressed(KeyCode::D) { 1. } else { 0. };
            let left = if input.pressed(KeyCode::A) { 1. } else { 0. };
            let up = if input.pressed(KeyCode::W) { 1. } else { 0. };
            let down = if input.pressed(KeyCode::S) { 1. } else { 0. };

            velocity.linvel.x = (right - left) * 100.;
            velocity.linvel.y = (up - down) * 100.;

            if input.pressed(KeyCode::D) {
                indices.animation_state = AnimationState::Walk;
                facing.direction = FaceDirection::Right;
            } else if input.pressed(KeyCode::A) {
                indices.animation_state = AnimationState::Walk;
                facing.direction = FaceDirection::Left;
            } else if input.pressed(KeyCode::W) {
                indices.animation_state = AnimationState::Walk;
                facing.direction = FaceDirection::Up;
            } else if input.pressed(KeyCode::S) {
                indices.animation_state = AnimationState::Walk;
                facing.direction = FaceDirection::Down;
            } else {
                indices.animation_state = AnimationState::Idle;
            }
        }

        // TODO: Need to be separate systems, with query filter `Changed<AnimationState>`
        if indices.animation_state == AnimationState::Walk {
            indices.first = 0;
            indices.last = 3;
        } else if indices.animation_state == AnimationState::Attack {
            indices.first = 0;
            indices.last = 4;
        }
    }
}
