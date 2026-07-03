use bevy::prelude::*;

use crate::dungeon::DungeonPlayer;

#[derive(Component)]
pub struct PlayerHitFlash {
    pub timer: Timer,
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

const PLAYER_KNOCKBACK_X: f32 = 155.0;
const PLAYER_KNOCKBACK_Y: f32 = 95.0;
pub fn apply_player_hurt(
    commands: &mut Commands,
    player: Entity,
    player_transform: &Transform,
    source: Vec2,
    knockback_strength: f32,
) {
    commands.entity(player).insert((
        PlayerHitFlash {
            timer: Timer::from_seconds(0.14, TimerMode::Once),
        },
        PlayerKnockback::away_from(source, player_transform, knockback_strength),
    ));
}

pub fn tick_player_hit_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut player: Query<(Entity, &mut PlayerHitFlash, &mut Sprite), With<DungeonPlayer>>,
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

