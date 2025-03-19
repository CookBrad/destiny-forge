// src/player/mod.rs
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

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

#[derive(Component)]
pub struct Inventory {
    pub items: Vec<Option<String>>,
}

impl Inventory {
    pub fn new(size: usize) -> Self {
        Self {
            items: vec![None; size],
        }
    }

    pub fn add_item(&mut self, item: String) -> bool {
        if let Some(slot) = self.items.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(item);
            true
        } else {
            false
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
    mut player_query: Query<(&mut Transform, &Player)>,
    collision_map: Res<CollisionMap>,
    tilemap_query: Query<&TilemapGridSize>,
) {
    let grid_size = tilemap_query.single();
    let (mut player_transform, player) = player_query.single_mut();

    let mut velocity = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        velocity.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        velocity.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        velocity.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        velocity.x += 1.0;
    }
    if velocity.length() > 0.0 {
        velocity = velocity.normalize() * player.speed;

        let dt = time.delta_secs();
        let new_position =
            player_transform.translation + Vec3::new(velocity.x * dt, velocity.y * dt, 0.0);

        let tile_pos = IVec2::new(
            (new_position.x / grid_size.x).floor() as i32,
            (new_position.y / grid_size.y).floor() as i32,
        );

        if let Some(is_walkable) = collision_map.get(tile_pos) {
            if is_walkable {
                player_transform.translation = new_position;
            }
        }
    }
}
