use bevy::prelude::{App, IntoSystemConfig, IntoSystemConfigs, Plugin};

pub mod arm;
pub mod base;
pub mod movement_detection;
pub mod weapon;
pub use arm::*;
pub use base::*;
pub use movement_detection::*;
pub use weapon::*;

use crate::units;

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_system(systems::sprite_size)
            .add_system(animate_player_sprite)
            .add_system(animate_npc_sprite)
            .add_system(animate_arm_sprites.after(animate_player_sprite))
            .add_system(animate_weapon_sprites.after(animate_arm_sprites))
            .add_system(spawn_arm_sprite.after(units::setup))
            .add_system(spawn_weapon_sprite.after(spawn_arm_sprite))
            .add_systems(
                (
                    change_animation_state_when_move,
                    change_animation_state_with_navigation,
                    set_to_idle_when_stop,
                    change_animation_indices_with_state,
                )
                    .chain(),
            )
            .add_system(y_sort);
    }
}
