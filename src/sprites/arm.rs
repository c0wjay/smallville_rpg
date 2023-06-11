use bevy::{
    prelude::{
        Added, AssetServer, Assets, BuildChildren, Changed, Commands, Component, Entity, Parent,
        Query, Res, ResMut, Vec2, With,
    },
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
};

use super::{AnimationIndices, AnimationState, FaceDirection, Facing};
use crate::units::Player;

#[derive(Copy, Clone, Debug, Default, Component)]
pub struct Arm;

pub fn spawn_arm_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut query: Query<Entity, Added<Player>>,
) {
    for entity in &mut query {
        let texture_handle = asset_server.load("char/player_base.png");
        let texture_atlas = TextureAtlas::from_grid(
            texture_handle,
            Vec2::new(16.0, 32.0),
            12,
            21,
            None,
            Some(Vec2::new(96., 0.)),
        );
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        let arm = commands
            .spawn((
                SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle,
                    sprite: TextureAtlasSprite::new(0),
                    ..Default::default()
                },
                Arm,
            ))
            .id();
        commands.entity(entity).push_children(&[arm]);
    }
}

pub fn animate_arm_sprites(
    player_query: Query<(&Facing, &AnimationIndices), (With<Player>, Changed<AnimationIndices>)>,
    mut arm_query: Query<(&Parent, &mut TextureAtlasSprite), With<Arm>>,
) {
    for (parent, mut sprite) in arm_query.iter_mut() {
        if let Ok((facing, indices)) = player_query.get(parent.get()) {
            // Facing
            sprite.flip_x = facing.direction == FaceDirection::Left;

            // Reset to first frame if idle
            if indices.animation_state == AnimationState::Idle {
                sprite.index = match facing.direction {
                    FaceDirection::Down => 0,
                    FaceDirection::Left | FaceDirection::Right => 12,
                    FaceDirection::Up => 24,
                };
                break;
            }

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
                    FaceDirection::Left | FaceDirection::Right => 12 + temp_index,
                    FaceDirection::Up => 24 + temp_index,
                };
            }

            // Attack animation
            if indices.animation_state == AnimationState::Attack {
                sprite.index = match facing.direction {
                    FaceDirection::Down => 132 + temp_index,
                    FaceDirection::Left | FaceDirection::Right => 96 + temp_index,
                    FaceDirection::Up => {
                        if temp_index == 3 {
                            123
                        } else if temp_index == 4 {
                            122
                        } else {
                            72 + temp_index
                        }
                    }
                };
            }
        }
    }
}
