// src/player/mod.rs
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::items::ItemStack;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 100.0, // Default speed set to 100.0
        }
    }
}

#[derive(Resource)]
pub struct CollisionMap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<bool>,
}

impl CollisionMap {
    pub fn get(&self, pos: IVec2) -> Option<bool> {
        if pos.x >= 0 && pos.x < self.width as i32 && pos.y >= 0 && pos.y < self.height as i32 {
            Some(self.data[(pos.y * self.width as i32 + pos.x) as usize])
        } else {
            None
        }
    }
}

pub fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&Player, &mut Transform)>, // Player query with mutable Transform
    tilemap_query: Query<(&TilemapSize, &TilemapTileSize, &Transform), Without<Player>>, // Tilemap query excluding Player
) {
    let (_player, mut player_transform) = player_query.single_mut(); // Get the player's Transform
    let (map_size, tile_size, tilemap_transform) = tilemap_query.single(); // Get the tilemap's Transform
    let scale = tilemap_transform.scale.x; // e.g., 6.0

    // Calculate tilemap world bounds
    let local_width = map_size.x as f32 * tile_size.x; // e.g., 800.0
    let local_height = map_size.y as f32 * tile_size.y; // e.g., 800.0
    let min_x = tilemap_transform.translation.x;
    let max_x = (tilemap_transform.translation.x + local_width * scale).floor() - 32.0;
    let min_y = tilemap_transform.translation.y;
    let max_y = (tilemap_transform.translation.y + local_height * scale).floor() - 10.0;

    // Calculate new position (example movement logic)
    let speed = 1.0;
    let mut new_x = player_transform.translation.x;
    let mut new_y = player_transform.translation.y;
    if keyboard_input.pressed(KeyCode::KeyW) {
        new_y += speed;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        new_y -= speed;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        new_x += speed;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        new_x -= speed;
    }

    // Clamp to tilemap bounds
    new_x = new_x.clamp(min_x, max_x);
    new_y = new_y.clamp(min_y, max_y);

    player_transform.translation.x = new_x;
    player_transform.translation.y = new_y;
}

#[derive(Component)]
pub struct Inventory {
    pub items: Vec<Option<ItemStack>>,
}
