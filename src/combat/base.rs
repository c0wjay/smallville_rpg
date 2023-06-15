use bevy::{
    prelude::{
        info, warn, Commands, Component, Deref, DerefMut, DespawnRecursiveExt, Entity, EventReader,
        Or, Parent, Query, ReflectComponent, Res, Vec2, Visibility, With,
    },
    reflect::Reflect,
    time::{Time, Timer, TimerMode},
};
use bevy_rapier2d::prelude::Group;

use crate::units::{Player, NPC};

#[derive(Component, Deref, DerefMut, Clone, Reflect)]
pub struct Delay(pub Timer);

impl Default for Delay {
    fn default() -> Self {
        Delay(Timer::from_seconds(0., TimerMode::Once))
    }
}

/// A component representing an attack that can do damage to [`Damageable`]s with [`Health`].
#[derive(Component, Clone, Copy, Default, Reflect, Debug)]
#[reflect(Component)]
pub struct Attack {
    //maybe just replace all fields with AttackMeta
    pub damage: i32,
    /// The direction and speed that the attack is hitting something in.
    pub pushback: Vec2,
    pub hitstun_duration: f32,
    // add this for attacks that are not immediately active, used in activate_hitbox
    // pub hitbox_meta: Option<ColliderMeta>,
}

#[derive(Debug)]
pub struct DamageEvent {
    pub damage_velocity: Vec2,
    pub damageing_entity: Entity,
    pub damaged_entity: Entity,
    pub damage: i32,
    pub hitstun_duration: f32,
}

#[derive(Component, Default, Clone)]
pub struct Hurtbox;

/// Empty struct simply for grouping collision layer constants.
#[derive(Copy, Clone)]
pub struct BodyLayers;

#[allow(dead_code)]
impl BodyLayers {
    // Each successive layer represents a different bit in the 32-bit u32 type.
    //
    // The layer is represented by 1 shifted 0 places to the left:          0b0001.
    // The second layer is represented by 1 shifted one place to the left:  0b0010.
    // And so on for the rest of the layers.
    pub const ENEMY: Group = Group::GROUP_1;
    pub const PLAYER: Group = Group::GROUP_2;
    pub const PLAYER_ATTACK: Group = Group::GROUP_3;
    pub const ENEMY_ATTACK: Group = Group::GROUP_4;
    pub const BREAKABLE_ITEM: Group = Group::GROUP_5;
    // u32::MAX is a u32 with all of it's bits set to 1, so this will contain all of the layers.
    pub const ALL: Group = Group::ALL;
}

pub fn collect_hit(
    mut npc: Query<&mut Visibility, Or<(With<NPC>, With<Player>)>>,
    mut damage_events: EventReader<DamageEvent>,
) {
    for event in damage_events.iter() {
        if let Ok(mut visibility) = npc.get_mut(event.damaged_entity) {
            info!("NPC {:?} hit {:?}", event.damaged_entity, visibility);
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn deactivate_attack(
    mut commands: Commands,
    attacks: Query<(&Parent, Entity), With<Attack>>,
    mut player: Query<&mut Delay, With<Player>>,
    time: Res<Time>,
) {
    for (parent, entity) in attacks.iter() {
        let delay = player.get_mut(parent.get());
        if !delay.is_err() {
            let delay = &mut delay.unwrap().0;
            delay.tick(time.delta());
            if delay.finished() {
                warn!("Attack {:?} demolished", entity);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
