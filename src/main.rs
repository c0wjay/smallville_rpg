// This example shows off a more in-depth implementation of a game with `bevy_ecs_ldtk`.
// Please run with `--release`.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};
use bevy_rapier2d::prelude::*;

mod components;
mod constants;
mod map;
mod systems;

// TODO: need to be moduled by plugins.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(LdtkPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // Required to prevent race conditions between bevy_ecs_ldtk's and bevy_rapier's systems
        .configure_set(LdtkSystemSet::ProcessApi.before(PhysicsSet::SyncBackend))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, 0.0),
            ..default()
        })
        .insert_resource(LevelSelection::Uid(109))
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..Default::default()
        })
        .insert_resource(map::EntityMap::new())
        .add_startup_system(systems::setup)
        .add_system(systems::spawn_wall_collision)
        .add_system(systems::movement)
        // .add_system(systems::sprite_size)
        .add_system(systems::animate_sprite)
        .add_system(systems::camera_fit_inside_current_level)
        .add_system(systems::update_level_selection)
        // .add_system(systems::dbg_player_items)
        .register_ldtk_int_cell::<components::WallBundle>(1)
        .register_ldtk_entity::<components::PlayerBundle>("Player")
        .register_ldtk_entity::<components::NPCBundle>("NPC")
        // Type should be registered to view in WorldInspector. Components should be derived from `Reflect` and `Clone`.
        .register_type::<components::Facing>()
        .register_type::<components::MoveLock>()
        .register_type::<components::Coordinate>()
        .register_type::<components::Delay>()
        .register_type::<components::AnimationTimer>()
        .add_system(systems::set_player)
        .add_system(systems::y_sort)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(ResourceInspectorPlugin::<map::EntityMap>::new())
        .add_system(systems::coordinate_setup)
        .add_system(systems::change_coordinate_of_moved_entity)
        .add_event::<components::DamageEvent>()
        .add_system(systems::punching)
        .add_system(systems::melee_attack_system)
        .add_system(systems::collect_hit)
        .add_system(systems::deactivate_attack)
        .run();
}
