pub mod npc_console;
use crate::state;
pub use npc_console::*;

use bevy::prelude::{
    App, CoreSet, IntoSystemAppConfig, IntoSystemConfig, IntoSystemConfigs, OnEnter, OnExit,
    OnUpdate, Plugin,
};

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(build_ui)
            // .add_system(build_ui.in_schedule(OnExit(state::AppState::MainMenu)))
            .add_system(open_npc_console.in_schedule(OnEnter(state::AppState::ConsoleOpenedState)))
            .add_systems(
                (update_logs_area, handle_input_keys, update_enter_command)
                    .in_set(OnUpdate(state::AppState::ConsoleOpenedState)),
            )
            .add_system(
                commands_handler
                    .in_set(OnUpdate(state::AppState::ConsoleOpenedState))
                    .run_if(should_run_cmd_handler)
                    .before(handle_input_keys),
            )
            .add_system(apply_animation.in_base_set(CoreSet::PostUpdate))
            .add_system(close_npc_console.in_schedule(OnExit(state::AppState::ConsoleOpenedState)))
            .add_system(push_message_events_to_console)
            .add_system(interact_with_npc)
            .add_system(mouse_scroll)
            .add_event::<PrintConsoleEvent>()
            .add_event::<EnteredConsoleCommandEvent>()
            .insert_resource(ConsoleAnimation {
                moving_speed: 15.0,
                ..Default::default()
            })
            .add_system(spawn_console_data_in_npc.in_base_set(CoreSet::PostUpdate))
        // .insert_resource(ConsoleData::default())
        ;
    }
}
