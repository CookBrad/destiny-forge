use bevy::prelude::*;

use std::f32::consts::FRAC_PI_2;

use crate::core::{DungeonPlayState, GameState, ProfileDirty};
use crate::overworld::setup::OverworldEntry;
use crate::dungeon::{
    player_frame_rect, player_half_extents, DungeonArt, DungeonPlayer, PlatformCollider,
    PlayerAnimation, PlayerVelocity, PLAYER_IDLE_FRAMES,
};
use crate::graphics::{facing_scale, DungeonScrollBounds, DUNGEON_FLOOR_Y, DUNGEON_GRAVITY, TILE};

use super::attack::{WeaponOnBack, WeaponSwingFx};
use super::block::WeaponBlockFx;
use super::health::Health;
use super::player_block::PlayerBlock;
use super::player_hurt::{PlayerHitFlash, PlayerKnockback};
use super::special_moves::{PlayerSpecialMove, WeaponSpecialFx};
use super::PlayerAttack;

const DEATH_DURATION: f32 = 1.45;
// World units match 64px TILE (4× classic); keep death fling proportional.
const DEATH_KNOCKBACK_X: f32 = 740.0;
const DEATH_KNOCKBACK_Y: f32 = 660.0;

#[derive(Component)]
pub struct PlayerFallDeath;

#[derive(Component)]
pub struct PlayerDeath {
    pub timer: Timer,
    pub knockback: Vec2,
    pub ground_y: f32,
}

impl PlayerDeath {
    pub fn new(facing: f32, ground_y: f32) -> Self {
        Self {
            timer: Timer::from_seconds(DEATH_DURATION, TimerMode::Once),
            knockback: Vec2::new(-facing.signum() * DEATH_KNOCKBACK_X, DEATH_KNOCKBACK_Y),
            ground_y,
        }
    }

    pub fn progress(&self) -> f32 {
        (self.timer.elapsed_secs() / DEATH_DURATION).clamp(0.0, 1.0)
    }

    pub fn is_finished(&self) -> bool {
        self.timer.finished()
    }
}

pub fn detect_player_death(
    mut commands: Commands,
    mut next_play: ResMut<NextState<DungeonPlayState>>,
    mut next_game: ResMut<NextState<GameState>>,
    mut profile_dirty: ResMut<ProfileDirty>,
    player: Query<
        (
            Entity,
            &Transform,
            &Health,
            &PlayerAnimation,
            Option<&PlayerDeath>,
            Option<&PlayerFallDeath>,
        ),
        With<DungeonPlayer>,
    >,
) {
    let Ok((entity, transform, health, animation, death, fall_death)) = player.get_single() else {
        return;
    };

    if !health.is_dead() || death.is_some() {
        return;
    }

    if fall_death.is_some() {
        commands.entity(entity).insert(Visibility::Hidden);
        commands.entity(entity).remove::<(
            PlayerAttack,
            PlayerBlock,
            PlayerSpecialMove,
            PlayerKnockback,
            PlayerHitFlash,
            PlayerFallDeath,
        )>();
        profile_dirty.mark();
        commands.insert_resource(OverworldEntry::DungeonReturn);
        next_game.set(GameState::Overworld);
        return;
    }

    commands.entity(entity).insert((
        PlayerDeath::new(animation.facing, transform.translation.y),
        PlayerVelocity {
            x: 0.0,
            y: 0.0,
            grounded: false,
        },
    ));
    commands.entity(entity).remove::<(
        PlayerAttack,
        PlayerBlock,
        PlayerSpecialMove,
        PlayerKnockback,
        PlayerHitFlash,
    )>();
    next_play.set(DungeonPlayState::Dying);
}

pub fn tick_player_death(
    time: Res<Time>,
    bounds: Res<DungeonScrollBounds>,
    platforms: Query<&PlatformCollider>,
    mut player: Query<
        (
            Entity,
            &mut Transform,
            &mut PlayerVelocity,
            &mut PlayerDeath,
            &Children,
        ),
        With<DungeonPlayer>,
    >,
    mut visibility: Query<&mut Visibility>,
) {
    let Ok((_entity, mut transform, mut velocity, mut death, children)) = player.get_single_mut() else {
        return;
    };

    death.timer.tick(time.delta());

    let dt = time.delta_secs();
    velocity.x = death.knockback.x;
    if death.knockback.y > 1.0 {
        velocity.y = death.knockback.y;
        death.knockback.y = 0.0;
    } else {
        velocity.y += DUNGEON_GRAVITY * dt;
    }

    death.knockback.x *= (-6.5 * dt).exp();

    let half = player_half_extents();
    let delta = Vec2::new(velocity.x, velocity.y) * dt;
    let mut position = transform.translation.truncate();
    position.x = (position.x + delta.x).clamp(half.x, bounds.width - half.x);
    position.y += delta.y;

    let feet_y = position.y - half.y;
    if velocity.y <= 0.0 {
        for collider in &platforms {
            if feet_y <= collider.top_y && feet_y >= collider.top_y - delta.y.abs() - 0.5 {
                let left = position.x - half.x;
                let right = position.x + half.x;
                if right > collider.min_x && left < collider.max_x {
                    position.y = collider.top_y + half.y;
                    velocity.y = 0.0;
                    velocity.x *= 0.35;
                    break;
                }
            }
        }
    }

    if position.y - half.y < DUNGEON_FLOOR_Y {
        position.y = DUNGEON_FLOOR_Y + half.y;
        velocity.y = 0.0;
        velocity.x *= 0.2;
    }

    transform.translation.x = position.x;
    transform.translation.y = position.y;
    death.ground_y = position.y;

    if death.timer.elapsed_secs() > 0.08 {
        for child in children.iter() {
            if let Ok(mut vis) = visibility.get_mut(*child) {
                *vis = Visibility::Hidden;
            }
        }
    }
}

pub fn animate_player_death(
    art: Res<DungeonArt>,
    mut player: Query<
        (
            &PlayerDeath,
            &PlayerAnimation,
            &mut Sprite,
            &mut Transform,
        ),
        With<DungeonPlayer>,
    >,
) {
    let Ok((death, animation, mut sprite, mut transform)) = player.get_single_mut() else {
        return;
    };

    let t = death.progress();
    let frame = if t < 0.22 {
        1
    } else if t < 0.5 {
        2
    } else {
        (PLAYER_IDLE_FRAMES - 1).min(3)
    };

    sprite.image = art.player_idle.clone();
    sprite.rect = Some(player_frame_rect(frame));

    let facing = animation.facing.signum().max(-1.0).min(1.0);
    transform.scale = facing_scale(facing);

    let fall = ((t - 0.18) / 0.55).clamp(0.0, 1.0);
    let tilt = -facing * fall * FRAC_PI_2 * 0.9;
    transform.rotation = Quat::from_rotation_z(tilt);

    let sink = fall * TILE * 0.35;
    transform.translation.y = death.ground_y - sink;

    let alpha = if t > 0.78 {
        1.0 - ((t - 0.78) / 0.22)
    } else {
        1.0
    };
    let shade = 0.52 + 0.28 * (1.0 - t);
    sprite.color = Color::srgba(shade, shade * 0.82, shade * 0.88, alpha);
}

pub fn finish_player_death(
    mut commands: Commands,
    player: Query<&PlayerDeath, With<DungeonPlayer>>,
    mut next_game: ResMut<NextState<GameState>>,
    mut profile_dirty: ResMut<ProfileDirty>,
) {
    let Ok(death) = player.get_single() else {
        return;
    };

    if death.is_finished() {
        profile_dirty.mark();
        commands.insert_resource(OverworldEntry::DungeonReturn);
        next_game.set(GameState::Overworld);
    }
}

pub fn hide_death_weapons(
    player: Query<&Children, (With<DungeonPlayer>, With<PlayerDeath>)>,
    weapons: Query<
        Entity,
        Or<(
            With<WeaponOnBack>,
            With<WeaponSwingFx>,
            With<WeaponBlockFx>,
            With<WeaponSpecialFx>,
        )>,
    >,
    mut visibility: Query<&mut Visibility>,
) {
    let Ok(children) = player.get_single() else {
        return;
    };

    for child in children.iter() {
        if weapons.get(*child).is_ok() {
            if let Ok(mut vis) = visibility.get_mut(*child) {
                *vis = Visibility::Hidden;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn death_knockback_scales_with_higher_res_world() {
        let death = PlayerDeath::new(1.0, DUNGEON_FLOOR_Y);
        // Facing right → fling left.
        assert!(death.knockback.x < 0.0);
        assert!((death.knockback.x.abs() - DEATH_KNOCKBACK_X).abs() < 0.01);
        assert!((death.knockback.y - DEATH_KNOCKBACK_Y).abs() < 0.01);
        // Must be multi-tile impulse on TILE=64 (classic 185 was ~11.5 tiles of 16px).
        assert!(
            DEATH_KNOCKBACK_X > TILE * 10.0,
            "death X {} should exceed 10 tiles of speed",
            DEATH_KNOCKBACK_X
        );
        assert!(DEATH_KNOCKBACK_Y > TILE * 5.0);
    }
}