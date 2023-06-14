use bevy::prelude::{
    info, Added, Changed, Commands, Component, Entity, GlobalTransform, Handle, Name, Parent,
    Query, Reflect, ResMut, Resource, Transform, With, Without,
};
use bevy_ecs_ldtk::LdtkLevel;
use bevy_ecs_tilemap::tiles::TilePos;
use std::cmp::{max, min};
use std::collections::HashMap;

use crate::{
    constants::{GRID_OFFSET, GRID_SIZE},
    units::UnitSize,
};

use super::Wall;

#[derive(Clone, Default, Debug, Component, Reflect)]
pub struct Coordinate {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

// Mirrored map that stores the coordinates of entities containing [`Coordinate`].
#[derive(Debug, Resource, Reflect)]
pub struct EntityGridMap {
    pub entity_map: HashMap<(i32, i32), Vec<Entity>>,
}

impl EntityGridMap {
    pub fn new() -> Self {
        EntityGridMap {
            entity_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, coordinate: (i32, i32), entity: Entity) {
        if let Some(entity_vec) = self.entity_map.get_mut(&coordinate) {
            if !entity_vec.contains(&entity) {
                entity_vec.push(entity);
            }
        } else {
            self.entity_map.insert(coordinate, vec![entity]);
        }
    }

    pub fn delete(&mut self, coordinate: (i32, i32), entity: Entity) {
        if let Some(entity_vec) = self.entity_map.get_mut(&coordinate) {
            entity_vec.retain(|&x| x != entity);

            if entity_vec.is_empty() {
                self.entity_map.remove(&coordinate);
            }
        }
    }

    #[allow(dead_code)]
    pub fn contains(&self, coordinate: (i32, i32), entity: Entity) -> bool {
        if let Some(entity_vec) = self.entity_map.get(&coordinate) {
            entity_vec.contains(&entity)
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self, coordinate: (i32, i32)) {
        self.entity_map.remove(&coordinate);
    }

    pub fn get(&self, coordinate: (i32, i32)) -> Option<&Vec<Entity>> {
        self.entity_map.get(&coordinate)
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self, coordinate: (i32, i32)) -> Option<&mut Vec<Entity>> {
        self.entity_map.get_mut(&coordinate)
    }
}

#[derive(Debug, Resource)]
pub struct TileGridMap {
    pub tile_map: HashMap<(i32, i32), (Entity, TileType)>,
    pub max_x: i32,
    pub max_y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileType {
    Wall,
    Floor,
}

impl TileGridMap {
    pub fn new() -> Self {
        TileGridMap {
            tile_map: HashMap::new(),
            max_x: 0,
            max_y: 0,
        }
    }

    pub fn insert(&mut self, coordinate: (i32, i32), entity: Entity, tile_type: TileType) {
        if self.max_x < coordinate.0 {
            self.max_x = coordinate.0;
        }
        if self.max_y < coordinate.1 {
            self.max_y = coordinate.1;
        }
        self.tile_map.insert(coordinate, (entity, tile_type));
    }

    pub fn delete(&mut self, coordinate: (i32, i32)) {
        self.tile_map.remove(&coordinate);
        // Maybe refreshing values of max_x and max_y is not needed.
    }

    pub fn get(&self, coordinate: (i32, i32)) -> Option<&(Entity, TileType)> {
        self.tile_map.get(&coordinate)
    }

    pub fn contains(&self, coordinate: (i32, i32)) -> bool {
        self.tile_map.contains_key(&coordinate)
    }
}

// Modify coordinate of entity when it moves or has been spawned.
pub fn change_coordinate_of_moved_entity(
    mut entity_map: ResMut<EntityGridMap>,
    mut query: Query<(&mut Coordinate, &UnitSize, &Transform, Entity), Changed<Transform>>,
) {
    for (mut coordinate, unit_size, transform, entity) in &mut query {
        let transform_x = transform.translation.x - GRID_OFFSET;
        let transform_y = transform.translation.y - GRID_OFFSET;
        let (min_x, min_y, max_x, max_y) = (
            ((transform_x - unit_size.width) / GRID_SIZE).ceil() as i32,
            ((transform_y - unit_size.height) / GRID_SIZE).ceil() as i32,
            ((transform_x + unit_size.width) / GRID_SIZE) as i32,
            ((transform_y + unit_size.height) / GRID_SIZE) as i32,
        );

        if (
            coordinate.min_x,
            coordinate.min_y,
            coordinate.max_x,
            coordinate.max_y,
        ) == (min_x, min_y, max_x, max_y)
        {
            continue;
        }

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
        } else if min_x > old_min_x {
            for x in old_min_x..min(min_x, old_max_x + 1) {
                for y in old_min_y..=old_max_y {
                    entity_map.delete((x, y), entity);
                }
            }
        }

        if max_x < old_max_x {
            for x in max(max_x + 1, old_min_x)..=old_max_x {
                for y in old_min_y..=old_max_y {
                    entity_map.delete((x, y), entity);
                }
            }
        } else if max_x > old_max_x {
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
        } else if min_y > old_min_y {
            for y in old_min_y..min(min_y, old_max_y + 1) {
                for x in old_min_x..=old_max_x {
                    entity_map.delete((x, y), entity);
                }
            }
        }

        if max_y < old_max_y {
            for y in max(max_y + 1, old_min_y)..=old_max_y {
                for x in old_min_x..=old_max_x {
                    entity_map.delete((x, y), entity);
                }
            }
        } else if max_y > old_max_y {
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

// TODO: need to delete wall entity from entity map
pub fn insert_wall(
    mut tile_map: ResMut<TileGridMap>,
    gparent_query: Query<&GlobalTransform, With<Handle<LdtkLevel>>>,
    parent_query: Query<(&Parent, &Transform), Without<Wall>>,
    wall_query: Query<(Entity, &Parent, &Transform), Added<Wall>>,
) {
    // Calculate Transform of wall entity by referring grand parent entity(World_Level)'s GlobalTransform and parent(Collisions)'s Transform and insert it into tile map.
    for (entity, parent, transform) in wall_query.iter() {
        if let Ok((gparent, p_transform)) = parent_query.get(parent.get()) {
            if let Ok(g_transform) = gparent_query.get(gparent.get()) {
                let translation =
                    g_transform.translation() + p_transform.translation + transform.translation;
                let x = ((translation.x - GRID_OFFSET) / GRID_SIZE) as i32;
                let y = ((translation.y - GRID_OFFSET) / GRID_SIZE) as i32;
                tile_map.insert((x, y), entity, TileType::Wall);
            }
        }
    }
}

pub fn insert_floor(
    mut tile_map: ResMut<TileGridMap>,
    gparent_query: Query<&GlobalTransform, With<Handle<LdtkLevel>>>,
    parent_query: Query<(&Parent, &Transform), Without<Wall>>,
    floor_query: Query<(Entity, &Parent, &Transform), Added<TilePos>>,
) {
    for (entity, parent, transform) in floor_query.iter() {
        if let Ok((gparent, p_transform)) = parent_query.get(parent.get()) {
            if let Ok(g_transform) = gparent_query.get(gparent.get()) {
                let translation =
                    g_transform.translation() + p_transform.translation + transform.translation;
                let x = ((translation.x - GRID_OFFSET) / GRID_SIZE) as i32;
                let y = ((translation.y - GRID_OFFSET) / GRID_SIZE) as i32;
                tile_map.insert((x, y), entity, TileType::Floor);
                info!("insert floor: {:?}, {:?}", entity, (x, y));
            }
        }
    }
}
