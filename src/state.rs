use crate::components::{Delay, MoveLock, Player};
use bevy::prelude::{info, Entity, Query, States, With};

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
pub fn move_lock_system(mut query: Query<(&mut MoveLock, Entity)>) {
    info!("move_lock_system");
    for (mut move_lock, player) in query.iter_mut() {
        info!("move_lock_system inner");
        move_lock.0 = true;
        info!("{:?} {:?}", player, move_lock);
    }
}

pub fn move_unlock_system(mut query: Query<(&mut MoveLock, &Delay), With<Player>>) {
    for (mut move_lock, delay) in query.iter_mut() {
        if delay.0.finished() && move_lock.0 {
            move_lock.0 = false;
        }
    }
}
