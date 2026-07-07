use bevy::prelude::*;

use rand::Rng;
use std::f32::consts::FRAC_PI_2;

use crate::audio::CombatSfx;
use crate::combat::{
    apply_player_hurt, ContactDamageCooldown, DeflectedProjectile, EnemyCorpse, EnemyProjectile,
    Health, PlayerHitFlash, ProjectileLifetime, ProjectileVelocity,
};
use crate::graphics::{DUNGEON_FLOOR_Y, PIXEL_SCALE, TILE};

use super::enemy::{EnemyAggro, EnemyKnockback, KingSlimeBoss};
use super::movement::DungeonPlayer;
use super::setup::DungeonEntity;
use super::sprites::DungeonArt;

const BOSS_ATTACK_RANGE: f32 = 22.0 * TILE;
const BOSS_REST_MIN: f32 = 2.0;
const BOSS_REST_MAX: f32 = 3.4;

const BOSS_COLOR_IDLE: Color = Color::srgb(0.55, 0.95, 0.45);
const BOSS_COLOR_WINDUP: Color = Color::srgb(1.0, 0.62, 0.18);
const BOSS_COLOR_RELEASE: Color = Color::srgb(0.72, 1.0, 0.55);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BossAttackKind {
    SlimeBolt,
    TripleSpread,
    SlimeRain,
    RingBurst,
    GroundSlam,
    RoyalCharge,
}

impl BossAttackKind {
    fn windup_secs(self) -> f32 {
        match self {
            Self::SlimeBolt => 0.45,
            Self::TripleSpread => 0.65,
            Self::SlimeRain => 0.95,
            Self::RingBurst => 0.75,
            Self::GroundSlam => 1.05,
            Self::RoyalCharge => 0.55,
        }
    }

    fn all() -> [Self; 6] {
        [
            Self::SlimeBolt,
            Self::TripleSpread,
            Self::SlimeRain,
            Self::RingBurst,
            Self::GroundSlam,
            Self::RoyalCharge,
        ]
    }
}

#[derive(Component)]
pub struct BossAttackController {
    pub rest_timer: Timer,
    pub windup_timer: Option<Timer>,
    pub pending: Option<BossAttackKind>,
    pub last_kind: Option<BossAttackKind>,
}

impl BossAttackController {
    pub fn new() -> Self {
        Self {
            rest_timer: Timer::from_seconds(1.8, TimerMode::Once),
            windup_timer: None,
            pending: None,
            last_kind: None,
        }
    }
}

/// Active dash attack toward the player.
#[derive(Component)]
pub struct BossCharging {
    pub velocity: Vec2,
    pub timer: Timer,
}

/// Lingering shockwave left by a ground slam.
#[derive(Component)]
pub struct BossGroundHazard {
    pub damage: f32,
    pub lifetime: Timer,
    pub half_width: f32,
    pub half_height: f32,
}

pub fn tick_boss_attacks(
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    time: Res<Time>,
    art: Res<DungeonArt>,
    player: Query<&Transform, (With<DungeonPlayer>, Without<KingSlimeBoss>)>,
    mut bosses: Query<
        (
            Entity,
            &Transform,
            &Health,
            &mut Sprite,
            &mut BossAttackController,
            Option<&EnemyKnockback>,
            Option<&EnemyAggro>,
            Option<&BossCharging>,
        ),
        (With<KingSlimeBoss>, Without<EnemyCorpse>, Without<DungeonPlayer>),
    >,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (entity, transform, health, mut sprite, mut controller, knockback, aggro, charging) in
        &mut bosses
    {
        if health.is_dead() {
            continue;
        }

        let boss_pos = transform.translation.truncate();
        let to_player = player_pos - boss_pos;
        let distance = to_player.length();

        if distance < BOSS_ATTACK_RANGE && aggro.is_none() {
            commands.entity(entity).insert(EnemyAggro { lock_secs: 0.0 });
        }

        if knockback.is_some() {
            sprite.color = BOSS_COLOR_IDLE;
            continue;
        }

        if charging.is_some() {
            sprite.color = BOSS_COLOR_RELEASE;
            continue;
        }

        if let Some(windup) = controller.windup_timer.as_mut() {
            windup.tick(time.delta());
            sprite.color = BOSS_COLOR_WINDUP;

            if windup.finished() {
                if let Some(kind) = controller.pending.take() {
                    execute_attack(
                        &mut commands,
                        &mut sfx,
                        &art,
                        entity,
                        boss_pos,
                        player_pos,
                        to_player,
                        kind,
                    );
                    controller.last_kind = Some(kind);
                    sprite.color = BOSS_COLOR_RELEASE;
                }
                controller.windup_timer = None;
                controller.rest_timer =
                    Timer::from_seconds(rand::thread_rng().gen_range(BOSS_REST_MIN..=BOSS_REST_MAX), TimerMode::Once);
            }
            continue;
        }

        sprite.color = BOSS_COLOR_IDLE;
        controller.rest_timer.tick(time.delta());

        if !controller.rest_timer.finished() || distance > BOSS_ATTACK_RANGE {
            continue;
        }

        let health_ratio = health.fraction();
        let kind = pick_attack(controller.last_kind, health_ratio);
        controller.pending = Some(kind);
        controller.windup_timer = Some(Timer::from_seconds(kind.windup_secs(), TimerMode::Once));
        controller.rest_timer.reset();
    }
}

pub fn resolve_boss_hazards(
    time: Res<Time>,
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    mut player: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            &mut ContactDamageCooldown,
        ),
        (
            With<DungeonPlayer>,
            Without<BossGroundHazard>,
            Without<PlayerHitFlash>,
        ),
    >,
    mut hazards: Query<(Entity, &Transform, &mut BossGroundHazard), Without<DungeonPlayer>>,
) {
    let Ok((player_entity, player_transform, mut health, mut cooldown)) = player.get_single_mut()
    else {
        return;
    };

    if health.is_dead() {
        return;
    }

    let player_center = player_transform.translation.truncate();
    let player_half = super::player_half_extents();

    for (entity, transform, mut hazard) in &mut hazards {
        hazard.lifetime.tick(time.delta());
        if hazard.lifetime.finished() {
            commands.entity(entity).try_despawn();
            continue;
        }

        let center = transform.translation.truncate();
        let overlaps = (player_center.x - player_half.x) < (center.x + hazard.half_width)
            && (player_center.x + player_half.x) > (center.x - hazard.half_width)
            && (player_center.y - player_half.y) < (center.y + hazard.half_height)
            && (player_center.y + player_half.y) > (center.y - hazard.half_height);

        if overlaps {
            cooldown.0.tick(time.delta());
            if cooldown.0.finished() {
                health.take_damage(hazard.damage);
                apply_player_hurt(
                    &mut commands,
                    player_entity,
                    player_transform,
                    center,
                    1.1,
                );
                sfx.send(CombatSfx::GroundSlam);
                cooldown.0 = Timer::from_seconds(0.5, TimerMode::Once);
            }
        }
    }
}

fn pick_attack(last: Option<BossAttackKind>, health_ratio: f32) -> BossAttackKind {
    let mut rng = rand::thread_rng();
    let candidates: Vec<BossAttackKind> = BossAttackKind::all()
        .into_iter()
        .filter(|kind| Some(*kind) != last)
        .collect();

    let mut pool = if candidates.is_empty() {
        BossAttackKind::all().to_vec()
    } else {
        candidates
    };

    if health_ratio < 0.45 {
        pool.extend([
            BossAttackKind::RingBurst,
            BossAttackKind::SlimeRain,
            BossAttackKind::GroundSlam,
        ]);
    }

    pool[rng.gen_range(0..pool.len())]
}

fn execute_attack(
    commands: &mut Commands,
    sfx: &mut EventWriter<CombatSfx>,
    art: &DungeonArt,
    boss_entity: Entity,
    boss_pos: Vec2,
    player_pos: Vec2,
    to_player: Vec2,
    kind: BossAttackKind,
) {
    match kind {
        BossAttackKind::SlimeBolt => {
            fire_slime_bolt(commands, art, boss_pos, to_player, 11.0, 240.0);
            sfx.send(CombatSfx::SlimeShoot);
        }
        BossAttackKind::TripleSpread => {
            let base = to_player.y.atan2(to_player.x);
            for offset in [-0.38, 0.0, 0.38] {
                let dir = Vec2::new((base + offset).cos(), (base + offset).sin());
                fire_slime_blob(commands, art, boss_pos, dir, 8.0, 200.0, 1.0);
            }
            sfx.send(CombatSfx::SlimeBurst);
        }
        BossAttackKind::SlimeRain => {
            for offset in [-2.5, -1.2, 0.0, 1.2, 2.5] {
                let spawn = Vec2::new(player_pos.x + offset * TILE, player_pos.y + 8.5 * TILE);
                spawn_falling_blob(commands, art, spawn, 7.0);
            }
            sfx.send(CombatSfx::SlimeBurst);
        }
        BossAttackKind::RingBurst => {
            let base = to_player.y.atan2(to_player.x);
            for offset in [-0.72, -0.48, -0.24, 0.0, 0.24, 0.48, 0.72] {
                let dir = Vec2::new((base + offset).cos(), (base + offset).sin());
                fire_slime_blob(commands, art, boss_pos, dir, 5.0, 165.0, 0.85);
            }
            sfx.send(CombatSfx::SlimeBurst);
        }
        BossAttackKind::GroundSlam => {
            spawn_ground_slam(commands, art, player_pos.x);
            sfx.send(CombatSfx::GroundSlam);
        }
        BossAttackKind::RoyalCharge => {
            let dx = to_player.x.signum();
            if dx != 0.0 {
                commands.entity(boss_entity).insert(BossCharging {
                    velocity: Vec2::new(dx * 140.0, 0.0),
                    timer: Timer::from_seconds(0.65, TimerMode::Once),
                });
                sfx.send(CombatSfx::BossCharge);
            }
        }
    }
}

fn fire_slime_bolt(commands: &mut Commands, art: &DungeonArt, origin: Vec2, to_target: Vec2, damage: f32, speed: f32) {
    let dir = to_target.normalize_or_zero();
    if dir == Vec2::ZERO {
        return;
    }
    spawn_projectile(
        commands,
        art.arrow.clone(),
        Color::srgb(0.55, 1.0, 0.45),
        origin + dir * TILE * 0.9,
        dir * speed,
        damage,
        PIXEL_SCALE,
        Vec2::new(3.5, 10.5),
    );
}

fn fire_slime_blob(
    commands: &mut Commands,
    art: &DungeonArt,
    origin: Vec2,
    dir: Vec2,
    damage: f32,
    speed: f32,
    scale: f32,
) {
    if dir == Vec2::ZERO {
        return;
    }
    spawn_projectile(
        commands,
        art.slime.clone(),
        Color::srgb(0.45, 0.95, 0.35),
        origin + dir * TILE * 0.75,
        dir * speed,
        damage,
        PIXEL_SCALE * scale,
        Vec2::new(8.0, 8.0),
    );
}

fn spawn_falling_blob(commands: &mut Commands, art: &DungeonArt, origin: Vec2, damage: f32) {
    spawn_projectile(
        commands,
        art.slime.clone(),
        Color::srgb(0.35, 0.85, 0.95),
        origin,
        Vec2::new(0.0, -210.0),
        damage,
        PIXEL_SCALE * 0.9,
        Vec2::new(7.0, 7.0),
    );
}

fn spawn_projectile(
    commands: &mut Commands,
    image: Handle<Image>,
    color: Color,
    position: Vec2,
    velocity: Vec2,
    damage: f32,
    scale: f32,
    _hit_half: Vec2,
) {
    let angle = velocity.y.atan2(velocity.x) - FRAC_PI_2;

    commands.spawn((
        Sprite {
            image,
            color,
            ..default()
        },
        Transform {
            translation: Vec3::new(position.x, position.y, 4.5),
            rotation: Quat::from_rotation_z(angle),
            scale: Vec3::splat(scale),
            ..default()
        },
        EnemyProjectile { damage },
        ProjectileVelocity(velocity),
        ProjectileLifetime {
            remaining: 4.5,
        },
        DeflectedProjectile::default(),
        DungeonEntity,
    ));
}

fn spawn_ground_slam(commands: &mut Commands, art: &DungeonArt, target_x: f32) {
    let half_height = TILE * 0.75;
    let y = DUNGEON_FLOOR_Y + half_height;

    commands.spawn((
        Sprite {
            image: art.floor_platform.clone(),
            color: Color::srgba(0.95, 0.25, 0.15, 0.7),
            ..default()
        },
        Transform {
            translation: Vec3::new(target_x, y, 2.0),
            scale: Vec3::new(PIXEL_SCALE * 3.2, PIXEL_SCALE * 0.55, 1.0),
            ..default()
        },
        BossGroundHazard {
            damage: 16.0,
            lifetime: Timer::from_seconds(1.35, TimerMode::Once),
            half_width: TILE * 2.8,
            half_height,
        },
        DungeonEntity,
    ));
}