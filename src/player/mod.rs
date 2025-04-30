use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::time::Duration;

use crate::items::ItemStack;
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub position: usize,
    pub direction: Direction,
    last_pressed: KeyCode,
    frame_timer: Timer,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 3.0,
            position: 1,
            direction: Direction::Down,
            last_pressed: KeyCode::KeyS,
            frame_timer: Timer::new(Duration::from_secs_f32(1.0 / 8.0), TimerMode::Once),
        }
    }
}

fn update_sprite(
    player_sprite: &mut Query<&mut Sprite, With<Player>>,
    position: usize,
) -> Option<()> {
    let mut sprite = player_sprite.get_single_mut().ok()?;
    let texture_atlas = sprite.texture_atlas.as_mut()?;
    texture_atlas.index = position;
    Some(())
}

fn handle_direction_change(
    player: &mut Player,
    player_sprite: &mut Query<&mut Sprite, With<Player>>,
    direction: Direction,
    position: usize,
) {
    player.direction = direction;
    player.position = position;
    update_sprite(player_sprite, position);
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

    // Define direction mappings: (KeyCode, Direction, position)
    let direction_mappings = [
        (KeyCode::KeyW, Direction::Up, 8),
        (KeyCode::KeyA, Direction::Left, 12),
        (KeyCode::KeyS, Direction::Down, 0),
        (KeyCode::KeyD, Direction::Right, 4),
    ];

    let all_direction_keys = [KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD];
    let pressed_directions: Vec<KeyCode> = all_direction_keys
        .iter()
        .filter(|&&key| keyboard_input.pressed(key))
        .cloned()
        .collect();

    // Handle just_pressed logic
    for &(key_code, direction, position) in &direction_mappings {
        if keyboard_input.just_pressed(key_code) && pressed_directions.len() == 1 {
            handle_direction_change(&mut player, &mut player_sprite, direction, position);
        }
    }

    // Set last_pressed when a direction key is just pressed
    for &(key_code, _, _) in &direction_mappings {
        if keyboard_input.just_pressed(key_code) {
            player.last_pressed = key_code;
        }
    }

    // Handle just_released logic
    for &(key_code, _, _) in &direction_mappings {
        if keyboard_input.just_released(key_code) && player.last_pressed != key_code {
            let last_mapping = direction_mappings
                .iter()
                .find(|&&mapping| mapping.0 == player.last_pressed)
                .unwrap();
            let (_, direction, position) = *last_mapping;
            handle_direction_change(&mut player, &mut player_sprite, direction, position);
        }
    }

    // Early return if opposite directions are pressed
    if keyboard_input.pressed(KeyCode::KeyA) && keyboard_input.pressed(KeyCode::KeyD) {
        return;
    }
    if keyboard_input.pressed(KeyCode::KeyW) && keyboard_input.pressed(KeyCode::KeyS) {
        return;
    }

    // Movement logic
    let mut new_x = player_transform.translation.x;
    let mut new_y = player_transform.translation.y;
    let mut should_animate = false;
    let last_index = 3;

    if keyboard_input.pressed(KeyCode::KeyW) && player.direction != Direction::Down {
        new_y += player.speed;
        should_animate = true;
    }
    if keyboard_input.pressed(KeyCode::KeyS) && player.direction != Direction::Up {
        new_y -= player.speed;
        should_animate = true;
    }
    if keyboard_input.pressed(KeyCode::KeyD) && player.direction != Direction::Left {
        new_x += player.speed;
        should_animate = true;
    }
    if keyboard_input.pressed(KeyCode::KeyA) && player.direction != Direction::Right {
        new_x -= player.speed;
        should_animate = true;
    }

    // Animation update
    if should_animate {
        player.frame_timer.tick(time.delta());
        if player.frame_timer.just_finished() {
            if let Ok(mut sprite) = player_sprite.get_single_mut() {
                if let Some(ref mut texture_atlas) = sprite.texture_atlas {
                    if texture_atlas.index != player.position + last_index {
                        texture_atlas.index += 1;
                    } else {
                        texture_atlas.index = player.position;
                    }
                }
            }
            player.frame_timer = Timer::new(Duration::from_secs_f32(1.0 / 8.0), TimerMode::Once);
        }
    }

    // Clamp to tilemap bounds
    let local_width = map_size.x as f32 * tile_size.x;
    let local_height = map_size.y as f32 * tile_size.y;
    let min_x = tilemap_transform.translation.x;
    let max_x = (tilemap_transform.translation.x + local_width * scale).floor() - 32.0;
    let min_y = tilemap_transform.translation.y;
    let max_y = (tilemap_transform.translation.y + local_height * scale).floor() - 10.0;
    new_x = new_x.clamp(min_x, max_x);
    new_y = new_y.clamp(min_y, max_y);

    // Update position with z = y
    player_transform.translation = Vec3::new(new_x, new_y, 500.0 - new_y);
}

#[derive(Component)]
pub struct Inventory {
    pub items: Vec<Option<ItemStack>>,
}
