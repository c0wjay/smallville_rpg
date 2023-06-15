use bevy::{
    prelude::{Component, Entity, Input, KeyCode, Query, Res, With},
    reflect::Reflect,
};
use bevy_rapier2d::prelude::Velocity;

use crate::units::Player;

#[derive(Component, Clone, Reflect, Debug)]

pub struct MoveLock(pub bool);

impl Default for MoveLock {
    fn default() -> Self {
        MoveLock(false)
    }
}

#[derive(Debug)]
pub struct MovementEvent {
    pub moved_entity: Entity,
}

pub fn movement(
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &MoveLock), With<Player>>,
) {
    for (mut velocity, move_lock) in &mut query {
        velocity.linvel.x = 0.;
        velocity.linvel.y = 0.;
        if !move_lock.0 {
            let right = if input.pressed(KeyCode::D) { 1. } else { 0. };
            let left = if input.pressed(KeyCode::A) { 1. } else { 0. };
            let up = if input.pressed(KeyCode::W) { 1. } else { 0. };
            let down = if input.pressed(KeyCode::S) { 1. } else { 0. };

            velocity.linvel.x = (right - left) * 100.;
            velocity.linvel.y = (up - down) * 100.;
        }
    }
}
