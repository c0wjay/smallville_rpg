use bevy::{
    prelude::{
        Added, AssetServer, Assets, BuildChildren, Changed, Commands, Component, Entity, Parent,
        Query, Res, ResMut, Transform, Vec2, Visibility, With,
    },
    sprite::{Anchor, SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
};

use super::{AnimationIndices, AnimationState, FaceDirection, Facing};
use crate::units::Player;

#[derive(Copy, Clone, Debug, Default, Component)]
pub struct Weapon;

pub fn spawn_weapon_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut query: Query<Entity, Added<Player>>,
) {
    for entity in &mut query {
        let texture_handle = asset_server.load("char/tools.png");
        let texture_atlas = TextureAtlas::from_grid(
            texture_handle,
            Vec2::new(16.0, 24.0),
            6,
            6,
            None,
            Some(Vec2::new(0., 24.)),
        );
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        let weapon = commands
            .spawn((
                SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle,
                    sprite: TextureAtlasSprite::new(0),
                    visibility: Visibility::Hidden,
                    ..Default::default()
                },
                Weapon,
            ))
            .id();
        commands.entity(entity).push_children(&[weapon]);
    }
}

pub fn animate_weapon_sprites(
    player_query: Query<(&Facing, &AnimationIndices), (With<Player>, Changed<AnimationIndices>)>,
    mut weapon_query: Query<
        (
            &Parent,
            &mut TextureAtlasSprite,
            &mut Transform,
            &mut Visibility,
        ),
        With<Weapon>,
    >,
) {
    for (parent, mut sprite, mut transform, mut visibility) in weapon_query.iter_mut() {
        if let Ok((facing, indices)) = player_query.get(parent.get()) {
            // Facing
            sprite.flip_x = facing.direction == FaceDirection::Left;

            // Reset to first frame if idle
            if indices.animation_state != AnimationState::Attack {
                *visibility = Visibility::Hidden;
                break;
            }
            *visibility = Visibility::Visible;
            (
                transform.rotation.x,
                transform.rotation.z,
                transform.rotation.w,
            ) = (0., 0., 0.);

            match facing.direction {
                FaceDirection::Down => {
                    if indices.current < 2 {
                        sprite.index = 24 + 0;
                        sprite.anchor = Anchor::BottomCenter;
                    } else if indices.current < 4 {
                        sprite.index = 24 + 1;
                        sprite.anchor = Anchor::Center;
                    } else {
                        sprite.index = 24 + 1;
                        (transform.rotation.x, transform.rotation.w) = (-0.3_f32).sin_cos();
                        sprite.anchor = Anchor::TopCenter;
                    }
                }
                FaceDirection::Left => {
                    sprite.index = 24 + 2;
                    match indices.current {
                        0 => {
                            (transform.rotation.z, transform.rotation.w) = (-0.1_f32).sin_cos();
                            sprite.anchor = Anchor::Custom((-0.35, -0.75).into())
                        }
                        1 => {
                            (transform.rotation.z, transform.rotation.w) = (0.1_f32).sin_cos();
                            sprite.anchor = Anchor::Custom((0., -0.7).into())
                        }
                        2 => {
                            (transform.rotation.z, transform.rotation.w) = (0.3_f32).sin_cos();
                            sprite.anchor = Anchor::Custom((0.2, -0.7).into())
                        }
                        3 => {
                            (transform.rotation.z, transform.rotation.w) = (0.5_f32).sin_cos();
                            sprite.anchor = Anchor::Custom((0.2, -0.7).into())
                        }
                        4 => {
                            (transform.rotation.z, transform.rotation.w) = (0.7_f32).sin_cos();
                            sprite.anchor = Anchor::Custom((0.2, -0.7).into())
                        }
                        _ => transform.rotation.z = 0.,
                    }
                }
                FaceDirection::Right => {
                    sprite.index = 24 + 2;
                    match indices.current {
                        0 => {
                            (transform.rotation.z, transform.rotation.w) = (0.1_f32).sin_cos();
                            sprite.anchor = Anchor::Custom((0.35, -0.75).into())
                        }
                        1 => {
                            (transform.rotation.z, transform.rotation.w) = (-0.1_f32).sin_cos();
                            sprite.anchor = Anchor::Custom((0., -0.7).into())
                        }
                        2 => {
                            (transform.rotation.z, transform.rotation.w) = (-0.3_f32).sin_cos();
                            sprite.anchor = Anchor::Custom((-0.2, -0.7).into())
                        }
                        3 => {
                            (transform.rotation.z, transform.rotation.w) = (-0.5_f32).sin_cos();
                            sprite.anchor = Anchor::Custom((-0.2, -0.7).into())
                        }
                        4 => {
                            (transform.rotation.z, transform.rotation.w) = (-0.7_f32).sin_cos();
                            sprite.anchor = Anchor::Custom((-0.2, -0.7).into())
                        }
                        _ => transform.rotation.z = 0.,
                    }
                }
                FaceDirection::Up => {
                    sprite.index = if indices.current < 2 { 24 + 3 } else { 24 + 4 };
                    sprite.anchor = Anchor::BottomCenter;
                }
            };
        }
    }
}
