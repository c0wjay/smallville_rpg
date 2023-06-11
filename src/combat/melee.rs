use bevy::{
    prelude::{
        info, BuildChildren, Commands, Entity, EventWriter, Input, KeyCode, Parent, Query, Res,
        Vec2, With,
    },
    time::{Time, Timer, TimerMode},
};
use bevy_rapier2d::prelude::{CollisionGroups, Velocity};

use crate::{
    combat::{Attack, BodyLayers},
    maps::{Coordinate, EntityGridMap},
    physics::MoveLock,
    sprites::{AnimationIndices, AnimationState, FaceDirection, Facing},
    units::Player,
};

use super::{DamageEvent, Delay, Hurtbox};

pub fn melee_attack(
    input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut fighters: Query<
        (
            &mut AnimationIndices,
            &mut MoveLock,
            &Facing,
            &mut Velocity,
            &mut Delay,
            Entity,
        ),
        With<Player>,
    >,
    time: Res<Time>,
) {
    for (mut indices, mut move_lock, _facing, mut _velocity, mut delay, entity) in &mut fighters {
        delay.tick(time.delta());
        if input.pressed(KeyCode::Space) && !move_lock.0 {
            // const MOVE_FRONT: f32 = 100.;
            // match facing.direction {
            //     FaceDirection::Down => velocity.linvel.y = -MOVE_FRONT,
            //     FaceDirection::Left => velocity.linvel.x = -MOVE_FRONT,
            //     FaceDirection::Right => velocity.linvel.x = MOVE_FRONT,
            //     FaceDirection::Up => velocity.linvel.y = MOVE_FRONT,
            // }

            indices.animation_state = AnimationState::Attack;

            // Spawn the attack entity
            let attack_entity = commands
                .spawn(CollisionGroups::new(
                    BodyLayers::PLAYER_ATTACK,
                    BodyLayers::ENEMY,
                ))
                .insert(Attack {
                    damage: 1,
                    pushback: Vec2::ZERO,
                    hitstun_duration: 1.,
                })
                .id();
            *delay = Delay(Timer::from_seconds(1., TimerMode::Once));
            commands.entity(entity).push_children(&[attack_entity]);

            move_lock.0 = true;
        }
    }
}

pub fn melee_attack_system(
    entity_map: Res<EntityGridMap>,
    attacks: Query<(&Parent, &Attack)>,
    attackers: Query<(&Facing, &Coordinate)>,
    _hurtboxes: Query<&Parent, With<Hurtbox>>,
    mut event_writer: EventWriter<DamageEvent>,
) {
    for (attacker, attack) in attacks.iter() {
        let attacker_entity = attacker.get();

        // TODO: under code should be separate systems. Maybe `fn find_entities_in_range`, and add entities in player range into player entitiy's children entities.
        // This always check player's range at each frame and update children entities. This system can enable to emphasize entities nearby.
        let (attacker_facing, attacker_coordinate) = attackers
            .get(attacker_entity)
            .expect("Attacker entity must have `Facing` and `Coordinate` components");

        let (range_x, range_y) = match attacker_facing.direction {
            FaceDirection::Down => (
                (attacker_coordinate.min_x..=attacker_coordinate.max_x),
                (attacker_coordinate.min_y - 1..=attacker_coordinate.min_y - 1), // Maybe have to range in min_y-1..=min_y, because of very small & adjoined objects
            ),
            FaceDirection::Left => (
                (attacker_coordinate.min_x - 1..=attacker_coordinate.min_x - 1),
                (attacker_coordinate.min_y..=attacker_coordinate.max_y),
            ),
            FaceDirection::Right => (
                (attacker_coordinate.max_x + 1..=attacker_coordinate.max_x + 1),
                (attacker_coordinate.min_y..=attacker_coordinate.max_y),
            ),
            FaceDirection::Up => (
                (attacker_coordinate.min_x..=attacker_coordinate.max_x),
                (attacker_coordinate.max_y + 1..=attacker_coordinate.max_y + 1),
            ),
        };

        let mut hurtbox_vec = Vec::new();
        for x in range_x {
            for y in range_y.clone() {
                if let Some(hit_range) = entity_map.get((x, y)) {
                    for hurtbox_entity in hit_range {
                        if attacker_entity.eq(hurtbox_entity)
                            | hurtbox_vec.contains(&hurtbox_entity)
                        {
                            continue;
                        }
                        hurtbox_vec.push(hurtbox_entity);
                    }
                }
            }
        }

        info!("Attacker: {:?}, Attack: {:?}", attacker_entity, attack);

        for hurtbox_entity in hurtbox_vec {
            event_writer.send(DamageEvent {
                damageing_entity: attacker_entity,
                damage_velocity: attack.pushback,
                damage: attack.damage,
                damaged_entity: *hurtbox_entity,
                hitstun_duration: attack.hitstun_duration,
            });
        }
    }
}
