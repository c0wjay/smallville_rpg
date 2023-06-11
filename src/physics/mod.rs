use bevy::prelude::{default, App, Plugin, Vec2};
use bevy_rapier2d::prelude::RapierConfiguration;

pub mod collision;
pub mod movement;
pub use collision::*;
pub use movement::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, 0.0),
            ..default()
        })
        .add_system(spawn_wall_collision)
        .add_system(movement);
    }
}
