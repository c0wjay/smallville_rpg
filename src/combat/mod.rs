use bevy::prelude::{App, Plugin};

pub mod base;
pub mod melee;
pub mod projectile;
pub use base::*;
pub use melee::*;
pub use projectile::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_system(melee_attack)
            .add_system(melee_attack_system)
            .add_system(collect_hit)
            .add_system(deactivate_attack);
    }
}
