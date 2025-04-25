// src/player/mod.rs
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::time::Duration;

use crate::items::ItemStack;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub position: usize,
    frame_timer: Timer,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 3.0,
            position: 1,
            frame_timer: Timer::new(Duration::from_secs_f32(1.0 / (8.0)), TimerMode::Once),
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
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
    mut player_sprite: Query<&mut Sprite, With<Player>>,
    tilemap_query: Query<(&TilemapSize, &TilemapTileSize, &Transform), Without<Player>>,
) {
    let (mut player, mut player_transform) = player_query.single_mut();
    let (map_size, tile_size, tilemap_transform) = tilemap_query.single();
    let scale = tilemap_transform.scale.x;

    // Calculate tilemap world bounds
    let local_width = map_size.x as f32 * tile_size.x;
    let local_height = map_size.y as f32 * tile_size.y;
    let min_x = tilemap_transform.translation.x;
    let max_x = (tilemap_transform.translation.x + local_width * scale).floor() - 32.0;
    let min_y = tilemap_transform.translation.y;
    let max_y = (tilemap_transform.translation.y + local_height * scale).floor() - 10.0;

    // Calculate new position
    let mut new_x = player_transform.translation.x;
    let mut new_y = player_transform.translation.y;
    let last_index = 3;

    if let Ok(mut player_sprite) = player_sprite.get_single_mut() {
        if let Some(ref mut player_sprite_texture_atlas) = player_sprite.texture_atlas {
            if keyboard_input.just_pressed(KeyCode::KeyW) {
                player.position = 8;
                player_sprite_texture_atlas.index = player.position;
            }
            if keyboard_input.just_pressed(KeyCode::KeyA) {
                player.position = 12;
                player_sprite_texture_atlas.index = player.position;
            }
            if keyboard_input.just_pressed(KeyCode::KeyD) {
                player.position = 4;
                player_sprite_texture_atlas.index = player.position;
            }
            if keyboard_input.just_pressed(KeyCode::KeyS) {
                player.position = 0;
                player_sprite_texture_atlas.index = player.position;
            }
            if keyboard_input.pressed(KeyCode::KeyW) {
                new_y += player.speed;
                player.frame_timer.tick(time.delta());
                if player.frame_timer.just_finished() {
                    if player_sprite_texture_atlas.index != last_index + player.position {
                        player_sprite_texture_atlas.index += 1;
                    } else {
                        player_sprite_texture_atlas.index = player.position;
                    }
                    player.frame_timer =
                        Timer::new(Duration::from_secs_f32(1.0 / 8.0), TimerMode::Once);
                }
            }
            if keyboard_input.pressed(KeyCode::KeyS) {
                new_y -= player.speed;
                player.frame_timer.tick(time.delta());
                if player.frame_timer.just_finished() {
                    if player_sprite_texture_atlas.index != last_index + player.position {
                        player_sprite_texture_atlas.index += 1;
                    } else {
                        player_sprite_texture_atlas.index = player.position;
                    }
                    player.frame_timer =
                        Timer::new(Duration::from_secs_f32(1.0 / 8.0), TimerMode::Once);
                }
            }
            if keyboard_input.pressed(KeyCode::KeyD) {
                new_x += player.speed;
                player.frame_timer.tick(time.delta());
                if player.frame_timer.just_finished() {
                    if player_sprite_texture_atlas.index != last_index + player.position {
                        player_sprite_texture_atlas.index += 1;
                    } else {
                        player_sprite_texture_atlas.index = player.position;
                    }
                    player.frame_timer =
                        Timer::new(Duration::from_secs_f32(1.0 / 8.0), TimerMode::Once);
                }
            }
            if keyboard_input.pressed(KeyCode::KeyA) {
                new_x -= player.speed;
                player.frame_timer.tick(time.delta());
                if player.frame_timer.just_finished() {
                    if player_sprite_texture_atlas.index != last_index + player.position {
                        player_sprite_texture_atlas.index += 1;
                    } else {
                        player_sprite_texture_atlas.index = player.position;
                    }
                    player.frame_timer =
                        Timer::new(Duration::from_secs_f32(1.0 / 8.0), TimerMode::Once);
                }
            }
        }
    }

    // Clamp to tilemap bounds
    new_x = new_x.clamp(min_x, max_x);
    new_y = new_y.clamp(min_y, max_y);

    // Update position with z = y
    player_transform.translation = Vec3::new(new_x, new_y, 500.0 - new_y);
}

#[derive(Component)]
pub struct Inventory {
    pub items: Vec<Option<ItemStack>>,
}
