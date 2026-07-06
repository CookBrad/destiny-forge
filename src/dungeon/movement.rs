use bevy::prelude::*;

use crate::combat::{
    charge_speed, special_blocks_movement, Health, PlayerAttack, PlayerBlock, PlayerFallDeath,
    PlayerKnockback, PlayerSpecialMove, SpecialMoveKind,
};
use crate::graphics::{
    viewport_bottom_y, DungeonScrollBounds, DUNGEON_AIR_JUMP_MULT, DUNGEON_GRAVITY,
    DUNGEON_JUMP_SPEED, PLAYER_WALK_SPEED,
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

#[derive(Component, Default)]
pub struct PlayerAirJumps {
    pub remaining: u8,
}

const MAX_AIR_JUMPS: u8 = 1;
const PLAYER_KNOCKBACK_DECAY: f32 = 7.0;
const PLAYER_KNOCKBACK_STOP: f32 = 22.0;

pub fn dungeon_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    window: Query<&Window>,
    mut commands: Commands,
    bounds: Res<DungeonScrollBounds>,
    platforms: Query<&PlatformCollider>,
    mut player: Query<
        (
            Entity,
            &mut Transform,
            &mut PlayerVelocity,
            &mut PlayerAirJumps,
            &mut Health,
            &PlayerAttack,
            &PlayerBlock,
            Option<&PlayerSpecialMove>,
            Option<&mut PlayerKnockback>,
        ),
        With<DungeonPlayer>,
    >,
) {
    let Ok((
        entity,
        mut transform,
        mut velocity,
        mut air_jumps,
        mut health,
        attack,
        block,
        special,
        mut knockback,
    )) = player.get_single_mut()
    else {
        return;
    };

    if health.is_dead() {
        return;
    }

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

    let movement_locked = special_blocks_movement(special);

    if movement_locked {
        if let Some(special) = special {
            velocity.x = special.charge_direction * charge_speed();
        }
    } else {
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
            velocity.x = move_input * PLAYER_WALK_SPEED;
        }
    }

    let jump_blocked = attack.is_active()
        || block.is_active()
        || special.is_some_and(|m| m.is_active() && m.kind == SpecialMoveKind::Charge);

    if !under_knockback && !jump_blocked && keyboard.just_pressed(KeyCode::Space) {
        if velocity.grounded {
            velocity.y = DUNGEON_JUMP_SPEED;
            velocity.grounded = false;
            air_jumps.remaining = MAX_AIR_JUMPS;
        } else if air_jumps.remaining > 0 {
            velocity.y = DUNGEON_JUMP_SPEED * DUNGEON_AIR_JUMP_MULT;
            air_jumps.remaining -= 1;
        }
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
                air_jumps.remaining = MAX_AIR_JUMPS;
                break;
            }
        }
    }

    if let Ok(window) = window.get_single() {
        let off_screen_bottom = position.y + half.y < viewport_bottom_y(window);
        if off_screen_bottom {
            health.current = 0.0;
            commands.entity(entity).insert(PlayerFallDeath);
        }
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