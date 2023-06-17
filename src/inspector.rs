use bevy::prelude::{App, Plugin};
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};

use crate::{ai, combat, maps, physics, sprites, ui};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WorldInspectorPlugin::new())
            .add_plugin(ResourceInspectorPlugin::<maps::EntityGridMap>::new())
            .add_plugin(ResourceInspectorPlugin::<ai::CursorPos>::new())
            // Type should be registered to view in WorldInspector. Components should be derived from `Reflect` and `Clone`.
            .register_type::<sprites::Facing>()
            .register_type::<physics::MoveLock>()
            .register_type::<maps::Coordinate>()
            .register_type::<combat::Delay>()
            .register_type::<ui::ConsoleData>()
            .register_type::<sprites::AnimationIndices>()
            .register_type::<seldom_map_nav::prelude::Pathfind>()
            .register_type::<seldom_map_nav::prelude::Nav>()
            .register_type::<ai::Distance>()
            .register_type::<ai::Approach>()
            .register_type::<big_brain::thinker::Actor>()
            .register_type::<big_brain::scorers::Score>()
            .register_type::<big_brain::actions::ActionState>();
    }
}
