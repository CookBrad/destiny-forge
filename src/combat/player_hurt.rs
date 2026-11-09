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

const PLAYER_KNOCKBACK_X: f32 = 155.0;
const PLAYER_KNOCKBACK_Y: f32 = 95.0;
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

pub(crate) type PlayerHitFlashTicks<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut PlayerHitFlash, &'static mut Sprite),
    (With<DungeonPlayer>, Without<PlayerDeath>),
>;

pub fn tick_player_hit_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut player: PlayerHitFlashTicks,
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

    #[test]
    fn a_fresh_hit_flash_paints_the_hunter_until_invulnerability_ends() {
        let mut app = flash_app();
        let hunter = app
            .world_mut()
            .spawn((
                DungeonPlayer,
                PlayerHitFlash::new(),
                Sprite {
                    color: Color::BLACK,
                    ..default()
                },
            ))
            .id();
        app.add_systems(Update, tick_player_hit_flash);
        app.update();
        let sprite = app.world().get::<Sprite>(hunter).unwrap();
        assert_eq!(sprite.color, Color::srgb(1.0, 0.42, 0.42));
        assert!(app.world().get::<PlayerHitFlash>(hunter).is_some());
    }

    fn flash_app() -> App {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        app
    }

    #[test]
    fn a_finished_hit_flash_restores_white_and_clears_the_component() {
        let mut app = flash_app();
        let mut flash = PlayerHitFlash::new();
        flash
            .timer
            .tick(std::time::Duration::from_secs_f32(PLAYER_INVULN_DURATION));
        let hunter = app
            .world_mut()
            .spawn((
                DungeonPlayer,
                flash,
                Sprite {
                    color: Color::BLACK,
                    ..default()
                },
            ))
            .id();
        app.add_systems(Update, tick_player_hit_flash);
        app.update();
        let sprite = app.world().get::<Sprite>(hunter).unwrap();
        assert_eq!(sprite.color, Color::WHITE);
        assert!(app.world().get::<PlayerHitFlash>(hunter).is_none());
    }

    #[test]
    fn a_dying_hunter_keeps_the_color_they_already_had() {
        let mut app = flash_app();
        let hunter = app
            .world_mut()
            .spawn((
                DungeonPlayer,
                PlayerDeath::new(1.0, Vec2::ZERO),
                PlayerHitFlash::new(),
                Sprite {
                    color: Color::BLACK,
                    ..default()
                },
            ))
            .id();
        app.add_systems(Update, tick_player_hit_flash);
        app.update();
        let sprite = app.world().get::<Sprite>(hunter).unwrap();
        assert_eq!(sprite.color, Color::BLACK);
        assert!(app.world().get::<PlayerHitFlash>(hunter).is_some());
    }
}
