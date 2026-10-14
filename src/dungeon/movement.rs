use bevy::prelude::*;

use crate::combat::{
    special_blocks_movement, special_move_speed, Health, PlayerAttack, PlayerBlock,
    PlayerFallDeath, PlayerKnockback, PlayerSpecialMove,
};
use crate::graphics::{
    viewport_bottom_y, DungeonScrollBounds, DUNGEON_AIR_JUMP_MULT, DUNGEON_CEILING_Y,
    DUNGEON_GRAVITY, DUNGEON_JUMP_SPEED, PLAYER_WALK_SPEED,
};
use crate::input::{Action, ActionState};

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
    actions: Res<ActionState>,
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
            velocity.x = special_move_speed(special);
        }
    } else {
        let mut move_input = 0.0;
        if !under_knockback && !attack.is_active() && !block.is_active() {
            if actions.pressed(Action::MoveLeft) {
                move_input -= 1.0;
            }
            if actions.pressed(Action::MoveRight) {
                move_input += 1.0;
            }
        }

        if !under_knockback {
            velocity.x = move_input * PLAYER_WALK_SPEED;
        }
    }

    let jump_blocked = attack.is_active()
        || block.is_active()
        || special.is_some_and(|m| {
            m.is_active()
                && matches!(
                    m.kind,
                    crate::combat::SpecialMoveKind::Charge | crate::combat::SpecialMoveKind::Thrust
                )
        });

    if !under_knockback && !jump_blocked && actions.just_pressed(Action::Jump) {
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
    if hits_ceiling(position.y, half.y, velocity.y) {
        position.y = DUNGEON_CEILING_Y - half.y;
        velocity.y = 0.0;
    }

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

fn hits_ceiling(center_y: f32, half_height: f32, velocity_y: f32) -> bool {
    velocity_y > 0.0 && center_y + half_height >= DUNGEON_CEILING_Y
}

fn feet_was_above(platform_top: f32, previous_feet: f32) -> bool {
    previous_feet >= platform_top - 0.5
}

fn overlaps_x(center_x: f32, half_width: f32, collider: &PlatformCollider) -> bool {
    let left = center_x - half_width;
    let right = center_x + half_width;
    right > collider.min_x && left < collider.max_x
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    use crate::combat::{Health, PlayerAttack, PlayerBlock};
    use crate::graphics::{DungeonScrollBounds, DUNGEON_JUMP_SPEED};
    use crate::input::action_state_after_pressing;

    #[test]
    fn rising_head_bonks_on_the_ceiling_beam() {
        let half = player_half_extents().y;
        assert!(hits_ceiling(DUNGEON_CEILING_Y - half, half, 10.0));
        assert!(!hits_ceiling(DUNGEON_CEILING_Y - half - 1.0, half, 10.0));
        assert!(!hits_ceiling(DUNGEON_CEILING_Y, half, -10.0));
    }

    fn vertical_velocity_after(keys: &[KeyCode]) -> f32 {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(action_state_after_pressing(keys, &[]))
            .insert_resource(DungeonScrollBounds { width: 4_000.0 });
        app.world_mut().spawn((
            DungeonPlayer,
            Transform::from_xyz(200.0, 200.0, 0.0),
            PlayerVelocity {
                x: 0.0,
                y: 0.0,
                grounded: true,
            },
            PlayerAirJumps { remaining: 1 },
            Health::new(100.0),
            finished_attack(),
            PlayerBlock::default(),
        ));
        app.world_mut().run_system_once(dungeon_movement).unwrap();
        app.world_mut()
            .query_filtered::<&PlayerVelocity, With<DungeonPlayer>>()
            .single(app.world())
            .y
    }

    fn finished_attack() -> PlayerAttack {
        let mut attack = PlayerAttack::inactive();
        attack.timer.tick(std::time::Duration::from_secs(1));
        attack
    }

    #[test]
    fn space_still_jumps_and_enter_does_not() {
        assert_eq!(
            vertical_velocity_after(&[KeyCode::Space]),
            DUNGEON_JUMP_SPEED
        );
        assert_eq!(vertical_velocity_after(&[KeyCode::Enter]), 0.0);
    }
}
