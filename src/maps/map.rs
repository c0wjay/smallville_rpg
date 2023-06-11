use bevy::prelude::{Changed, Component, Entity, Query, Reflect, ResMut, Resource, Transform};
use std::cmp::{max, min};
use std::collections::HashMap;

use crate::{constants::GRID_SIZE, units::UnitSize};

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

#[allow(unused_macros)]
macro_rules! entitymap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = EntityGridMap::new();
         $( map.insert($key, $val); )*
         map
    }}
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
    pub fn contains_entity(&self, coordinate: (i32, i32), entity: Entity) -> bool {
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

// Modify coordinate of entity when it moves or has been spawned.
pub fn change_coordinate_of_moved_entity(
    mut entity_map: ResMut<EntityGridMap>,
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
