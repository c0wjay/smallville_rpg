// In this game, the player navigates to wherever you click

use bevy::{
    prelude::{
        error, Camera, Camera2d, Commands, Deref, DerefMut, DetectChanges, Entity, EventReader,
        EventWriter, GlobalTransform, Input, MouseButton, Query, Res, ResMut, Resource, UVec2,
        Vec2, With,
    },
    reflect::Reflect,
    window::Window,
};
use seldom_map_nav::prelude::{
    Nav, NavBundle, NavPathMode, NavQuery, Navability, Navmeshes, PathTarget, Pathfind,
};

use crate::{
    constants::{GRID_SIZE, UNIT_SIZE},
    maps::{TileGridMap, TileType},
    units::Player,
};

pub fn setup(
    mut commands: Commands,
    tile_grid_map: Res<TileGridMap>,
    navmesheses: Query<Entity, With<Navmeshes>>,
) {
    if tile_grid_map.is_changed() {
        let max_x = tile_grid_map.max_x;
        let max_y = tile_grid_map.max_y;
        let mut tilemap: Vec<Navability> = Vec::with_capacity(((max_x + 1) * (max_y + 1)) as usize);

        for y in 0..=max_y {
            for x in 0..=max_x {
                if let Some((_, tile)) = tile_grid_map.tile_map.get(&(x, y)) {
                    match tile {
                        TileType::Floor => tilemap.push(Navability::Navable),
                        TileType::Wall => tilemap.push(Navability::Solid),
                    }
                } else {
                    tilemap.push(Navability::Solid);
                }
            }
        }

        let navability = |pos: UVec2| tilemap[(pos.y * (max_x as u32 + 1) + pos.x) as usize];

        // Spawn the tilemap with a `Navmeshes` component
        let navmeshes = Navmeshes::generate(
            UVec2::new(max_x as u32 + 1, max_y as u32 + 1),
            Vec2::new(GRID_SIZE, GRID_SIZE),
            navability,
            [UNIT_SIZE - 0.01],
        );

        // Unit size radius is slightly smaller than half of grid size.
        // This prevent Triangulation Error.
        if let Ok(navmeshes) = navmeshes {
            for entity in navmesheses.iter() {
                commands.entity(entity).despawn();
            }
            commands.spawn(navmeshes);
        } else {
            error!("Navmeshes error: {:?}", navmeshes)
        }
    }
}

// TODO: CursorPos should be moduled into `input.rs`
#[derive(Default, Deref, DerefMut, Resource, Reflect)]
pub struct CursorPos(Option<Vec2>);

pub fn update_cursor_pos(
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    windows: Query<&Window>,
    mut pos: ResMut<CursorPos>,
) {
    let (camera, camera_transform) = camera_query.single();
    let Some(cursor_position) = windows.single().cursor_position() else {return;};
    let cursor_world_position = camera.viewport_to_world_2d(camera_transform, cursor_position);

    **pos = cursor_world_position;
}

// Navigate the player to wherever you click
pub fn move_player_when_mouse_click(
    players: Query<Entity, With<Player>>,
    cursor_pos: Res<CursorPos>,
    mouse: Res<Input<MouseButton>>,
    mut movement_writer: EventWriter<OrderMovementEvent>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        if let Some(cursor_pos) = **cursor_pos {
            // Clicked somewhere on the screen!
            movement_writer.send(OrderMovementEvent {
                entity: players.single(),
                destination: cursor_pos,
            });
        }
    }
}

pub struct OrderMovementEvent {
    pub entity: Entity,
    pub destination: Vec2,
}

pub fn processing_order_movement_event(
    mut commands: Commands,
    mut events: EventReader<OrderMovementEvent>,
    navmesheses: Query<Entity, With<Navmeshes>>,
) {
    for OrderMovementEvent {
        entity,
        destination,
    } in events.iter()
    {
        // Add `NavBundle` to start navigating to that position
        // If you want to write your own movement, but still want paths generated,
        // only insert `Pathfind`.
        commands.entity(*entity).insert(NavBundle {
            pathfind: Pathfind::new(
                navmesheses.single(),
                UNIT_SIZE - 0.01,
                None,
                PathTarget::Static(*destination),
                NavQuery::Accuracy,
                NavPathMode::Accuracy,
            ),
            nav: Nav::new(100.),
        });
    }
}
