use bevy::{
    prelude::{
        Bundle, Commands, Component, Deref, DerefMut, Query, Res, ResMut, Resource, Transform, With,
    },
    reflect::Reflect,
    sprite::TextureAtlasSprite,
    time::{Time, Timer, TimerMode},
};

use crate::units::Player;

#[derive(Clone, Default, Bundle)]
pub struct AnimationBundle {
    pub animation_indices: AnimationIndices,
    pub facing: Facing,
}

#[derive(Component, Default, Clone)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
    pub current: usize,
    pub animation_state: AnimationState,
}

#[derive(Component, Default, Clone, PartialEq, Eq)]
pub enum AnimationState {
    #[default]
    Idle,
    Walk,
    Attack,
    #[allow(dead_code)]
    BeHit,
}

#[derive(Component, Default, Clone, Reflect, Debug)]
pub struct Facing {
    pub direction: FaceDirection,
}

#[derive(Component, Default, Clone, PartialEq, Eq, Reflect, Debug)]
pub enum FaceDirection {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

#[derive(Resource, Deref, DerefMut, Clone)]
pub struct AnimationTimer(pub Timer);

impl Default for AnimationTimer {
    fn default() -> Self {
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}

#[derive(Copy, Clone, Debug, Default, Component)]
pub struct Body;

#[derive(Component, Default, Clone, Debug)]
pub struct YSort {
    pub z: f32,
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(AnimationTimer(Timer::from_seconds(
        0.1,
        TimerMode::Repeating,
    )));
}

// pub fn sprite_size(mut query: Query<&mut TextureAtlasSprite, Or<(With<Player>, With<NPC>)>>) {
//     for mut sprite in &mut query {
//         sprite.custom_size = Some(Vec2::new(24., 24.));
//     }
// }

pub fn animate_sprite(
    time: Res<Time>,
    mut timer: ResMut<AnimationTimer>,
    mut query: Query<(&mut TextureAtlasSprite, &Facing, &mut AnimationIndices), With<Player>>,
) {
    for (mut sprite, facing, mut indices) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            // Facing
            sprite.flip_x = facing.direction == FaceDirection::Left;

            // Reset to first frame if idle
            if indices.animation_state == AnimationState::Idle {
                sprite.index = match facing.direction {
                    FaceDirection::Down => 0,
                    FaceDirection::Left | FaceDirection::Right => 6,
                    FaceDirection::Up => 12,
                };
                indices.current = 0;
                break;
            }

            // Move to next frame
            indices.current = if indices.current == indices.last {
                indices.first
            } else {
                indices.current + 1
            };

            let mut temp_index = indices.current;

            // Walking animation
            if indices.animation_state == AnimationState::Walk {
                if indices.current == 2 {
                    temp_index = 0;
                } else if indices.current == 3 {
                    temp_index = 2;
                }

                sprite.index = match facing.direction {
                    FaceDirection::Down => temp_index,
                    FaceDirection::Left | FaceDirection::Right => 6 + temp_index,
                    FaceDirection::Up => 12 + temp_index,
                };
            }

            // Attack animation
            if indices.animation_state == AnimationState::Attack {
                sprite.index = match facing.direction {
                    FaceDirection::Down => 66 + temp_index,
                    FaceDirection::Left | FaceDirection::Right => 48 + temp_index,
                    FaceDirection::Up => {
                        if temp_index == 3 {
                            63
                        } else if temp_index == 4 {
                            62
                        } else {
                            36 + temp_index
                        }
                    }
                };
            }
        }
    }
}

pub fn y_sort(mut q: Query<(&mut Transform, &YSort)>) {
    for (mut tf, ysort) in q.iter_mut() {
        tf.translation.z = ysort.z - (1.0f32 / (1.0f32 + (2.0f32.powf(-0.01 * tf.translation.y))));
    }
}
