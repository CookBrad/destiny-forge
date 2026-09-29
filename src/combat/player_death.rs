use bevy::prelude::*;

use crate::core::{DungeonPlayState, GameState, ProfileDirty};
use crate::dungeon::hunter_pose::death_pose;
use crate::dungeon::{
    player_frame_rect, player_half_extents, DungeonArt, DungeonPlayer, PlatformCollider,
    PlayerAnimation, PlayerVelocity,
};
use crate::graphics::{facing_scale, DungeonScrollBounds, DUNGEON_FLOOR_Y, DUNGEON_GRAVITY};
use crate::overworld::setup::OverworldEntry;

use super::health::Health;
use super::player_block::PlayerBlock;
use super::player_hurt::{PlayerHitFlash, PlayerKnockback};
use super::special_moves::PlayerSpecialMove;
use super::PlayerAttack;

const DEATH_DURATION: f32 = 1.45;
const DEATH_KNOCKBACK_X: f32 = 185.0;
const DEATH_KNOCKBACK_Y: f32 = 165.0;

#[derive(Component)]
pub struct PlayerFallDeath;

#[derive(Component)]
pub struct PlayerDeath {
    pub timer: Timer,
    pub knockback: Vec2,
    /// Upright body center. The flop orbits the sprite and must not feed that pose back into collision.
    pub physics_center: Vec2,
}

impl PlayerDeath {
    pub fn new(facing: f32, physics_center: Vec2) -> Self {
        Self {
            timer: Timer::from_seconds(DEATH_DURATION, TimerMode::Once),
            knockback: Vec2::new(-facing.signum() * DEATH_KNOCKBACK_X, DEATH_KNOCKBACK_Y),
            physics_center,
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
        PlayerDeath::new(animation.facing, transform.translation.truncate()),
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
    mut player: Query<(&mut Transform, &mut PlayerVelocity, &mut PlayerDeath), With<DungeonPlayer>>,
) {
    let Ok((mut transform, mut velocity, mut death)) = player.get_single_mut() else {
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
    let mut position = death.physics_center;
    position.x = (position.x + delta.x).clamp(half.x, bounds.width - half.x);
    position.y += delta.y;

    let mut landed = false;
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
                    landed = true;
                    break;
                }
            }
        }
    }

    if position.y - half.y < DUNGEON_FLOOR_Y {
        position.y = DUNGEON_FLOOR_Y + half.y;
        velocity.y = 0.0;
        velocity.x *= 0.2;
        landed = true;
    }

    velocity.grounded = landed;
    death.physics_center = position;
    transform.translation.x = position.x;
    transform.translation.y = position.y;
}

pub fn animate_player_death(
    art: Res<DungeonArt>,
    mut player: Query<
        (
            &PlayerDeath,
            &PlayerAnimation,
            &PlayerVelocity,
            &mut Sprite,
            &mut Transform,
        ),
        With<DungeonPlayer>,
    >,
) {
    let Ok((death, animation, velocity, mut sprite, mut transform)) = player.get_single_mut()
    else {
        return;
    };

    let t = death.progress();
    let facing = animation.facing.signum().clamp(-1.0, 1.0);
    let fall = ((t - 0.18) / 0.55).clamp(0.0, 1.0);
    let pose = death_pose(
        death.physics_center,
        velocity.y,
        velocity.grounded,
        facing,
        fall,
    );

    sprite.image = art.hunter_image(pose.sheet);
    sprite.rect = Some(player_frame_rect(pose.cell));
    sprite.anchor = pose.anchor;
    transform.scale = facing_scale(facing);
    transform.rotation = Quat::from_rotation_z(pose.tilt);
    transform.translation.x = pose.visual_center.x;
    transform.translation.y = pose.visual_center.y;

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
