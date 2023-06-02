use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use bevy_rapier2d::prelude::*;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub damping: Damping,
    pub velocity: Velocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderMassProperties,
}

impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> ColliderBundle {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;

        match entity_instance.identifier.as_ref() {
            "Player" => ColliderBundle {
                collider: Collider::cuboid(4., 4.),
                rigid_body: RigidBody::Dynamic,
                friction: Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                rotation_constraints,
                ..Default::default()
            },
            "NPC" => ColliderBundle {
                collider: Collider::cuboid(4., 4.),
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
                ..Default::default()
            },
            _ => ColliderBundle::default(),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet_bundle("Tiny16-ExpandedMaleSprites.png", 16.0, 16.0, 6, 4, 0.0, 0.0, 0)]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,
    pub player: Player,
    #[worldly]
    pub worldly: Worldly,

    pub animation_indices: AnimationIndices,
    pub ysort: YSort,
    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct NPC;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct NPCBundle {
    #[sprite_sheet_bundle("Tiny16-ExpandedFemaleSprites.png", 16.0, 16.0, 6, 4, 0.0, 0.0, 0)]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,
    pub npc: NPC,
    #[worldly]
    pub worldly: Worldly,

    pub animation_indices: AnimationIndices,
    pub ysort: YSort,
    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
}

#[derive(Component, Default, Clone)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
    pub animation_state: AnimationState,
}

#[derive(Component, Default, Clone, PartialEq, Eq)]
pub enum AnimationState {
    #[default]
    Idle,
    WalkDown,
    WalkLeft,
    WalkRight,
    WalkUp,
}

#[derive(Resource, Deref, DerefMut, Default, Clone)]
pub struct AnimationTimer(pub Timer);

#[derive(Component, Default, Clone, Debug)]
pub struct YSort {
    pub z: f32,
}
