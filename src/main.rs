// This example shows off a more in-depth implementation of a game with `bevy_ecs_ldtk`.
// Please run with `--release`.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::LdtkPlugin;
use bevy_rapier2d::prelude::*;

mod camera;
mod combat;
mod constants;
mod inspector;
mod maps;
mod physics;
mod sprites;
mod state;
mod ui;
mod units;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(LdtkPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(maps::MapPlugin)
        .add_plugin(physics::PhysicsPlugin)
        .add_plugin(sprites::SpritesPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(units::UnitsPlugin)
        .add_plugin(combat::CombatPlugin)
        // StatePlugin should be front of ConsolePlugin due to `add_state`.
        .add_plugin(state::StatePlugin)
        .add_plugin(ui::ConsolePlugin)
        .add_plugin(inspector::InspectorPlugin)
        .run();
}
