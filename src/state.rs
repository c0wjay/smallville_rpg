use bevy::prelude::{
    App, IntoSystemAppConfig, IntoSystemConfig, OnExit, OnUpdate, Plugin, Query, States, With,
};

use crate::{combat::Delay, physics::MoveLock, units::Player};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app
            // use `AppState` to define current game status, and we can use this when we want to run system logic only once on game loading. ex: `add_system(ui::build_ui.in_schedule(OnEnter(state::AppState::MainGame)))`
            .add_state::<AppState>()
            .add_system(move_lock_system.in_schedule(OnExit(AppState::MainGame)))
            .add_system(move_unlock_system.in_set(OnUpdate(AppState::MainGame)));
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    MainGame,
    ConsoleOpenedState,
    GamePausedState,
    MainMenu,
    ControlMenu,
}

// TODO: Code need to be modified when multiplayer feature is implemented.
pub fn move_lock_system(mut query: Query<&mut MoveLock, With<Player>>) {
    for mut move_lock in query.iter_mut() {
        move_lock.0 = true;
    }
}

pub fn move_unlock_system(mut query: Query<(&mut MoveLock, &Delay)>) {
    for (mut move_lock, delay) in query.iter_mut() {
        if delay.0.finished() && move_lock.0 {
            move_lock.0 = false;
        }
    }
}
