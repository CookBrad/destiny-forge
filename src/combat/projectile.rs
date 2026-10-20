use bevy::prelude::*;

use std::f32::consts::FRAC_PI_2;

use crate::audio::CombatSfx;
use crate::combat::EnemyCorpse;
use crate::dungeon::{
    DungeonArt, DungeonEntity, DungeonPlayer, EnemyAggro, EnemyHitbox, EnemyKind, EnemyKnockback,
    EnemyShootCooldown, KingSlimeBoss, KnockbackTarget, Patrol,
};
use crate::graphics::TILE;

use super::attack::{player_sword_hit_rect, HitFlash, PlayerAttack};
use super::hitbox::{enemy_aabb, hitbox_overlaps, player_body_rect, sword_guard_aabb, HitRect};
use super::player_block::PlayerBlock;
use super::special_moves::{spin_deflects_projectile, PlayerSpecialMove};
use crate::player::Loadout;

use super::health::{damage_amount, ContactDamageCooldown, Health};
use super::player_hurt::{apply_player_hurt, PlayerHitFlash};

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

/// Gameplay hit stays the legacy 7×21 box. The sheet is 16×48 and is not this hit.
const ARROW_WIDTH: f32 = 7.0;
const ARROW_HEIGHT: f32 = 21.0;
const PROJECTILE_LIFETIME: f32 = 4.0;
const PROJECTILE_Z: f32 = 4.5;
const PROJECTILE_HIT_HALF: Vec2 = Vec2::new(ARROW_WIDTH * 0.5, ARROW_HEIGHT * 0.5);
const PROJECTILE_DAMAGE_INTERVAL: f32 = 0.45;
const DEFLECT_SPEED_MULT: f32 = 1.2;

pub(crate) type ProjectileShooters<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static EnemyKind,
        &'static Health,
        &'static mut EnemyShootCooldown,
        Option<&'static EnemyAggro>,
    ),
    (Without<EnemyCorpse>, Without<DungeonPlayer>),
>;

pub fn enemy_shoot_projectiles(
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    time: Res<Time>,
    art: Res<DungeonArt>,
    player: Query<&Transform, With<DungeonPlayer>>,
    mut shooters: ProjectileShooters,
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
    mut projectiles: Query<(
        Entity,
        &mut Transform,
        &ProjectileVelocity,
        &mut ProjectileLifetime,
    )>,
) {
    let dt = time.delta_secs();
    let margin = TILE * 4.0;

    for (entity, mut transform, velocity, mut lifetime) in &mut projectiles {
        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;

        lifetime.remaining -= dt;
        let x = transform.translation.x;
        if lifetime.remaining <= 0.0 || x < -margin || x > bounds.width + margin {
            commands.entity(entity).try_despawn();
        }
    }
}

pub(crate) type SwingDeflectPlayer<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        &'static PlayerAttack,
        Option<&'static PlayerSpecialMove>,
    ),
    (With<DungeonPlayer>, Without<EnemyProjectile>),
>;

pub(crate) type SwingDeflectProjectiles<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut ProjectileVelocity,
        &'static mut Sprite,
        &'static mut DeflectedProjectile,
    ),
    (With<EnemyProjectile>, Without<DungeonPlayer>),
>;

pub fn deflect_projectiles_with_swing(
    mut sfx: EventWriter<CombatSfx>,
    player: SwingDeflectPlayer,
    mut projectiles: SwingDeflectProjectiles,
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

pub(crate) type DeflectedProjectileShots<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static EnemyProjectile,
        &'static mut DeflectedProjectile,
    ),
    (With<EnemyProjectile>, Without<DungeonPlayer>),
>;

pub(crate) type DeflectedHitEnemies<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static EnemyHitbox,
        &'static mut Health,
        &'static mut Sprite,
        Option<&'static KingSlimeBoss>,
        Option<&'static EnemyKind>,
    ),
    (
        With<Health>,
        Without<DungeonPlayer>,
        Without<EnemyCorpse>,
        Without<EnemyProjectile>,
        Without<DeflectedProjectile>,
    ),
>;

pub fn resolve_deflected_projectile_hits(
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    mut projectiles: DeflectedProjectileShots,
    mut enemies: DeflectedHitEnemies,
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

            if !hitbox_overlaps(
                projectile_rect,
                enemy_aabb(enemy_transform.translation.truncate(), hitbox.0),
            ) {
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
                    KnockbackTarget::of(kind, boss),
                ),
            ));

            if health.is_dead() {
                commands.entity(enemy_entity).remove::<Patrol>();
                commands.entity(enemy_entity).insert(EnemyCorpse);
                sprite.color = Color::srgba(0.55, 0.55, 0.6, 0.85);
            }

            commands.entity(projectile_entity).try_despawn();
            break;
        }
    }
}

pub(crate) type ProjectileHitPlayer<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static mut Health,
        &'static mut ContactDamageCooldown,
        &'static PlayerBlock,
        Option<&'static PlayerHitFlash>,
    ),
    (With<DungeonPlayer>, Without<EnemyProjectile>),
>;

pub(crate) type IncomingProjectiles<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Transform,
        &'static EnemyProjectile,
        &'static mut ProjectileVelocity,
        &'static mut Sprite,
        &'static mut DeflectedProjectile,
    ),
    (With<EnemyProjectile>, Without<DungeonPlayer>),
>;

pub fn resolve_enemy_projectiles(
    time: Res<Time>,
    loadout: Res<Loadout>,
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    mut player: ProjectileHitPlayer,
    mut projectiles: IncomingProjectiles,
) {
    let Ok((player_entity, player_transform, mut health, mut cooldown, block, hit_flash)) =
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

    for (projectile_entity, mut transform, projectile, mut velocity, mut sprite, mut deflected) in
        &mut projectiles
    {
        if deflected.active {
            continue;
        }

        let center = transform.translation.truncate();
        let projectile_hit = projectile_rect(center);

        if block.is_active() && hitbox_overlaps(guard, projectile_hit) {
            if block.in_parry_window() {
                velocity.0 = -velocity.0 * DEFLECT_SPEED_MULT;
                let angle = velocity.0.y.atan2(velocity.0.x) - FRAC_PI_2;
                transform.rotation = Quat::from_rotation_z(angle);
                sprite.color = Color::srgb(1.0, 0.95, 0.55);
                deflected.active = true;
                deflected.hit_entities.clear();
                sfx.send(CombatSfx::Parry);
            } else {
                commands.entity(projectile_entity).try_despawn();
                sfx.send(CombatSfx::Block);
            }
            continue;
        }

        if hitbox_overlaps(body, projectile_hit) {
            if hit_flash.is_none() && !took_damage_this_frame && cooldown.0.finished() {
                health.take_damage(damage_amount(projectile.damage, loadout.total_defense()));
                apply_player_hurt(
                    &mut commands,
                    player_entity,
                    player_transform,
                    center,
                    0.85,
                    loadout.knockback_resist(),
                );
                sfx.send(CombatSfx::PlayerHurt);
                cooldown.0 = Timer::from_seconds(PROJECTILE_DAMAGE_INTERVAL, TimerMode::Once);
                took_damage_this_frame = true;
            }
            commands.entity(projectile_entity).try_despawn();
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn blank_art(image: Handle<Image>) -> DungeonArt {
        DungeonArt {
            player_idle: image.clone(),
            player_run: image.clone(),
            player_attack: image.clone(),
            player_jump: image.clone(),
            player_fall: image.clone(),
            player_death: image.clone(),
            floor_ground: image.clone(),
            floor_platform: image.clone(),
            floor_ladder: image.clone(),
            wall: image.clone(),
            floor_pit: image.clone(),
            pit_stake: image.clone(),
            pit_lip: image.clone(),
            ground_fill: image.clone(),
            ceiling_beam: image.clone(),
            pillar: image.clone(),
            torch: image.clone(),
            chain: image.clone(),
            ground_slam: image.clone(),
            slime: image.clone(),
            bat: image.clone(),
            goblin: image.clone(),
            skeleton: image.clone(),
            zombie: image.clone(),
            king_slime: image.clone(),
            arrow: image.clone(),
            slime_bolt: image.clone(),
            slime_blob: image.clone(),
            fill: image,
        }
    }

    fn projectile_app() -> App {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.insert_resource(Time::<()>::default());
        app.add_event::<CombatSfx>();
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::default());
        app.insert_resource(blank_art(image));
        app
    }

    fn projectile_count(app: &mut App) -> usize {
        let mut shots = app.world_mut().query::<&EnemyProjectile>();
        shots.iter(app.world()).len()
    }

    #[test]
    fn projectile_hit_stays_seven_by_twenty_one() {
        assert_eq!(ARROW_WIDTH, 7.0);
        assert_eq!(ARROW_HEIGHT, 21.0);
        assert_eq!(PROJECTILE_HIT_HALF, Vec2::new(3.5, 10.5));
    }

    #[test]
    fn a_skeleton_in_range_shoots_an_arrow_at_the_hunter() {
        let mut app = projectile_app();
        app.world_mut()
            .spawn((DungeonPlayer, Transform::from_xyz(400.0, 100.0, 0.0)));
        app.world_mut().spawn((
            Transform::from_xyz(200.0, 100.0, 0.0),
            EnemyKind::Skeleton,
            Health::new(20.0),
            EnemyShootCooldown(Timer::from_seconds(0.0, TimerMode::Once)),
        ));
        app.world_mut()
            .run_system_once(enemy_shoot_projectiles)
            .unwrap();
        assert_eq!(projectile_count(&mut app), 1);
        let shot = app
            .world_mut()
            .query::<(&Transform, &EnemyProjectile, &ProjectileVelocity)>()
            .single(app.world());
        assert_eq!(shot.0.translation, Vec3::new(217.6, 100.0, PROJECTILE_Z));
        assert_eq!(shot.1.damage, 8.0);
        assert_eq!(shot.2.0, Vec2::new(220.0, 0.0));
    }

    #[test]
    fn a_skeleton_closer_than_one_tile_does_not_shoot() {
        let mut app = projectile_app();
        app.world_mut()
            .spawn((DungeonPlayer, Transform::from_xyz(210.0, 100.0, 0.0)));
        app.world_mut().spawn((
            Transform::from_xyz(200.0, 100.0, 0.0),
            EnemyKind::Skeleton,
            Health::new(20.0),
            EnemyShootCooldown(Timer::from_seconds(0.0, TimerMode::Once)),
        ));
        app.world_mut()
            .run_system_once(enemy_shoot_projectiles)
            .unwrap();
        assert_eq!(projectile_count(&mut app), 0);
    }

    #[test]
    fn enemies_do_not_shoot_when_no_hunter_is_present() {
        let mut app = projectile_app();
        app.world_mut().spawn((
            Transform::from_xyz(200.0, 100.0, 0.0),
            EnemyKind::Skeleton,
            Health::new(20.0),
            EnemyShootCooldown(Timer::from_seconds(0.0, TimerMode::Once)),
        ));
        app.world_mut()
            .run_system_once(enemy_shoot_projectiles)
            .unwrap();
        assert_eq!(projectile_count(&mut app), 0);
    }

    fn sword_player(elapsed: f32) -> (Transform, PlayerAttack) {
        let transform = Transform::from_xyz(400.0, 120.0, 1.0);
        let mut attack = PlayerAttack::inactive();
        attack.weapon = crate::combat::WeaponKind::RustySword;
        attack.step_index = 0;
        let duration = attack.weapon.moveset().steps[0].duration;
        attack.timer = Timer::from_seconds(duration, TimerMode::Once);
        attack
            .timer
            .tick(std::time::Duration::from_secs_f32(elapsed));
        (transform, attack)
    }

    #[test]
    fn a_swing_that_meets_an_arrow_reverses_it() {
        let mut app = projectile_app();
        let (transform, attack) = sword_player((0.07 + 0.16) * 0.5);
        let swing = player_sword_hit_rect(&transform, &attack).expect("hit window");
        let center = Vec2::new(
            (swing.min_x + swing.max_x) * 0.5,
            (swing.min_y + swing.max_y) * 0.5,
        );
        app.world_mut()
            .spawn((DungeonPlayer, transform, attack));
        app.world_mut().spawn((
            EnemyProjectile { damage: 8.0 },
            Transform::from_xyz(center.x, center.y, PROJECTILE_Z),
            ProjectileVelocity(Vec2::new(10.0, 0.0)),
            Sprite::default(),
            DeflectedProjectile::default(),
        ));
        app.world_mut()
            .run_system_once(deflect_projectiles_with_swing)
            .unwrap();
        let (velocity, deflected, sprite) = app
            .world_mut()
            .query::<(&ProjectileVelocity, &DeflectedProjectile, &Sprite)>()
            .single(app.world());
        assert_eq!(velocity.0, Vec2::new(-12.0, 0.0));
        assert!(deflected.active);
        assert_eq!(sprite.color, Color::srgb(1.0, 0.95, 0.55));
    }

    #[test]
    fn an_arrow_already_deflected_keeps_its_velocity() {
        let mut app = projectile_app();
        let (transform, attack) = sword_player((0.07 + 0.16) * 0.5);
        app.world_mut()
            .spawn((DungeonPlayer, transform, attack));
        app.world_mut().spawn((
            EnemyProjectile { damage: 8.0 },
            Transform::from_xyz(400.0, 120.0, PROJECTILE_Z),
            ProjectileVelocity(Vec2::new(10.0, 0.0)),
            Sprite::default(),
            DeflectedProjectile {
                active: true,
                hit_entities: Vec::new(),
            },
        ));
        app.world_mut()
            .run_system_once(deflect_projectiles_with_swing)
            .unwrap();
        let velocity = app
            .world_mut()
            .query::<&ProjectileVelocity>()
            .single(app.world())
            .0;
        assert_eq!(velocity, Vec2::new(10.0, 0.0));
    }

    #[test]
    fn a_swing_does_not_deflect_when_no_hunter_is_present() {
        let mut app = projectile_app();
        app.world_mut().spawn((
            EnemyProjectile { damage: 8.0 },
            Transform::from_xyz(400.0, 120.0, PROJECTILE_Z),
            ProjectileVelocity(Vec2::new(10.0, 0.0)),
            Sprite::default(),
            DeflectedProjectile::default(),
        ));
        app.world_mut()
            .run_system_once(deflect_projectiles_with_swing)
            .unwrap();
        let (velocity, deflected) = app
            .world_mut()
            .query::<(&ProjectileVelocity, &DeflectedProjectile)>()
            .single(app.world());
        assert_eq!(velocity.0, Vec2::new(10.0, 0.0));
        assert!(!deflected.active);
    }

    #[test]
    fn a_deflected_arrow_deals_its_damage_to_the_enemy_it_hits() {
        let mut app = projectile_app();
        app.world_mut().spawn((
            EnemyProjectile { damage: 8.0 },
            Transform::from_xyz(100.0, 80.0, PROJECTILE_Z),
            DeflectedProjectile {
                active: true,
                hit_entities: Vec::new(),
            },
        ));
        app.world_mut().spawn((
            Transform::from_xyz(100.0, 80.0, 0.0),
            EnemyHitbox(Vec2::new(16.0, 16.0)),
            Health::new(20.0),
            Sprite::default(),
        ));
        app.world_mut()
            .run_system_once(resolve_deflected_projectile_hits)
            .unwrap();
        assert_eq!(projectile_count(&mut app), 0);
        let health = app.world_mut().query::<&Health>().single(app.world()).current;
        assert_eq!(health, 12.0);
    }

    #[test]
    fn a_deflected_arrow_misses_an_enemy_standing_apart_from_it() {
        let mut app = projectile_app();
        app.world_mut().spawn((
            EnemyProjectile { damage: 8.0 },
            Transform::from_xyz(100.0, 80.0, PROJECTILE_Z),
            DeflectedProjectile {
                active: true,
                hit_entities: Vec::new(),
            },
        ));
        app.world_mut().spawn((
            Transform::from_xyz(400.0, 80.0, 0.0),
            EnemyHitbox(Vec2::new(16.0, 16.0)),
            Health::new(20.0),
            Sprite::default(),
        ));
        app.world_mut()
            .run_system_once(resolve_deflected_projectile_hits)
            .unwrap();
        assert_eq!(projectile_count(&mut app), 1);
        let health = app.world_mut().query::<&Health>().single(app.world()).current;
        assert_eq!(health, 20.0);
    }

    #[test]
    fn an_arrow_that_was_not_deflected_does_not_damage_an_enemy() {
        let mut app = projectile_app();
        app.world_mut().spawn((
            EnemyProjectile { damage: 8.0 },
            Transform::from_xyz(100.0, 80.0, PROJECTILE_Z),
            DeflectedProjectile::default(),
        ));
        app.world_mut().spawn((
            Transform::from_xyz(100.0, 80.0, 0.0),
            EnemyHitbox(Vec2::new(16.0, 16.0)),
            Health::new(20.0),
            Sprite::default(),
        ));
        app.world_mut()
            .run_system_once(resolve_deflected_projectile_hits)
            .unwrap();
        assert_eq!(projectile_count(&mut app), 1);
        let health = app.world_mut().query::<&Health>().single(app.world()).current;
        assert_eq!(health, 20.0);
    }

    fn spawn_hunter(app: &mut App, health: f32) {
        app.world_mut().spawn((
            DungeonPlayer,
            Transform::from_xyz(300.0, 100.0, 0.0),
            Health::new(health),
            ContactDamageCooldown::default(),
            PlayerBlock::default(),
        ));
    }

    #[test]
    fn an_arrow_that_hits_the_hunter_deals_eight_damage() {
        let mut app = projectile_app();
        app.insert_resource(Loadout::default());
        spawn_hunter(&mut app, 50.0);
        app.world_mut().spawn((
            EnemyProjectile { damage: 8.0 },
            Transform::from_xyz(300.0, 100.0, PROJECTILE_Z),
            ProjectileVelocity(Vec2::new(-30.0, 0.0)),
            Sprite::default(),
            DeflectedProjectile::default(),
        ));
        app.world_mut()
            .run_system_once(resolve_enemy_projectiles)
            .unwrap();
        assert_eq!(projectile_count(&mut app), 0);
        let health = app
            .world_mut()
            .query_filtered::<&Health, With<DungeonPlayer>>()
            .single(app.world())
            .current;
        assert_eq!(health, 42.0);
    }

    #[test]
    fn a_deflected_arrow_does_not_damage_the_hunter() {
        let mut app = projectile_app();
        app.insert_resource(Loadout::default());
        spawn_hunter(&mut app, 50.0);
        app.world_mut().spawn((
            EnemyProjectile { damage: 8.0 },
            Transform::from_xyz(300.0, 100.0, PROJECTILE_Z),
            ProjectileVelocity(Vec2::new(-30.0, 0.0)),
            Sprite::default(),
            DeflectedProjectile {
                active: true,
                hit_entities: Vec::new(),
            },
        ));
        app.world_mut()
            .run_system_once(resolve_enemy_projectiles)
            .unwrap();
        assert_eq!(projectile_count(&mut app), 1);
        let health = app
            .world_mut()
            .query_filtered::<&Health, With<DungeonPlayer>>()
            .single(app.world())
            .current;
        assert_eq!(health, 50.0);
    }

    #[test]
    fn enemy_arrows_do_nothing_when_no_hunter_is_present() {
        let mut app = projectile_app();
        app.insert_resource(Loadout::default());
        app.world_mut().spawn((
            EnemyProjectile { damage: 8.0 },
            Transform::from_xyz(300.0, 100.0, PROJECTILE_Z),
            ProjectileVelocity(Vec2::new(-30.0, 0.0)),
            Sprite::default(),
            DeflectedProjectile::default(),
        ));
        app.world_mut()
            .run_system_once(resolve_enemy_projectiles)
            .unwrap();
        assert_eq!(projectile_count(&mut app), 1);
        let velocity = app
            .world_mut()
            .query::<&ProjectileVelocity>()
            .single(app.world())
            .0;
        assert_eq!(velocity, Vec2::new(-30.0, 0.0));
    }
}
