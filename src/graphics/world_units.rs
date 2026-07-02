use bevy::prelude::*;

/// Gameplay and rendering positions use native sprite pixels; apply `PIXEL_SCALE` on transforms.
pub const TILE: f32 = 16.0;
pub const PIXEL_SCALE: f32 = 3.0;

pub const DUNGEON_MOVE_SPEED: f32 = 90.0;
pub const DUNGEON_JUMP_SPEED: f32 = 260.0;
pub const DUNGEON_GRAVITY: f32 = -720.0;
pub const DUNGEON_FLOOR_Y: f32 = 64.0;
pub const INTERACT_DISTANCE: f32 = 20.0;

/// Native sprite size in logical pixels. Rendering multiplies by `PIXEL_SCALE` on the transform only.
pub const PLAYER_DISPLAY_SIZE: Vec2 = Vec2::new(16.0, 28.0);
pub const ENEMY_DISPLAY_SIZE: Vec2 = Vec2::new(TILE, TILE);

pub fn to_world(pixels: Vec2, z: f32) -> Vec3 {
    Vec3::new(pixels.x, pixels.y, z)
}

pub fn player_display_size() -> Vec2 {
    PLAYER_DISPLAY_SIZE
}

pub fn player_half_extents() -> Vec2 {
    player_display_size() * 0.5
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