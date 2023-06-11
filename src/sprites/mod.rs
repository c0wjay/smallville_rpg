use bevy::prelude::{App, IntoSystemConfig, Plugin};

pub mod arm;
pub mod base;
pub mod weapon;
pub use arm::*;
pub use base::*;
pub use weapon::*;

use crate::units;

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            // .add_system(systems::sprite_size)
            .add_system(animate_sprite)
            .add_system(animate_arm_sprites.after(animate_sprite))
            .add_system(animate_weapon_sprites.after(animate_arm_sprites))
            .add_system(spawn_arm_sprite.after(units::setup))
            .add_system(spawn_weapon_sprite.after(spawn_arm_sprite))
            .add_system(y_sort);
    }
}
