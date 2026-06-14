use bevy::prelude::*;

use super::atlas::{DungeonSprite, GameSprites, HubTile, PLAYER_FRAME_HEIGHT, PLAYER_FRAME_WIDTH};

pub const PIXEL_SCALE: f32 = 3.0;

pub fn pixel_sprite(sprites: &GameSprites, sheet: PixelSheet, atlas_index: usize) -> Sprite {
    let (image, layout) = match sheet {
        PixelSheet::Hub => (sprites.hub_tiles.clone(), sprites.hub_tiles_layout.clone()),
        PixelSheet::Dungeon => (
            sprites.dungeon_sheet.clone(),
            sprites.dungeon_layout.clone(),
        ),
        PixelSheet::Player => (sprites.player.clone(), sprites.player_layout.clone()),
    };

    Sprite {
        image,
        texture_atlas: Some(TextureAtlas {
            layout,
            index: atlas_index,
        }),
        ..default()
    }
}

pub enum PixelSheet {
    Hub,
    Dungeon,
    Player,
}

pub fn image_sprite(image: Handle<Image>, pixel_size: Vec2) -> Sprite {
    Sprite {
        image,
        custom_size: Some(pixel_size),
        ..default()
    }
}

pub fn sprite_transform(position: Vec3) -> Transform {
    Transform::from_translation(position).with_scale(Vec3::splat(PIXEL_SCALE))
}

pub fn hub_tile_sprite(sprites: &GameSprites, tile: HubTile) -> Sprite {
    pixel_sprite(sprites, PixelSheet::Hub, tile.atlas_index())
}

pub fn dungeon_tile_sprite(sprites: &GameSprites, tile: DungeonSprite) -> Sprite {
    pixel_sprite(sprites, PixelSheet::Dungeon, tile.atlas_index())
}

pub fn player_frame_size() -> Vec2 {
    Vec2::new(PLAYER_FRAME_WIDTH as f32, PLAYER_FRAME_HEIGHT as f32)
}

pub fn player_world_size() -> Vec2 {
    player_frame_size() * PIXEL_SCALE
}

pub fn tile_world_size() -> Vec2 {
    Vec2::splat(16.0 * PIXEL_SCALE)
}