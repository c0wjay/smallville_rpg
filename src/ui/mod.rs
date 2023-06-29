use bevy::prelude::{
    App, CoreSet, IntoSystemAppConfig, IntoSystemConfig, IntoSystemConfigs, OnEnter, OnExit,
    OnUpdate, Plugin,
};
use bevy_tokio_tasks::TokioTasksPlugin;

pub mod inventory;
pub mod npc_console;
use crate::state;
pub use inventory::*;
pub use npc_console::*;

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(build_ui)
            // .add_system(build_ui.in_schedule(OnExit(state::AppState::MainMenu)))     // TODO: Implement this after MainMenu UI is implemented.
            .init_resource::<AskGPT>() // TODO: Remove this after changing `AskGPT` to Component.
            .add_plugin(TokioTasksPlugin::default())
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
            .add_system(send_message_to_chatgpt)
            .add_system(handle_tasks);
    }
}
