use bevy::prelude::*;

use rand::Rng;
use std::f32::consts::FRAC_PI_2;

use crate::audio::CombatSfx;
use crate::combat::{
    apply_player_hurt, damage_amount, ContactDamageCooldown, DeflectedProjectile, EnemyCorpse,
    EnemyProjectile, Health, PlayerHitFlash, ProjectileLifetime, ProjectileVelocity,
};
use crate::graphics::{solid_fill, world_transform, DUNGEON_FLOOR_Y, ENEMY_DISPLAY_SIZE, TILE};
use crate::player::Loadout;

use super::enemy::{EnemyAggro, EnemyKnockback, KingSlimeBoss};
use super::movement::DungeonPlayer;
use super::setup::DungeonEntity;
use super::sprites::DungeonArt;

const BOSS_ATTACK_RANGE: f32 = 22.0 * TILE;
/// Hurtbox of `ground_slam.png` (96×24). The 1.35 is seconds, not a scale.
const GROUND_SLAM_HALF_WIDTH: f32 = 48.0;
const GROUND_SLAM_HALF_HEIGHT: f32 = 12.0;
const GROUND_SLAM_LIFETIME_SECS: f32 = 1.35;

/// Untinted, so the painted sheet shows. The old green was the placeholder fill.
const BOSS_COLOR_IDLE: Color = Color::WHITE;
const BOSS_COLOR_RELEASE: Color = Color::WHITE;

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
    /// 1 = openers only, 2 = mid patterns, 3 = all + aggressive rest.
    pub phase: u8,
}

impl BossAttackController {
    pub fn new() -> Self {
        Self {
            rest_timer: Timer::from_seconds(1.8, TimerMode::Once),
            windup_timer: None,
            pending: None,
            last_kind: None,
            phase: 1,
        }
    }
}

/// Brief white flash when King Slime changes phase.
#[derive(Component)]
pub struct BossPhaseFlash {
    pub timer: Timer,
}

fn phase_for_health(health_ratio: f32) -> u8 {
    if health_ratio > 0.66 {
        1
    } else if health_ratio > 0.33 {
        2
    } else {
        3
    }
}

fn rest_range_for_phase(phase: u8) -> (f32, f32) {
    match phase {
        1 => (2.2, 3.6),
        2 => (1.7, 2.9),
        _ => (1.15, 2.1),
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
        (
            With<KingSlimeBoss>,
            Without<EnemyCorpse>,
            Without<DungeonPlayer>,
        ),
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

        let health_ratio = health.fraction();
        let new_phase = phase_for_health(health_ratio);
        if new_phase > controller.phase {
            controller.phase = new_phase;
            commands.entity(entity).insert(BossPhaseFlash {
                timer: Timer::from_seconds(0.35, TimerMode::Once),
            });
            sfx.send(CombatSfx::BossCharge);
        }

        let boss_pos = transform.translation.truncate();
        let to_player = player_pos - boss_pos;
        let distance = to_player.length();

        if distance < BOSS_ATTACK_RANGE && aggro.is_none() {
            commands
                .entity(entity)
                .insert(EnemyAggro { lock_secs: 0.0 });
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
            // Stronger telegraph: pulse orange during windup.
            let pulse = ((time.elapsed_secs() * 10.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
            sprite.color = Color::srgb(1.0, 0.45 + 0.25 * pulse, 0.12 + 0.1 * pulse);

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
                let (lo, hi) = rest_range_for_phase(controller.phase);
                controller.rest_timer =
                    Timer::from_seconds(rand::thread_rng().gen_range(lo..=hi), TimerMode::Once);
            }
            continue;
        }

        sprite.color = BOSS_COLOR_IDLE;
        controller.rest_timer.tick(time.delta());

        if !controller.rest_timer.finished() || distance > BOSS_ATTACK_RANGE {
            continue;
        }

        let kind = pick_attack(controller.last_kind, controller.phase);
        controller.pending = Some(kind);
        // Phase 3 winds up slightly faster for pressure.
        let windup = if controller.phase >= 3 {
            kind.windup_secs() * 0.85
        } else {
            kind.windup_secs()
        };
        controller.windup_timer = Some(Timer::from_seconds(windup, TimerMode::Once));
        controller.rest_timer.reset();
    }
}

pub fn tick_boss_phase_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut flashes: Query<(Entity, &mut BossPhaseFlash, &mut Sprite), With<KingSlimeBoss>>,
) {
    for (entity, mut flash, mut sprite) in &mut flashes {
        flash.timer.tick(time.delta());
        // White would match the untinted sheet, so the phase pop stays visible.
        sprite.color = Color::srgb(0.78, 0.45, 1.0);
        if flash.timer.finished() {
            sprite.color = BOSS_COLOR_IDLE;
            commands.entity(entity).remove::<BossPhaseFlash>();
        }
    }
}

pub fn resolve_boss_hazards(
    time: Res<Time>,
    loadout: Res<Loadout>,
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    mut player: Query<
        (Entity, &Transform, &mut Health, &mut ContactDamageCooldown),
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
                health.take_damage(damage_amount(hazard.damage, loadout.total_defense()));
                apply_player_hurt(
                    &mut commands,
                    player_entity,
                    player_transform,
                    center,
                    1.1,
                    loadout.knockback_resist(),
                );
                sfx.send(CombatSfx::GroundSlam);
                cooldown.0 = Timer::from_seconds(0.5, TimerMode::Once);
            }
        }
    }
}

fn pick_attack(last: Option<BossAttackKind>, phase: u8) -> BossAttackKind {
    let mut rng = rand::thread_rng();

    let phase_pool: &[BossAttackKind] = match phase {
        1 => &[BossAttackKind::SlimeBolt, BossAttackKind::TripleSpread],
        2 => &[
            BossAttackKind::SlimeBolt,
            BossAttackKind::TripleSpread,
            BossAttackKind::SlimeRain,
            BossAttackKind::RingBurst,
            BossAttackKind::GroundSlam,
        ],
        _ => &[
            BossAttackKind::SlimeBolt,
            BossAttackKind::TripleSpread,
            BossAttackKind::SlimeRain,
            BossAttackKind::RingBurst,
            BossAttackKind::GroundSlam,
            BossAttackKind::RoyalCharge,
        ],
    };

    let mut pool: Vec<BossAttackKind> = phase_pool
        .iter()
        .copied()
        .filter(|kind| Some(*kind) != last)
        .collect();

    if pool.is_empty() {
        pool = phase_pool.to_vec();
    }

    // Weight heavier patterns more in late phases.
    if phase >= 3 {
        pool.extend([
            BossAttackKind::GroundSlam,
            BossAttackKind::RoyalCharge,
            BossAttackKind::SlimeRain,
        ]);
    } else if phase == 2 {
        pool.extend([BossAttackKind::GroundSlam, BossAttackKind::RingBurst]);
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
                fire_slime_blob(
                    commands,
                    art,
                    boss_pos,
                    dir,
                    8.0,
                    200.0,
                    ENEMY_DISPLAY_SIZE.x,
                );
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
                fire_slime_blob(
                    commands,
                    art,
                    boss_pos,
                    dir,
                    5.0,
                    165.0,
                    ENEMY_DISPLAY_SIZE.x * 0.85,
                );
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

fn fire_slime_bolt(
    commands: &mut Commands,
    art: &DungeonArt,
    origin: Vec2,
    to_target: Vec2,
    damage: f32,
    speed: f32,
) {
    let dir = to_target.normalize_or_zero();
    if dir == Vec2::ZERO {
        return;
    }
    spawn_projectile(
        commands,
        Sprite {
            image: art.slime_bolt.clone(),
            color: Color::WHITE,
            ..default()
        },
        origin + dir * TILE * 0.9,
        dir * speed,
        damage,
    );
}

fn fire_slime_blob(
    commands: &mut Commands,
    art: &DungeonArt,
    origin: Vec2,
    dir: Vec2,
    damage: f32,
    speed: f32,
    diameter: f32,
) {
    if dir == Vec2::ZERO {
        return;
    }
    spawn_blob(
        commands,
        art,
        Color::srgb(0.45, 0.95, 0.35),
        origin + dir * TILE * 0.75,
        dir * speed,
        damage,
        diameter,
    );
}

fn spawn_falling_blob(commands: &mut Commands, art: &DungeonArt, origin: Vec2, damage: f32) {
    spawn_blob(
        commands,
        art,
        Color::srgb(0.35, 0.85, 0.95),
        origin,
        Vec2::new(0.0, -210.0),
        damage,
        ENEMY_DISPLAY_SIZE.x * 0.9,
    );
}

/// A one-tile blob uses the 32×32 sheet at 1×. Any other diameter stays a solid fill.
/// The sheet is not fitted to those other diameters.
fn spawn_blob(
    commands: &mut Commands,
    art: &DungeonArt,
    color: Color,
    position: Vec2,
    velocity: Vec2,
    damage: f32,
    diameter: f32,
) {
    let native = (diameter - ENEMY_DISPLAY_SIZE.x).abs() < 0.01;
    let sprite = if native {
        Sprite {
            image: art.slime_blob.clone(),
            color: Color::WHITE,
            ..default()
        }
    } else {
        // Off-size diameters stay a 1×1 fill. The 32×32 sheet is not scaled to fit them.
        solid_fill(art.fill.clone(), Vec2::splat(diameter), color)
    };
    spawn_projectile(commands, sprite, position, velocity, damage);
}

fn spawn_projectile(
    commands: &mut Commands,
    sprite: Sprite,
    position: Vec2,
    velocity: Vec2,
    damage: f32,
) {
    let angle = velocity.y.atan2(velocity.x) - FRAC_PI_2;

    commands.spawn((
        sprite,
        Transform {
            translation: Vec3::new(position.x, position.y, 4.5),
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        EnemyProjectile { damage },
        ProjectileVelocity(velocity),
        ProjectileLifetime { remaining: 4.5 },
        DeflectedProjectile::default(),
        DungeonEntity,
    ));
}

fn spawn_ground_slam(commands: &mut Commands, art: &DungeonArt, target_x: f32) {
    let y = DUNGEON_FLOOR_Y + GROUND_SLAM_HALF_HEIGHT;

    commands.spawn((
        Sprite {
            image: art.ground_slam.clone(),
            ..default()
        },
        world_transform(Vec2::new(target_x, y), 2.0),
        BossGroundHazard {
            damage: 16.0,
            lifetime: Timer::from_seconds(GROUND_SLAM_LIFETIME_SECS, TimerMode::Once),
            half_width: GROUND_SLAM_HALF_WIDTH,
            half_height: GROUND_SLAM_HALF_HEIGHT,
        },
        DungeonEntity,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ground_slam_hurtbox_matches_the_authored_sprite() {
        assert_eq!(GROUND_SLAM_HALF_WIDTH * 2.0, 96.0);
        assert_eq!(GROUND_SLAM_HALF_HEIGHT * 2.0, 24.0);
        assert_eq!(GROUND_SLAM_LIFETIME_SECS, 1.35);
    }

    #[test]
    fn boss_attack_range_stays_twenty_two_tiles() {
        assert_eq!(BOSS_ATTACK_RANGE, 22.0 * TILE);
    }
}
