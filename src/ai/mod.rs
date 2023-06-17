use bevy::prelude::{App, IntoSystemConfig, IntoSystemConfigs, Plugin, Transform};
use big_brain::{BigBrainPlugin, BigBrainSet};
use seldom_map_nav::prelude::MapNavPlugin;

use crate::maps::insert_wall;

pub mod path_finder;
pub mod state_machine;
pub use path_finder::*;
pub use state_machine::*;

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app
            // This plugin is required for pathfinding and navigation
            // The type parameter is the position component that you use
            .add_plugin(MapNavPlugin::<Transform>::default())
            .add_plugin(BigBrainPlugin)
            .init_resource::<CursorPos>()
            .add_event::<OrderMovementEvent>()
            .add_system(setup.after(insert_wall))
            .add_systems((update_cursor_pos, move_player_when_mouse_click).chain())
            .add_system(processing_order_movement_event)
            .add_system(setup_thinkers)
            .add_systems((push_target_in_range, update_distance_from_target).chain())
            .add_system(remove_target_if_out_of_range)
            .add_system(move_toward_target.in_set(BigBrainSet::Actions))
            .add_system(distance_scorer.in_set(BigBrainSet::Scorers));
    }
}
