use bevy::prelude::*;

use crate::combat::PlayerAttack;
use crate::graphics::{
    DUNGEON_FLOOR_Y, DUNGEON_GRAVITY, DUNGEON_JUMP_SPEED, DUNGEON_MOVE_SPEED, TILE,
};

use super::sprites::player_half_extents;

use super::setup::PlatformCollider;

#[derive(Component)]
pub struct DungeonPlayer;

#[derive(Component, Default)]
pub struct PlayerVelocity {
    pub x: f32,
    pub y: f32,
    pub grounded: bool,
}

pub fn dungeon_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    platforms: Query<&PlatformCollider>,
    mut player: Query<(&mut Transform, &mut PlayerVelocity, &PlayerAttack), With<DungeonPlayer>>,
) {
    let Ok((mut transform, mut velocity, attack)) = player.get_single_mut() else {
        return;
    };

    let mut move_input = 0.0;
    if !attack.is_active() {
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            move_input -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            move_input += 1.0;
        }
    }

    velocity.x = move_input * DUNGEON_MOVE_SPEED;

    if velocity.grounded && !attack.is_active() && keyboard.just_pressed(KeyCode::Space) {
        velocity.y = DUNGEON_JUMP_SPEED;
        velocity.grounded = false;
    }

    if !velocity.grounded {
        velocity.y += DUNGEON_GRAVITY * time.delta_secs();
    }

    let half = player_half_extents();
    let delta = Vec2::new(velocity.x, velocity.y) * time.delta_secs();
    let mut position = Vec2::new(transform.translation.x, transform.translation.y);

    position.x += delta.x;
    position.x = position.x.max(half.x);

    position.y += delta.y;
    velocity.grounded = false;

    let feet_y = position.y - half.y;
    if velocity.y <= 0.0 {
        for collider in &platforms {
            if feet_was_above(collider.top_y, feet_y - delta.y)
                && feet_y <= collider.top_y
                && overlaps_x(position.x, half.x, collider)
            {
                position.y = collider.top_y + half.y;
                velocity.y = 0.0;
                velocity.grounded = true;
                break;
            }
        }
    }

    if feet_y < -4.0 * TILE {
        velocity.y = DUNGEON_JUMP_SPEED * 0.5;
        position.y = DUNGEON_FLOOR_Y + half.y + TILE;
        velocity.grounded = false;
    }

    transform.translation.x = position.x;
    transform.translation.y = position.y;
}

fn feet_was_above(platform_top: f32, previous_feet: f32) -> bool {
    previous_feet >= platform_top - 0.5
}

fn overlaps_x(center_x: f32, half_width: f32, collider: &PlatformCollider) -> bool {
    let left = center_x - half_width;
    let right = center_x + half_width;
    right > collider.min_x && left < collider.max_x
}