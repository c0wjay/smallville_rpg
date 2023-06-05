use bevy::prelude::{Entity, Reflect, Resource};
use std::collections::HashMap;

macro_rules! entitymap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = EntityMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

// Mirrored map that stores the coordinates of entities containing [`Coordinate`].
#[derive(Debug, Resource, Reflect)]
pub struct EntityMap {
    pub entity_map: HashMap<(i32, i32), Vec<Entity>>,
    pub setup_flag: bool,
}

impl EntityMap {
    pub fn new() -> Self {
        EntityMap {
            entity_map: HashMap::new(),
            setup_flag: false,
        }
    }

    pub fn insert(&mut self, coordinate: (i32, i32), entity: Entity) {
        if let Some(entity_vec) = self.entity_map.get_mut(&coordinate) {
            entity_vec.push(entity);
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

    pub fn contains_entity(&self, coordinate: (i32, i32), entity: Entity) -> bool {
        if let Some(entity_vec) = self.entity_map.get(&coordinate) {
            entity_vec.contains(&entity)
        } else {
            false
        }
    }

    pub fn clear(&mut self, coordinate: (i32, i32)) {
        self.entity_map.remove(&coordinate);
    }

    pub fn get(&self, coordinate: (i32, i32)) -> Option<&Vec<Entity>> {
        self.entity_map.get(&coordinate)
    }

    pub fn get_mut(&mut self, coordinate: (i32, i32)) -> Option<&mut Vec<Entity>> {
        self.entity_map.get_mut(&coordinate)
    }
}
