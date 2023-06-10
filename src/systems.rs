use crate::components::*;
use crate::map::*;
use bevy::{
    prelude::*, render::view::visibility, render::view::VisibilityPlugin, sprite::Anchor,
    utils::tracing::event,
};
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::egui::text;

use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
    time::Duration,
};

use bevy_rapier2d::prelude::*;

use crate::constants::{ASPECT_RATIO, GRID_SIZE, UNIT_SIZE};

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera = Camera2dBundle::default();
    commands.spawn(camera);

    let ldtk_handle = asset_server.load("Typical_TopDown_example.ldtk");
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: ldtk_handle,
        ..Default::default()
    });

    commands.insert_resource(AnimationTimer(Timer::from_seconds(
        0.1,
        TimerMode::Repeating,
    )));
}

// TODO(DEAD CODE): Maybe it doesn't need.
pub fn coordinate_setup(
    mut entity_map: ResMut<EntityMap>,
    mut query: Query<(Entity, &UnitSize, &mut Coordinate, &Transform)>,
) {
    if entity_map.setup_flag {
        return;
    }

    for (entity, unit_size, mut coordinate, transform) in &mut query {
        // Store coordinates of entities in local component.
        // TODO: check that transform.translation is always positive. if true, then we can use `as u32` instead of `round() as i32`.

        let min_x = (transform.translation.x / GRID_SIZE).ceil() as i32;
        let max_x = ((transform.translation.x + 2. * unit_size.width) / GRID_SIZE).ceil() as i32;

        let min_y = (transform.translation.y / GRID_SIZE).ceil() as i32;
        let max_y = ((transform.translation.y + 2. * unit_size.height) / GRID_SIZE).ceil() as i32;

        coordinate.min_x = min_x;
        coordinate.max_x = max_x;

        coordinate.min_y = min_y;
        coordinate.max_y = max_y;

        // Store coordinates of entities in global entity map.
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                entity_map.insert((x, y), entity);
            }
        }

        entity_map.setup_flag = true;
    }
}

pub fn setup_units(
    mut query: Query<
        (&mut UnitSize, &mut AnimationIndices, &mut YSort),
        Or<(Added<Player>, Added<NPC>)>,
    >,
) {
    for (mut unit_size, mut animation_indices, mut ysort) in &mut query {
        // TODO: This is hard-coded for now. unit_size can be differ for each entity.
        unit_size.width = UNIT_SIZE;
        unit_size.height = UNIT_SIZE;
        ysort.z = 5.0;
    }
}

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

// TODO: Maybe need to be combined with `coordinate_setup`.
// Because There might be spawned entity after game startup, but they will not included into entity_map until they move.
// With a few tests, with ldtk-base spawned entities are successfully included into entity_map without `coordinate_setup`.
// I didn't test with entities spawned after game startup, but it seems `coordinate_setup` is not needed as [bevy-cheatbook](https://bevy-cheatbook.github.io/programming/change-detection.html#filtering) says that `Changed<T>` includes `Added<T>`.
pub fn change_coordinate_of_moved_entity(
    mut entity_map: ResMut<EntityMap>,
    mut query: Query<(&mut Coordinate, &UnitSize, &Transform, Entity), Changed<Transform>>,
) {
    for (mut coordinate, unit_size, transform, entity) in &mut query {
        let (min_x, min_y) = (
            (transform.translation.x / GRID_SIZE).ceil() as i32,
            (transform.translation.y / GRID_SIZE).ceil() as i32,
        );

        if (coordinate.min_x, coordinate.min_y) == (min_x, min_y) {
            continue;
        }

        let (max_x, max_y) = (
            ((transform.translation.x + 2. * unit_size.height) / GRID_SIZE).ceil() as i32,
            ((transform.translation.y + 2. * unit_size.height) / GRID_SIZE).ceil() as i32,
        );

        // TODO: Need to be refactored
        let old_min_x = coordinate.min_x;
        let old_min_y = coordinate.min_y;
        let old_max_x = coordinate.max_x;
        let old_max_y = coordinate.max_y;

        // Update columns of changed coordinates in entity map.
        if min_x < old_min_x {
            for x in min_x..min(max_x + 1, old_min_x) {
                for y in min_y..=max_y {
                    entity_map.insert((x, y), entity);
                }
            }
            for x in max(max_x + 1, old_min_x)..=old_max_x {
                for y in old_min_y..=old_max_y {
                    entity_map.delete((x, y), entity);
                }
            }
        } else if min_x > old_min_x {
            for x in old_min_x..min(min_x, old_max_x + 1) {
                for y in old_min_y..=old_max_y {
                    entity_map.delete((x, y), entity);
                }
            }
            for x in max(old_max_x + 1, min_x)..=max_x {
                for y in min_y..=max_y {
                    entity_map.insert((x, y), entity);
                }
            }
        }

        // Update rows of changed coordinates in entity map.
        if min_y < old_min_y {
            for y in min_y..min(max_y + 1, old_min_y) {
                for x in min_x..=max_x {
                    entity_map.insert((x, y), entity);
                }
            }
            for y in max(max_y + 1, old_min_y)..=old_max_y {
                for x in old_min_x..=old_max_x {
                    entity_map.delete((x, y), entity);
                }
            }
        } else if min_y > old_min_y {
            for y in old_min_y..min(min_y, old_max_y + 1) {
                for x in old_min_x..=old_max_x {
                    entity_map.delete((x, y), entity);
                }
            }
            for y in max(old_max_y + 1, min_y)..=max_y {
                for x in min_x..=max_x {
                    entity_map.insert((x, y), entity);
                }
            }
        }

        // Updating coordinates of entities in local component.
        (coordinate.min_x, coordinate.min_y) = (min_x, min_y);
        (coordinate.max_x, coordinate.max_y) = (max_x, max_y);
    }
}

/// Spawns heron collisions for the walls of a level
///
/// You could just insert a ColliderBundle in to the WallBundle,
/// but this spawns a different collider for EVERY wall tile.
/// This approach leads to bad performance.
///
/// Instead, by flagging the wall tiles and spawning the collisions later,
/// we can minimize the amount of colliding entities.
///
/// The algorithm used here is a nice compromise between simplicity, speed,
/// and a small number of rectangle colliders.
/// In basic terms, it will:
/// 1. consider where the walls are
/// 2. combine wall tiles into flat "plates" in each individual row
/// 3. combine the plates into rectangles across multiple rows wherever possible
/// 4. spawn colliders for each rectangle
pub fn spawn_wall_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
    parent_query: Query<&Parent, Without<Wall>>,
    level_query: Query<(Entity, &Handle<LdtkLevel>)>,
    levels: Res<Assets<LdtkLevel>>,
) {
    /// Represents a wide wall that is 1 tile tall
    /// Used to spawn wall collisions
    #[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    /// A simple rectangle type representing a wall of any size
    struct Rect {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    // Consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    //
    // The key of this map will be the entity of the level the wall belongs to.
    // This has two consequences in the resulting collision entities:
    // 1. it forces the walls to be split along level boundaries
    // 2. it lets us easily add the collision entities as children of the appropriate level entity
    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.for_each(|(&grid_coords, parent)| {
        // An intgrid tile's direct parent will be a layer entity, not the level entity
        // To get the level entity, you need the tile's grandparent.
        // This is where parent_query comes in.
        if let Ok(grandparent) = parent_query.get(parent.get()) {
            level_to_wall_locations
                .entry(grandparent.get())
                .or_default()
                .insert(grid_coords);
        }
    });

    if !wall_query.is_empty() {
        level_query.for_each(|(level_entity, level_handle)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let level = levels
                    .get(level_handle)
                    .expect("Level should be loaded by this point");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level
                    .level
                    .layer_instances
                    .clone()
                    .expect("Level asset should have layers")[0];

                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right edge
                    for x in 0..width + 1 {
                        match (plate_start, level_walls.contains(&GridCoords { x, y })) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => plate_start = Some(x),
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut rect_builder: HashMap<Plate, Rect> = HashMap::new();
                let mut prev_row: Vec<Plate> = Vec::new();
                let mut wall_rects: Vec<Rect> = Vec::new();

                // an extra empty row so the algorithm "finishes" the rects that touch the top edge
                plate_stack.push(Vec::new());

                for (y, current_row) in plate_stack.into_iter().enumerate() {
                    for prev_plate in &prev_row {
                        if !current_row.contains(prev_plate) {
                            // remove the finished rect so that the same plate in the future starts a new rect
                            if let Some(rect) = rect_builder.remove(prev_plate) {
                                wall_rects.push(rect);
                            }
                        }
                    }
                    for plate in &current_row {
                        rect_builder
                            .entry(plate.clone())
                            .and_modify(|e| e.top += 1)
                            .or_insert(Rect {
                                bottom: y as i32,
                                top: y as i32,
                                left: plate.left,
                                right: plate.right,
                            });
                    }
                    prev_row = current_row;
                }

                commands.entity(level_entity).with_children(|level| {
                    // Spawn colliders for every rectangle..
                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    for wall_rect in wall_rects {
                        level
                            .spawn_empty()
                            .insert(Collider::cuboid(
                                (wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                                (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                            ))
                            .insert(RigidBody::Fixed)
                            .insert(Friction::new(1.0))
                            .insert(Transform::from_xyz(
                                (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32
                                    / 2.,
                                (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32
                                    / 2.,
                                0.,
                            ))
                            .insert(GlobalTransform::default());
                    }
                });
            }
        });
    }
}

#[allow(clippy::type_complexity)]
pub fn camera_fit_inside_current_level(
    mut camera_query: Query<
        (
            &mut bevy::render::camera::OrthographicProjection,
            &mut Transform,
        ),
        Without<Player>,
    >,
    player_query: Query<&Transform, With<Player>>,
    level_query: Query<
        (&Transform, &Handle<LdtkLevel>),
        (Without<OrthographicProjection>, Without<Player>),
    >,
    level_selection: Res<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    if let Ok(Transform {
        translation: player_translation,
        ..
    }) = player_query.get_single()
    {
        let player_translation = *player_translation;

        let (mut orthographic_projection, mut camera_transform) = camera_query.single_mut();

        for (level_transform, level_handle) in &level_query {
            if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
                let level = &ldtk_level.level;
                if level_selection.is_match(&0, level) {
                    let level_ratio = level.px_wid as f32 / ldtk_level.level.px_hei as f32;
                    orthographic_projection.viewport_origin = Vec2::ZERO;
                    if level_ratio > ASPECT_RATIO {
                        // level is wider than the screen
                        let height = (level.px_hei as f32 / 9.).round() * 9.;
                        let width = height * ASPECT_RATIO;
                        orthographic_projection.scaling_mode =
                            bevy::render::camera::ScalingMode::Fixed { width, height };
                        camera_transform.translation.x =
                            (player_translation.x - level_transform.translation.x - width / 2.)
                                .clamp(0., level.px_wid as f32 - width);
                        camera_transform.translation.y = 0.;
                    } else {
                        // level is taller than the screen
                        let width = (level.px_wid as f32 / 16.).round() * 16.;
                        let height = width / ASPECT_RATIO;
                        orthographic_projection.scaling_mode =
                            bevy::render::camera::ScalingMode::Fixed { width, height };
                        camera_transform.translation.y =
                            (player_translation.y - level_transform.translation.y - height / 2.)
                                .clamp(0., level.px_hei as f32 - height);
                        camera_transform.translation.x = 0.;
                    }

                    camera_transform.translation.x += level_transform.translation.x;
                    camera_transform.translation.y += level_transform.translation.y;
                }
            }
        }
    }
}

pub fn update_level_selection(
    level_query: Query<(&Handle<LdtkLevel>, &Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    mut level_selection: ResMut<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    for (level_handle, level_transform) in &level_query {
        if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
            let level_bounds = Rect {
                min: Vec2::new(level_transform.translation.x, level_transform.translation.y),
                max: Vec2::new(
                    level_transform.translation.x + ldtk_level.level.px_wid as f32,
                    level_transform.translation.y + ldtk_level.level.px_hei as f32,
                ),
            };

            for player_transform in &player_query {
                if player_transform.translation.x < level_bounds.max.x
                    && player_transform.translation.x > level_bounds.min.x
                    && player_transform.translation.y < level_bounds.max.y
                    && player_transform.translation.y > level_bounds.min.y
                    && !level_selection.is_match(&0, &ldtk_level.level)
                {
                    *level_selection = LevelSelection::Iid(ldtk_level.level.iid.clone());
                }
            }
        }
    }
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

pub fn y_sort(mut q: Query<(&mut Transform, &YSort)>) {
    for (mut tf, ysort) in q.iter_mut() {
        tf.translation.z = ysort.z - (1.0f32 / (1.0f32 + (2.0f32.powf(-0.01 * tf.translation.y))));
    }
}

pub fn attack(
    input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut fighters: Query<
        (
            &mut AnimationIndices,
            &mut MoveLock,
            &Facing,
            &mut Velocity,
            &mut Delay,
            Entity,
        ),
        With<Player>,
    >,
    time: Res<Time>,
) {
    for (mut indices, mut move_lock, facing, mut velocity, mut delay, entity) in &mut fighters {
        delay.tick(time.delta());
        if input.pressed(KeyCode::Space) && !move_lock.0 {
            // const MOVE_FRONT: f32 = 100.;
            // match facing.direction {
            //     FaceDirection::Down => velocity.linvel.y = -MOVE_FRONT,
            //     FaceDirection::Left => velocity.linvel.x = -MOVE_FRONT,
            //     FaceDirection::Right => velocity.linvel.x = MOVE_FRONT,
            //     FaceDirection::Up => velocity.linvel.y = MOVE_FRONT,
            // }

            indices.animation_state = AnimationState::Attack;

            // Spawn the attack entity
            let attack_entity = commands
                .spawn(CollisionGroups::new(
                    BodyLayers::PLAYER_ATTACK,
                    BodyLayers::ENEMY,
                ))
                .insert(Attack {
                    damage: 1,
                    pushback: Vec2::ZERO,
                    hitstun_duration: 1.,
                })
                .id();
            *delay = Delay(Timer::from_seconds(1., TimerMode::Once));
            commands.entity(entity).push_children(&[attack_entity]);
            info!("Spawned attack entity: {:?}", attack_entity);

            move_lock.0 = true;
        }
    }
}

pub fn melee_attack_system(
    entity_map: Res<EntityMap>,
    attacks: Query<(&Parent, &Attack)>,
    attackers: Query<(&Facing, &Coordinate)>,
    hurtboxes: Query<&Parent, With<Hurtbox>>,
    mut event_writer: EventWriter<DamageEvent>,
) {
    for (attacker, attack) in attacks.iter() {
        let attacker_entity = attacker.get();

        // TODO: under code should be separate systems. Maybe `fn find_entities_in_range`, and add entities in player range into player entitiy's children entities.
        // This always check player's range at each frame and update children entities. This system can enable to emphasize entities nearby.
        let (attacker_facing, attacker_coordinate) = attackers
            .get(attacker_entity)
            .expect("Attacker entity must have `Facing` and `Coordinate` components");

        let (range_x, range_y) = match attacker_facing.direction {
            FaceDirection::Down => (
                (attacker_coordinate.min_x..=attacker_coordinate.max_x),
                (attacker_coordinate.min_y - 1..=attacker_coordinate.min_y - 1), // Maybe have to range in min_y-1..=min_y, because of very small & adjoined objects
            ),
            FaceDirection::Left => (
                (attacker_coordinate.min_x - 1..=attacker_coordinate.min_x - 1),
                (attacker_coordinate.min_y..=attacker_coordinate.max_y),
            ),
            FaceDirection::Right => (
                (attacker_coordinate.max_x + 1..=attacker_coordinate.max_x + 1),
                (attacker_coordinate.min_y..=attacker_coordinate.max_y),
            ),
            FaceDirection::Up => (
                (attacker_coordinate.min_x..=attacker_coordinate.max_x),
                (attacker_coordinate.max_y + 1..=attacker_coordinate.max_y + 1),
            ),
        };

        let mut hurtbox_vec = Vec::new();
        for x in range_x {
            for y in range_y.clone() {
                if let Some(hit_range) = entity_map.get((x, y)) {
                    for hurtbox_entity in hit_range {
                        if attacker_entity.eq(hurtbox_entity)
                            | hurtbox_vec.contains(&hurtbox_entity)
                        {
                            continue;
                        }
                        hurtbox_vec.push(hurtbox_entity);
                    }
                }
            }
        }

        info!("Attacker: {:?}", attacker_entity);
        info!("Attack: {:?}", attack);

        for hurtbox_entity in hurtbox_vec {
            info!("Hurtbox: {:?}", hurtbox_entity);

            event_writer.send(DamageEvent {
                damageing_entity: attacker_entity,
                damage_velocity: attack.pushback,
                damage: attack.damage,
                damaged_entity: *hurtbox_entity,
                hitstun_duration: attack.hitstun_duration,
            });
        }
    }
}

// Previous version of melee_attack_system. But using CollisionEvent is better for projectiles.
// TODO: reuse this code for projectile attacks
pub fn projectile_attack_system(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    attacks: Query<(&Parent, Entity, &Attack)>,
    hurtboxes: Query<&Parent, With<Hurtbox>>,
    mut event_writer: EventWriter<DamageEvent>,
) {
    for event in events.iter() {
        info!("Event: {:?}", event);
        if let CollisionEvent::Started(e1, e2, _flags) = event {
            for (attacker, _, attack) in attacks.iter() {
                let (attacker_entity, hurtbox_entity) = if attacker.get() == *e1 {
                    (*e1, *e2)
                } else if attacker.get() == *e2 {
                    (*e2, *e1)
                } else {
                    continue;
                };
                info!("Attacker: {:?}", attacker_entity);
                info!("Attack: {:?}", attack);

                if attacker_entity.eq(e1) | attacker_entity.eq(e2) {
                    event_writer.send(DamageEvent {
                        damageing_entity: attacker_entity,
                        damage_velocity: attack.pushback,
                        damage: attack.damage,
                        damaged_entity: hurtbox_entity,
                        hitstun_duration: attack.hitstun_duration,
                    });
                    break;
                }
                info!("message should not be shown");
            }
        }
    }
}

pub fn collect_hit(
    mut npc: Query<&mut Visibility, Or<(With<NPC>, With<Player>)>>,
    mut damage_events: EventReader<DamageEvent>,
) {
    for event in damage_events.iter() {
        info!("Damage event: {:?}", event);
        if let Ok(mut visibility) = npc.get_mut(event.damaged_entity) {
            info!("NPC {:?} hit {:?}", event.damaged_entity, visibility);
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn deactivate_attack(
    mut commands: Commands,
    attacks: Query<(&Parent, Entity, &Attack)>,
    mut player: Query<&mut Delay, With<Player>>,
    time: Res<Time>,
) {
    for (parent, entity, attack) in attacks.iter() {
        let delay = player.get_mut(parent.get());
        if !delay.is_err() {
            let delay = &mut delay.unwrap().0;
            // info!("Timer: {:?}", timer);
            delay.tick(time.delta());
            if delay.finished() {
                warn!("Attack {:?} demolished", entity);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
