use bevy::prelude::{App, IntoSystemConfig, IntoSystemConfigs, Plugin, Transform};
use seldom_map_nav::prelude::MapNavPlugin;

use crate::maps::insert_wall;

pub mod path_finder;
pub use path_finder::*;

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app
            // This plugin is required for pathfinding and navigation
            // The type parameter is the position component that you use
            .add_plugin(MapNavPlugin::<Transform>::default())
            .init_resource::<CursorPos>()
            .add_event::<OrderMovementEvent>()
            .add_system(setup.after(insert_wall))
            .add_systems((update_cursor_pos, move_player_when_mouse_click).chain())
            .add_system(processing_order_movement_event);
    }
}
