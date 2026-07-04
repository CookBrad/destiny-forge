use bevy::prelude::*;

use std::f32::consts::FRAC_PI_2;

use crate::combat::EnemyCorpse;
use crate::audio::CombatSfx;
use crate::dungeon::{
    DungeonArt, DungeonEntity, DungeonPlayer, EnemyAggro, EnemyHitbox, EnemyKind, EnemyKnockback,
    EnemyShootCooldown, KingSlimeBoss, Patrol,
};
use crate::graphics::{PIXEL_SCALE, TILE};

use super::attack::{player_sword_hit_rect, HitFlash, PlayerAttack};
use super::special_moves::{spin_deflects_projectile, PlayerSpecialMove};
use super::hitbox::{enemy_aabb, hitbox_overlaps, player_body_rect, sword_guard_aabb, HitRect};
use super::player_block::PlayerBlock;
use super::health::{ContactDamageCooldown, Health};
use super::player_hurt::apply_player_hurt;

#[derive(Component)]
pub struct EnemyProjectile {
    pub damage: f32,
}

#[derive(Component)]
pub struct ProjectileVelocity(pub Vec2);

#[derive(Component)]
pub struct ProjectileLifetime {
    pub remaining: f32,
}

/// Tracks whether a projectile was parried and which enemies it already struck.
#[derive(Component, Default)]
pub struct DeflectedProjectile {
    pub active: bool,
    pub hit_entities: Vec<Entity>,
}

const ARROW_WIDTH: f32 = 7.0;
const ARROW_HEIGHT: f32 = 21.0;
const PROJECTILE_LIFETIME: f32 = 4.0;
const PROJECTILE_Z: f32 = 4.5;
const PROJECTILE_HIT_HALF: Vec2 = Vec2::new(ARROW_WIDTH * 0.5, ARROW_HEIGHT * 0.5);
const PROJECTILE_DAMAGE_INTERVAL: f32 = 0.45;
const DEFLECT_SPEED_MULT: f32 = 1.2;

pub fn enemy_shoot_projectiles(
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    time: Res<Time>,
    art: Res<DungeonArt>,
    player: Query<&Transform, With<DungeonPlayer>>,
    mut shooters: Query<
        (
            Entity,
            &Transform,
            &EnemyKind,
            &Health,
            &mut EnemyShootCooldown,
            Option<&EnemyAggro>,
        ),
        (Without<EnemyCorpse>, Without<DungeonPlayer>),
    >,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (entity, transform, kind, health, mut cooldown, aggro) in &mut shooters {
        if health.is_dead() || !kind.shoots_projectiles() {
            continue;
        }

        cooldown.0.tick(time.delta());
        if !cooldown.0.finished() {
            continue;
        }

        let enemy_pos = transform.translation.truncate();
        let to_player = player_pos - enemy_pos;
        let distance = to_player.length();

        let has_aggro = aggro.is_some() || distance < 20.0 * TILE;
        if !has_aggro || distance > kind.shoot_range() || distance < TILE {
            continue;
        }

        let direction = to_player.normalize_or_zero();
        if direction == Vec2::ZERO {
            continue;
        }

        let spawn = enemy_pos + direction * (TILE * 0.55);
        let velocity = direction * kind.projectile_speed();
        let angle = direction.y.atan2(direction.x) - FRAC_PI_2;

        let color = match kind {
            EnemyKind::Skeleton => Color::srgb(0.95, 0.92, 0.75),
            EnemyKind::Goblin => Color::srgb(0.75, 0.95, 0.55),
            EnemyKind::Bat => Color::srgb(0.85, 0.65, 1.0),
            _ => Color::WHITE,
        };

        commands.spawn((
            Sprite {
                image: art.arrow.clone(),
                color,
                ..default()
            },
            Transform {
                translation: Vec3::new(spawn.x, spawn.y, PROJECTILE_Z),
                rotation: Quat::from_rotation_z(angle),
                scale: Vec3::splat(PIXEL_SCALE),
                ..default()
            },
            EnemyProjectile {
                damage: kind.projectile_damage(),
            },
            ProjectileVelocity(velocity),
            ProjectileLifetime {
                remaining: PROJECTILE_LIFETIME,
            },
            DeflectedProjectile::default(),
            DungeonEntity,
        ));

        cooldown.0 = Timer::from_seconds(kind.shoot_cooldown(), TimerMode::Once);
        commands.entity(entity).insert(EnemyAggro::from_hit());
        sfx.send(CombatSfx::EnemyShoot);
    }
}

pub fn move_enemy_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    bounds: Res<crate::graphics::DungeonScrollBounds>,
    mut projectiles: Query<(Entity, &mut Transform, &ProjectileVelocity, &mut ProjectileLifetime)>,
) {
    let dt = time.delta_secs();
    let margin = TILE * 4.0;

    for (entity, mut transform, velocity, mut lifetime) in &mut projectiles {
        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;

        lifetime.remaining -= dt;
        let x = transform.translation.x;
        if lifetime.remaining <= 0.0 || x < -margin || x > bounds.width + margin {
            commands.entity(entity).despawn();
        }
    }
}

pub fn deflect_projectiles_with_swing(
    mut sfx: EventWriter<CombatSfx>,
    player: Query<
        (&Transform, &PlayerAttack, Option<&PlayerSpecialMove>),
        (With<DungeonPlayer>, Without<EnemyProjectile>),
    >,
    mut projectiles: Query<
        (
            &mut Transform,
            &mut ProjectileVelocity,
            &mut Sprite,
            &mut DeflectedProjectile,
        ),
        (With<EnemyProjectile>, Without<DungeonPlayer>),
    >,
) {
    let Ok((player_transform, attack, special)) = player.get_single() else {
        return;
    };

    for (mut projectile_transform, mut velocity, mut sprite, mut deflected) in &mut projectiles {
        if deflected.active {
            continue;
        }

        let center = projectile_transform.translation.truncate();
        let projectile_hit = projectile_rect(center);

        let deflected_by_swing = player_sword_hit_rect(player_transform, attack)
            .is_some_and(|swing_rect| hitbox_overlaps(swing_rect, projectile_hit));

        let deflected_by_spin = special.is_some_and(|special| {
            spin_deflects_projectile(player_transform, special, center, projectile_hit)
        });

        if !deflected_by_swing && !deflected_by_spin {
            continue;
        }

        velocity.0 = -velocity.0 * DEFLECT_SPEED_MULT;
        let angle = velocity.0.y.atan2(velocity.0.x) - FRAC_PI_2;
        projectile_transform.rotation = Quat::from_rotation_z(angle);
        sprite.color = Color::srgb(1.0, 0.95, 0.55);
        deflected.active = true;
        deflected.hit_entities.clear();
        sfx.send(CombatSfx::Parry);
    }
}

pub fn resolve_deflected_projectile_hits(
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    mut projectiles: Query<
        (
            Entity,
            &Transform,
            &EnemyProjectile,
            &mut DeflectedProjectile,
        ),
        (With<EnemyProjectile>, Without<DungeonPlayer>),
    >,
    mut enemies: Query<
        (
            Entity,
            &Transform,
            &EnemyHitbox,
            &mut Health,
            &mut Sprite,
            Option<&KingSlimeBoss>,
            Option<&EnemyKind>,
        ),
        (
            With<Health>,
            Without<DungeonPlayer>,
            Without<EnemyCorpse>,
            Without<EnemyProjectile>,
            Without<DeflectedProjectile>,
        ),
    >,
) {
    for (projectile_entity, transform, projectile, mut deflected) in &mut projectiles {
        if !deflected.active {
            continue;
        }

        let projectile_rect = projectile_rect(transform.translation.truncate());

        for (enemy_entity, enemy_transform, hitbox, mut health, mut sprite, boss, kind) in
            &mut enemies
        {
            if deflected.hit_entities.contains(&enemy_entity) || health.is_dead() {
                continue;
            }

            if !hitbox_overlaps(projectile_rect, enemy_aabb(
                enemy_transform.translation.truncate(),
                hitbox.0,
            )) {
                continue;
            }

            health.take_damage(projectile.damage);
            deflected.hit_entities.push(enemy_entity);
            sfx.send(CombatSfx::SwordHit);

            sprite.color = Color::srgb(1.0, 0.45, 0.45);
            commands.entity(enemy_entity).insert((
                EnemyAggro::from_hit(),
                HitFlash {
                    timer: Timer::from_seconds(0.12, TimerMode::Once),
                },
                EnemyKnockback::away_from_player(
                    transform,
                    enemy_transform,
                    if boss.is_some() { 0.35 } else { 1.0 },
                    kind.is_some_and(|kind| kind.is_airborne()),
                ),
            ));

            if health.is_dead() {
                commands.entity(enemy_entity).remove::<Patrol>();
                commands.entity(enemy_entity).insert(EnemyCorpse);
                sprite.color = Color::srgba(0.55, 0.55, 0.6, 0.85);
            }

            commands.entity(projectile_entity).despawn();
            break;
        }
    }
}

pub fn resolve_enemy_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    mut player: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            &mut ContactDamageCooldown,
            &PlayerBlock,
        ),
        (With<DungeonPlayer>, Without<EnemyProjectile>),
    >,
    projectiles: Query<
        (Entity, &Transform, &EnemyProjectile, &DeflectedProjectile),
        (With<EnemyProjectile>, Without<DungeonPlayer>),
    >,
) {
    let Ok((player_entity, player_transform, mut health, mut cooldown, block)) =
        player.get_single_mut()
    else {
        return;
    };

    if health.is_dead() {
        return;
    }

    cooldown.0.tick(time.delta());

    let body = player_body_rect(player_transform);
    let guard = sword_guard_aabb(player_transform);
    let mut took_damage_this_frame = false;

    for (projectile_entity, transform, projectile, deflected) in &projectiles {
        if deflected.active {
            continue;
        }

        let center = transform.translation.truncate();
        let projectile_hit = projectile_rect(center);

        if block.is_active() && hitbox_overlaps(guard, projectile_hit) {
            commands.entity(projectile_entity).despawn();
            sfx.send(CombatSfx::Parry);
            continue;
        }

        if hitbox_overlaps(body, projectile_hit) {
            if !took_damage_this_frame && cooldown.0.finished() {
                health.take_damage(projectile.damage);
                apply_player_hurt(
                    &mut commands,
                    player_entity,
                    player_transform,
                    center,
                    0.85,
                );
                sfx.send(CombatSfx::PlayerHurt);
                cooldown.0 = Timer::from_seconds(PROJECTILE_DAMAGE_INTERVAL, TimerMode::Once);
                took_damage_this_frame = true;
            }
            commands.entity(projectile_entity).despawn();
        }
    }
}

fn projectile_rect(center: Vec2) -> HitRect {
    HitRect {
        min_x: center.x - PROJECTILE_HIT_HALF.x,
        max_x: center.x + PROJECTILE_HIT_HALF.x,
        min_y: center.y - PROJECTILE_HIT_HALF.y,
        max_y: center.y + PROJECTILE_HIT_HALF.y,
    }
}