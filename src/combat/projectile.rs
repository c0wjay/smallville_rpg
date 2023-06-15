use bevy::prelude::{error, info, Commands, Entity, EventReader, EventWriter, Parent, Query, With};
use bevy_rapier2d::prelude::CollisionEvent;

use crate::combat::Attack;

use super::{DamageEvent, Hurtbox};

// Previous version of melee_attack_system. But using CollisionEvent is better for projectiles.
// TODO: reuse this code for projectile attacks
#[allow(dead_code)]
pub fn projectile_attack_system(
    mut _commands: Commands,
    mut events: EventReader<CollisionEvent>,
    attacks: Query<(&Parent, Entity, &Attack)>,
    _hurtboxes: Query<&Parent, With<Hurtbox>>,
    mut event_writer: EventWriter<DamageEvent>,
) {
    for event in events.iter() {
        if let CollisionEvent::Started(e1, e2, _flags) = event {
            for (attacker, _, attack) in attacks.iter() {
                let (attacker_entity, hurtbox_entity) = if attacker.get() == *e1 {
                    (*e1, *e2)
                } else if attacker.get() == *e2 {
                    (*e2, *e1)
                } else {
                    continue;
                };
                info!("Attacker: {:?}, Attack: {:?}", attacker_entity, attack);

                if attacker_entity.eq(e1) | attacker_entity.eq(e2) {
                    event_writer.send(DamageEvent {
                        damageing_entity: attacker_entity,
                        damage_velocity: attack.pushback,
                        damage: attack.damage,
                        damaged_entity: hurtbox_entity,
                        hitstun_duration: attack.hitstun_duration,
                    });
                    break;
                }
                error!("message should not be shown");
            }
        }
    }
}
