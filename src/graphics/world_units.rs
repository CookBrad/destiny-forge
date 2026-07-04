use bevy::prelude::*;

/// Gameplay and rendering positions use native sprite pixels; apply `PIXEL_SCALE` on transforms.
pub const TILE: f32 = 16.0;
pub const PIXEL_SCALE: f32 = 3.0;

pub const DUNGEON_MOVE_SPEED: f32 = 138.0;
pub const DUNGEON_JUMP_SPEED: f32 = 385.0;
pub const DUNGEON_AIR_JUMP_MULT: f32 = 0.88;
pub const DUNGEON_GRAVITY: f32 = -760.0;
pub const DUNGEON_FLOOR_Y: f32 = 64.0;
pub const INTERACT_DISTANCE: f32 = 20.0;

pub const ENEMY_DISPLAY_SIZE: Vec2 = Vec2::new(TILE, TILE);

pub fn to_world(pixels: Vec2, z: f32) -> Vec3 {
    Vec3::new(pixels.x, pixels.y, z)
}

pub fn enemy_half_extents() -> Vec2 {
    ENEMY_DISPLAY_SIZE * 0.5
}

/// Place a sprite center so its feet sit on `surface_y`.
pub fn center_on_surface(surface_y: f32, sprite_height: f32) -> f32 {
    surface_y + sprite_height * 0.5
}

pub fn scaled_size(size: Vec2) -> Vec2 {
    size * PIXEL_SCALE
}

pub fn scaled_transform(pixels: Vec2, z: f32) -> Transform {
    Transform {
        translation: to_world(pixels, z),
        scale: Vec3::splat(PIXEL_SCALE),
        ..default()
    }
}