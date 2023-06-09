use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::constants::UNIT_SIZE;

// TODO: components should be moduled in part.

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
}

#[derive(Clone, Debug, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub damping: Damping,
    pub velocity: Velocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderMassProperties,
    pub active_events: ActiveEvents,
    pub active_collision_types: ActiveCollisionTypes,
    pub collision_groups: CollisionGroups,
}

impl Default for ColliderBundle {
    fn default() -> Self {
        ColliderBundle {
            collider: Collider::cuboid(UNIT_SIZE, UNIT_SIZE),
            rigid_body: RigidBody::Dynamic,
            damping: Damping {
                linear_damping: 0.0,
                angular_damping: 0.0,
            },
            velocity: Velocity::default(),
            rotation_constraints: LockedAxes::ROTATION_LOCKED,
            gravity_scale: GravityScale(0.),
            friction: Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            density: ColliderMassProperties::default(),
            active_events: ActiveEvents::COLLISION_EVENTS,
            active_collision_types: ActiveCollisionTypes::default()
                | ActiveCollisionTypes::STATIC_STATIC,
            collision_groups: CollisionGroups::default(),
        }
    }
}

impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> ColliderBundle {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;

        match entity_instance.identifier.as_ref() {
            "Player" => ColliderBundle {
                collider: Collider::cuboid(UNIT_SIZE, UNIT_SIZE),
                rigid_body: RigidBody::Dynamic,
                friction: Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                rotation_constraints,
                ..ColliderBundle::default()
            },
            "NPC" => ColliderBundle {
                collider: Collider::cuboid(UNIT_SIZE, UNIT_SIZE),
                rigid_body: RigidBody::Dynamic,
                friction: Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                damping: Damping {
                    linear_damping: 100.0,
                    angular_damping: 0.0,
                },
                rotation_constraints,
                ..ColliderBundle::default()
            },
            _ => ColliderBundle::default(),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct NPC;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet_bundle("char/player_base.png", 16.0, 32.0, 6, 21, 0.0, 0.0, 0)]
    #[bundle]
    // TODO: move SpriteSheetBundle to child entity, to order body and other sprites.
    pub sprite_sheet_bundle: SpriteSheetBundle,
    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,
    pub player: Player,
    #[worldly]
    pub worldly: Worldly,
    pub unit_size: UnitSize,
    #[bundle]
    pub animation_bundle: AnimationBundle,
    pub ysort: YSort,
    pub delay: Delay,
    pub move_lock: MoveLock,
    pub coordinate: Coordinate,
    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct NPCBundle {
    #[sprite_sheet_bundle("char/npc_1.png", 16.0, 32.0, 4, 4, 0.0, 0.0, 0)]
    #[bundle]
    // TODO: move SpriteSheetBundle to child entity, to order body and other sprites.
    pub sprite_sheet_bundle: SpriteSheetBundle,
    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,
    pub npc: NPC,
    #[worldly]
    pub worldly: Worldly,
    pub unit_size: UnitSize,
    #[bundle]
    pub animation_bundle: AnimationBundle,
    pub ysort: YSort,
    pub delay: Delay,
    pub hurtbox: Hurtbox,
    pub coordinate: Coordinate,
    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

#[derive(Clone, Default, Component)]
pub struct UnitSize {
    // width and height is half of the actual size.
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Default, Bundle)]
pub struct AnimationBundle {
    pub animation_indices: AnimationIndices,
    pub facing: Facing,
}

#[derive(Component, Default, Clone)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
    pub current: usize,
    pub animation_state: AnimationState,
}

#[derive(Component, Default, Clone, PartialEq, Eq)]
pub enum AnimationState {
    #[default]
    Idle,
    Walk,
    Attack,
    BeHit,
}

#[derive(Component, Default, Clone, Reflect, Debug)]
pub struct Facing {
    pub direction: FaceDirection,
}

#[derive(Component, Default, Clone, PartialEq, Eq, Reflect, Debug)]
pub enum FaceDirection {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, Default, Component)]
pub struct Body;

#[derive(Copy, Clone, Debug, Default, Component)]
pub struct Arm;

#[derive(Copy, Clone, Debug, Default, Component)]
pub struct Weapon;

#[derive(Resource, Deref, DerefMut, Clone)]
pub struct AnimationTimer(pub Timer);

impl Default for AnimationTimer {
    fn default() -> Self {
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}

#[derive(Component, Deref, DerefMut, Clone, Reflect)]
pub struct Delay(pub Timer);

impl Default for Delay {
    fn default() -> Self {
        Delay(Timer::from_seconds(0.5, TimerMode::Once))
    }
}

#[derive(Component, Default, Clone, Reflect, Debug)]

pub struct MoveLock(pub bool);

#[derive(Clone, Default, Debug, Component, Reflect)]
pub struct Coordinate {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

#[derive(Component, Default, Clone, Debug)]
pub struct YSort {
    pub z: f32,
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

#[derive(Debug)]
pub struct MovementEvent {
    pub moved_entity: Entity,
}

#[derive(Component, Default, Clone)]
pub struct Hurtbox;

/// Empty struct simply for grouping collision layer constants.
#[derive(Copy, Clone)]
pub struct BodyLayers;

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
