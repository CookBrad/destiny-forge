use bevy::prelude::*;

use crate::combat::{PlayerAttack, PlayerBlock, PlayerKnockback};
use crate::graphics::{
    DungeonScrollBounds, DUNGEON_FLOOR_Y, DUNGEON_GRAVITY, DUNGEON_JUMP_SPEED, DUNGEON_MOVE_SPEED,
    TILE,
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

const PLAYER_KNOCKBACK_DECAY: f32 = 7.0;
const PLAYER_KNOCKBACK_STOP: f32 = 22.0;

pub fn dungeon_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    bounds: Res<DungeonScrollBounds>,
    platforms: Query<&PlatformCollider>,
    mut player: Query<
        (
            Entity,
            &mut Transform,
            &mut PlayerVelocity,
            &PlayerAttack,
            &PlayerBlock,
            Option<&mut PlayerKnockback>,
        ),
        With<DungeonPlayer>,
    >,
) {
    let Ok((entity, mut transform, mut velocity, attack, block, mut knockback)) =
        player.get_single_mut()
    else {
        return;
    };

    let mut under_knockback = false;
    if let Some(knockback) = knockback.as_mut() {
        if knockback.velocity.length() > PLAYER_KNOCKBACK_STOP {
            under_knockback = true;
            velocity.x = knockback.velocity.x;
            if knockback.velocity.y > 1.0 {
                velocity.y = knockback.velocity.y;
                knockback.velocity.y = 0.0;
            }
            velocity.grounded = false;

            let dt = time.delta_secs();
            let decay = (-PLAYER_KNOCKBACK_DECAY * dt).exp();
            knockback.velocity.x *= decay;

            if knockback.velocity.x.abs() <= PLAYER_KNOCKBACK_STOP {
                commands.entity(entity).remove::<PlayerKnockback>();
                under_knockback = false;
            }
        } else {
            commands.entity(entity).remove::<PlayerKnockback>();
        }
    }

    let mut move_input = 0.0;
    if !under_knockback && !attack.is_active() && !block.is_active() {
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            move_input -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            move_input += 1.0;
        }
    }

    if !under_knockback {
        velocity.x = move_input * DUNGEON_MOVE_SPEED;
    }

    if !under_knockback
        && velocity.grounded
        && !attack.is_active()
        && !block.is_active()
        && keyboard.just_pressed(KeyCode::Space)
    {
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
    let max_x = bounds.width - half.x;
    position.x = position.x.clamp(half.x, max_x);

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