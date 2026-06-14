use bevy::prelude::*;

use super::atlas::{DungeonSprite, GameSprites, HubTile};
use super::spawn::{dungeon_tile_sprite, hub_tile_sprite, sprite_transform, tile_world_size};

pub fn spawn_hub_grass_field<M: Bundle + Copy>(
    commands: &mut Commands,
    sprites: &GameSprites,
    center: Vec2,
    columns: i32,
    rows: i32,
    marker: M,
) {
    let tile = tile_world_size();
    let origin_x = center.x - (columns as f32 - 1.0) * tile.x * 0.5;
    let origin_y = center.y - (rows as f32 - 1.0) * tile.y * 0.5;

    for row in 0..rows {
        for col in 0..columns {
            let tile_type = HubTile::grass_variant(col, row);
            let position = Vec3::new(
                origin_x + col as f32 * tile.x,
                origin_y + row as f32 * tile.y,
                0.0,
            );

            commands.spawn((
                hub_tile_sprite(sprites, tile_type),
                sprite_transform(position),
                marker,
            ));
        }
    }
}

pub fn spawn_dungeon_tile_floor<M: Bundle + Copy>(
    commands: &mut Commands,
    sprites: &GameSprites,
    center: Vec2,
    columns: i32,
    rows: i32,
    tile: DungeonSprite,
    marker: M,
) {
    let tile_size = tile_world_size();
    let origin_x = center.x - (columns as f32 - 1.0) * tile_size.x * 0.5;
    let origin_y = center.y - (rows as f32 - 1.0) * tile_size.y * 0.5;

    for row in 0..rows {
        for col in 0..columns {
            let tile_type = match tile {
                DungeonSprite::CaveFloorA | DungeonSprite::CaveFloorB => {
                    DungeonSprite::cave_floor_variant(col)
                }
                other => other,
            };

            let position = Vec3::new(
                origin_x + col as f32 * tile_size.x,
                origin_y + row as f32 * tile_size.y,
                0.0,
            );

            commands.spawn((
                dungeon_tile_sprite(sprites, tile_type),
                sprite_transform(position),
                marker,
            ));
        }
    }
}

pub fn spawn_dungeon_platform_tiles(
    commands: &mut Commands,
    sprites: &GameSprites,
    center: Vec2,
    world_size: Vec2,
) {
    let tile_size = tile_world_size();
    let columns = (world_size.x / tile_size.x).ceil() as i32;
    let rows = (world_size.y / tile_size.y).ceil().max(1.0) as i32;
    let origin_x = center.x - world_size.x * 0.5 + tile_size.x * 0.5;
    let origin_y = center.y - world_size.y * 0.5 + tile_size.y * 0.5;

    for row in 0..rows {
        for col in 0..columns {
            let position = Vec3::new(
                origin_x + col as f32 * tile_size.x,
                origin_y + row as f32 * tile_size.y,
                1.0,
            );
            commands.spawn((
                dungeon_tile_sprite(sprites, DungeonSprite::StonePlatform),
                sprite_transform(position),
            ));
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct PlatformBounds {
    pub half_size: Vec2,
}

pub fn spawn_decorative_hub_props<M: Bundle + Copy>(
    commands: &mut Commands,
    sprites: &GameSprites,
    marker: M,
) {
    let props = [
        (HubTile::Bush, Vec2::new(-220.0, 40.0)),
        (HubTile::Rock, Vec2::new(220.0, -30.0)),
        (HubTile::Bush, Vec2::new(60.0, 80.0)),
        (HubTile::Rock, Vec2::new(-80.0, -60.0)),
    ];

    for (tile, position) in props {
        commands.spawn((
            hub_tile_sprite(sprites, tile),
            sprite_transform(position.extend(4.0)),
            marker,
        ));
    }
}