use bevy::prelude::*;

use crate::dungeon::DungeonPlayer;

use super::player_death::PlayerDeath;

pub const PLAYER_INVULN_DURATION: f32 = 0.65;

#[derive(Component)]
pub struct PlayerHitFlash {
    pub timer: Timer,
}

impl PlayerHitFlash {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(PLAYER_INVULN_DURATION, TimerMode::Once),
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct PlayerKnockback {
    pub velocity: Vec2,
}

impl PlayerKnockback {
    pub fn away_from(source: Vec2, player: &Transform, strength: f32) -> Self {
        let player_pos = player.translation.truncate();
        let delta = player_pos - source;
        let horizontal = if delta.x.abs() > 0.5 {
            delta.x.signum()
        } else if player.scale.x < 0.0 {
            1.0
        } else {
            -1.0
        };

        Self {
            velocity: Vec2::new(
                horizontal * PLAYER_KNOCKBACK_X * strength,
                PLAYER_KNOCKBACK_Y * strength,
            ),
        }
    }
}

// World units match 64px TILE (4× classic); keep hurt travel proportional.
const PLAYER_KNOCKBACK_X: f32 = 620.0;
const PLAYER_KNOCKBACK_Y: f32 = 380.0;
pub fn apply_player_hurt(
    commands: &mut Commands,
    player: Entity,
    player_transform: &Transform,
    source: Vec2,
    knockback_strength: f32,
    knockback_resist: f32,
) {
    let strength = knockback_strength * (1.0 - knockback_resist.clamp(0.0, 0.95));
    commands.entity(player).insert((
        PlayerHitFlash::new(),
        PlayerKnockback::away_from(source, player_transform, strength),
    ));
}

pub fn tick_player_hit_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut player: Query<
        (Entity, &mut PlayerHitFlash, &mut Sprite),
        (With<DungeonPlayer>, Without<PlayerDeath>),
    >,
) {
    for (entity, mut flash, mut sprite) in &mut player {
        flash.timer.tick(time.delta());
        sprite.color = Color::srgb(1.0, 0.42, 0.42);

        if flash.timer.finished() {
            sprite.color = Color::WHITE;
            commands.entity(entity).remove::<PlayerHitFlash>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::TILE;

    #[test]
    fn hurt_knockback_scales_with_higher_res_world() {
        let mut transform = Transform::from_xyz(100.0, 200.0, 0.0);
        transform.scale.x = 1.0;
        // Source to the left → knock right at full strength.
        let kb = PlayerKnockback::away_from(Vec2::new(50.0, 200.0), &transform, 1.0);
        assert!(kb.velocity.x > 0.0);
        // Must exceed dungeon knockback stop (88) so the reaction is not a no-op.
        assert!(
            kb.velocity.x.abs() > 88.0,
            "knockback X {} must exceed stop threshold 88",
            kb.velocity.x
        );
        assert!((kb.velocity.x.abs() - PLAYER_KNOCKBACK_X).abs() < 0.01);
        assert!((kb.velocity.y - PLAYER_KNOCKBACK_Y).abs() < 0.01);
        // On TILE=64 world, impulse must cover multiple tiles of initial speed.
        assert!(PLAYER_KNOCKBACK_X > TILE * 4.0);
        assert!(PLAYER_KNOCKBACK_Y > TILE);
    }
}

