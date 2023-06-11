use bevy::{
    prelude::{Added, App, Bundle, Component, Entity, Name, Or, Plugin, Query},
    sprite::SpriteSheetBundle,
};
use bevy_ecs_ldtk::{
    prelude::{LdtkEntity, LdtkEntityAppExt, LdtkFields},
    EntityInstance, Worldly,
};

use crate::{
    combat::{Delay, Hurtbox},
    constants::UNIT_SIZE,
    maps::Coordinate,
    physics::{ColliderBundle, MoveLock},
    sprites::{AnimationBundle, YSort},
};

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_system(systems::dbg_player_items)
            .register_ldtk_entity::<PlayerBundle>("Player")
            .register_ldtk_entity::<NPCBundle>("NPC")
            .add_system(setup);
    }
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet_bundle("char/player_base.png", 16.0, 32.0, 6, 21, 0.0, 0.0, 0)]
    #[bundle]
    // TODO: move SpriteSheetBundle to child entity, to order body and other sprites.
    pub sprite_sheet_bundle: SpriteSheetBundle,
    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,
    #[with(name_from_ldtk_field)]
    pub name: Name,
    pub player: Player,
    pub current_interacting_npc: CurrentInteractingNPC,
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
    #[with(name_from_ldtk_field)]
    pub name: Name,
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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

#[derive(Component, Clone, Default)]
pub struct CurrentInteractingNPC(pub Option<Entity>);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct NPC;

fn name_from_ldtk_field(entity_instance: &EntityInstance) -> Name {
    Name::new(
        entity_instance
            .get_string_field("name")
            .expect("entity should have non-nullable name string field")
            .clone(),
    )
}

pub fn setup(mut query: Query<(&mut UnitSize, &mut YSort), Or<(Added<Player>, Added<NPC>)>>) {
    for (mut unit_size, mut ysort) in &mut query {
        // TODO: This is hard-coded for now. unit_size can be differ for each entity.
        unit_size.width = UNIT_SIZE;
        unit_size.height = UNIT_SIZE;
        ysort.z = 5.0;
    }
}
