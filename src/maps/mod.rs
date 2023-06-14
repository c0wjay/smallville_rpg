use bevy::prelude::{App, IntoSystemConfig, IntoSystemSetConfig, Plugin};
use bevy_ecs_ldtk::{
    prelude::LdtkIntCellAppExt, LdtkSettings, LdtkSystemSet, LevelSelection, LevelSpawnBehavior,
    SetClearColor,
};
use bevy_rapier2d::prelude::PhysicsSet;

pub mod ldtk;
pub mod map;
pub use ldtk::*;
pub use map::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
            // Required to prevent race conditions between bevy_ecs_ldtk's and bevy_rapier's systems
            .configure_set(LdtkSystemSet::ProcessApi.before(PhysicsSet::SyncBackend))
            .insert_resource(LevelSelection::Uid(109))
            .insert_resource(LdtkSettings {
                level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                    load_level_neighbors: true,
                },
                set_clear_color: SetClearColor::FromLevelBackground,
                ..Default::default()
            })
            .insert_resource(map::EntityGridMap::new())
            .insert_resource(map::TileGridMap::new())
            .register_ldtk_int_cell::<WallBundle>(1)
            .add_startup_system(setup)
            // wall should be inserted after floor in constructiong tile grid map.
            .add_system(insert_wall.after(insert_floor))
            .add_system(insert_floor)
            .add_system(change_coordinate_of_moved_entity)
            .add_system(update_level_selection);
    }
}
