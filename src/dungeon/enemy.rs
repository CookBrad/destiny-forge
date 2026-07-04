use bevy::prelude::*;

use crate::combat::{EnemyCorpse, Health};
use super::boss::{BossAttackController, BossCharging};
use super::level::{
    constrain_ground_walk, horizontal_move_crosses_pit, is_on_ground_floor, pit_jump_landing_exists,
    DungeonLayout,
};
use super::movement::DungeonPlayer;
use crate::graphics::{DUNGEON_FLOOR_Y, ENEMY_DISPLAY_SIZE, TILE};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    Slime,
    Bat,
    Goblin,
    Skeleton,
    Zombie,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyMovement {
    Ground,
    Flying,
}

impl EnemyKind {
    pub fn movement(self) -> EnemyMovement {
        match self {
            Self::Bat => EnemyMovement::Flying,
            Self::Slime | Self::Goblin | Self::Skeleton | Self::Zombie => EnemyMovement::Ground,
        }
    }

    pub fn max_health(self) -> f32 {
        match self {
            Self::Slime => 30.0,
            Self::Bat => 20.0,
            Self::Goblin => 22.0,
            Self::Skeleton => 35.0,
            Self::Zombie => 45.0,
        }
    }

    pub fn contact_damage(self) -> f32 {
        match self {
            Self::Slime => 8.0,
            Self::Bat => 6.0,
            Self::Goblin => 10.0,
            Self::Skeleton => 9.0,
            Self::Zombie => 7.0,
        }
    }

    pub fn patrol_speed(self) -> f32 {
        match self {
            Self::Slime => 35.0,
            Self::Bat => 50.0,
            Self::Goblin => 45.0,
            Self::Skeleton => 28.0,
            Self::Zombie => 20.0,
        }
    }

    pub fn chase_speed(self) -> f32 {
        match self {
            Self::Slime => 55.0,
            Self::Bat => 72.0,
            Self::Goblin => 78.0,
            Self::Skeleton => 48.0,
            Self::Zombie => 38.0,
        }
    }

    /// Half-width of the idle patrol region in tiles.
    pub fn patrol_radius_tiles(self) -> f32 {
        match self {
            Self::Slime => 2.0,
            Self::Bat => 1.0,
            Self::Goblin => 3.0,
            Self::Skeleton => 2.5,
            Self::Zombie => 1.5,
        }
    }

    pub fn is_airborne(self) -> bool {
        self.movement() == EnemyMovement::Flying
    }

    pub fn shoots_projectiles(self) -> bool {
        matches!(self, Self::Skeleton | Self::Bat | Self::Goblin)
    }

    pub fn projectile_damage(self) -> f32 {
        match self {
            Self::Goblin => 6.0,
            Self::Skeleton => 8.0,
            Self::Bat => 5.0,
            Self::Slime | Self::Zombie => 0.0,
        }
    }

    pub fn projectile_speed(self) -> f32 {
        match self {
            Self::Goblin => 190.0,
            Self::Skeleton => 220.0,
            Self::Bat => 165.0,
            Self::Slime | Self::Zombie => 0.0,
        }
    }

    pub fn shoot_cooldown(self) -> f32 {
        match self {
            Self::Goblin => 2.4,
            Self::Skeleton => 1.9,
            Self::Bat => 2.1,
            Self::Slime | Self::Zombie => 0.0,
        }
    }

    pub fn shoot_range(self) -> f32 {
        match self {
            Self::Goblin => 9.0 * TILE,
            Self::Skeleton => 14.0 * TILE,
            Self::Bat => 11.0 * TILE,
            Self::Slime | Self::Zombie => 0.0,
        }
    }
}

/// Fired by ranged enemies; timer ticks down between shots.
#[derive(Component)]
pub struct EnemyShootCooldown(pub Timer);

#[derive(Component)]
pub struct KingSlimeBoss;

/// Touch damage dealt to the player on overlap.
#[derive(Component, Clone, Copy)]
pub struct EnemyContactDamage(pub f32);

/// Set while the enemy is actively pursuing the player.
#[derive(Component)]
pub struct EnemyAggro {
    /// While > 0, the enemy keeps chasing even if the player moves out of range.
    pub lock_secs: f32,
}

impl EnemyAggro {
    pub fn from_hit() -> Self {
        Self {
            lock_secs: HIT_AGGRO_LOCK_SECS,
        }
    }
}

/// Logical collision half-extents in native sprite pixels (independent of transform scale).
#[derive(Component, Clone, Copy)]
pub struct EnemyHitbox(pub Vec2);

impl EnemyHitbox {
    pub fn standard() -> Self {
        Self(ENEMY_DISPLAY_SIZE * 0.5)
    }

    pub fn scaled(multiplier: f32) -> Self {
        Self(ENEMY_DISPLAY_SIZE * 0.5 * multiplier)
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct EnemyKnockback {
    pub velocity: Vec2,
}

impl EnemyKnockback {
    pub fn away_from_player(player: &Transform, enemy: &Transform, strength: f32, airborne: bool) -> Self {
        let delta = enemy.translation.truncate() - player.translation.truncate();
        let horizontal = if delta.x.abs() > 0.5 {
            delta.x.signum()
        } else if player.scale.x < 0.0 {
            -1.0
        } else {
            1.0
        };

        Self::in_direction(horizontal, strength, airborne)
    }

    pub fn in_direction(direction: f32, strength: f32, airborne: bool) -> Self {
        let dir = direction.signum();
        Self {
            velocity: Vec2::new(
                dir * KNOCKBACK_FORCE_X * strength,
                if airborne {
                    KNOCKBACK_FORCE_Y * strength
                } else {
                    0.0
                },
            ),
        }
    }

    /// Launches enemies farther than a full player charge dash travels.
    pub fn from_charge(direction: f32, is_boss: bool, airborne: bool) -> Self {
        let dir = direction.signum();
        // Decay-integrated travel ≈ speed / KNOCKBACK_DECAY; charge dash ≈ 124px.
        let speed = if is_boss { 780.0 } else { 1_020.0 };
        Self {
            velocity: Vec2::new(
                dir * speed,
                if airborne {
                    KNOCKBACK_FORCE_Y * 0.5
                } else {
                    0.0
                },
            ),
        }
    }
}

#[derive(Resource, Default)]
pub struct DungeonProgress {
    pub boss_defeated: bool,
}

#[derive(Component, Default)]
pub struct GoblinJump {
    pub velocity_y: f32,
}

impl GoblinJump {
    pub fn is_airborne(&self) -> bool {
        self.velocity_y.abs() > 1.0
    }
}

#[derive(Component)]
pub struct Patrol {
    pub min_x: f32,
    pub max_x: f32,
    pub speed: f32,
    pub direction: f32,
}

impl Patrol {
    pub fn between(min_x: f32, max_x: f32, speed: f32) -> Self {
        Self {
            min_x,
            max_x,
            speed,
            direction: -1.0,
        }
    }
}

const AGGRO_RANGE: f32 = 20.0 * TILE;
const DEAGGRO_RANGE: f32 = 26.0 * TILE;
const HIT_AGGRO_LOCK_SECS: f32 = 3.0;
const BOSS_CHASE_SPEED: f32 = 34.0;
const KNOCKBACK_FORCE_X: f32 = 130.0;
const KNOCKBACK_FORCE_Y: f32 = 85.0;
const KNOCKBACK_DECAY: f32 = 6.0;
const KNOCKBACK_GRAVITY: f32 = -360.0;
const KNOCKBACK_STOP_SPEED: f32 = 18.0;
const GOBLIN_JUMP_SPEED: f32 = 400.0;
const GOBLIN_JUMP_HSPEED: f32 = 115.0;
const GOBLIN_GRAVITY: f32 = -740.0;

pub fn move_enemies(
    time: Res<Time>,
    layout: Res<DungeonLayout>,
    mut commands: Commands,
    player: Query<&Transform, With<DungeonPlayer>>,
    mut enemies: Query<
        (
            Entity,
            &mut Transform,
            &mut Patrol,
            Option<&mut EnemyKnockback>,
            Option<&mut EnemyAggro>,
            Option<&EnemyKind>,
            Option<&mut GoblinJump>,
            Option<&KingSlimeBoss>,
            Option<&BossAttackController>,
            Option<&mut BossCharging>,
        ),
        (Without<EnemyCorpse>, Without<DungeonPlayer>),
    >,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let dt = time.delta_secs();

    let segments = &layout.floor.ground_segments;
    let pitfalls = &layout.floor.pitfalls;

    for (
        entity,
        mut transform,
        mut patrol,
        mut knockback,
        mut aggro,
        kind,
        mut goblin_jump,
        boss,
        attack_ctrl,
        mut charge,
    ) in &mut enemies
    {
        let enemy_pos = transform.translation.truncate();
        let to_player = player_pos - enemy_pos;
        let distance = to_player.length();
        let airborne = kind.is_some_and(|kind| kind.is_airborne());

        let is_aggro = update_aggro(&mut commands, entity, aggro.as_deref_mut(), distance, dt);
        let mut velocity = Vec2::ZERO;
        let mut under_knockback = false;

        if let Some(knockback) = knockback.as_mut() {
            let knockback_active = if airborne {
                knockback.velocity.length() > KNOCKBACK_STOP_SPEED
            } else {
                knockback.velocity.x.abs() > KNOCKBACK_STOP_SPEED
            };

            if knockback_active {
                velocity = knockback.velocity;
                under_knockback = true;
                let decay = (-KNOCKBACK_DECAY * dt).exp();
                knockback.velocity.x *= decay;
                if airborne {
                    knockback.velocity.y *= decay;
                    knockback.velocity.y += KNOCKBACK_GRAVITY * dt;
                } else {
                    knockback.velocity.y = 0.0;
                }

                let still_active = if airborne {
                    knockback.velocity.length() > KNOCKBACK_STOP_SPEED
                } else {
                    knockback.velocity.x.abs() > KNOCKBACK_STOP_SPEED
                };
                if !still_active {
                    commands.entity(entity).remove::<EnemyKnockback>();
                }
            } else {
                commands.entity(entity).remove::<EnemyKnockback>();
            }
        }

        let boss_winding_up = boss.is_some() && attack_ctrl.is_some_and(|c| c.windup_timer.is_some());

        let mut charging = false;
        if let Some(charge) = charge.as_mut() {
            charging = true;
            charge.timer.tick(time.delta());
            velocity = charge.velocity;
            if charge.timer.finished() {
                commands.entity(entity).remove::<BossCharging>();
                charging = false;
            }
        } else if !under_knockback && !boss_winding_up && is_aggro {
            let chase_speed = if boss.is_some() {
                BOSS_CHASE_SPEED
            } else {
                kind.map(|kind| kind.chase_speed()).unwrap_or(BOSS_CHASE_SPEED)
            };

            if airborne {
                velocity = to_player.normalize_or_zero() * chase_speed;
            } else {
                velocity.x = to_player.x.signum() * chase_speed;
            }
        } else if !under_knockback && !boss_winding_up {
            velocity.x = patrol.direction * patrol.speed;
        }

        let is_goblin = kind.is_some_and(|kind| *kind == EnemyKind::Goblin);
        let goblin_airborne = goblin_jump.as_ref().is_some_and(|jump| jump.is_airborne())
            || transform.translation.y > DUNGEON_FLOOR_Y + ENEMY_DISPLAY_SIZE.y * 0.5 + 0.5;

        if is_goblin && !under_knockback && !charging && !goblin_airborne {
            let jump_direction = if is_aggro {
                to_player.x.signum()
            } else {
                patrol.direction
            };
            if jump_direction != 0.0
                && pit_jump_landing_exists(transform.translation.x, jump_direction, pitfalls, segments)
            {
                let probe_dx = jump_direction * TILE * 0.5;
                let (_, hit_edge) =
                    constrain_ground_walk(transform.translation.x, probe_dx, segments);
                if hit_edge || horizontal_move_crosses_pit(
                    transform.translation.x,
                    transform.translation.x + jump_direction * TILE,
                    pitfalls,
                ) {
                    if let Some(jump) = goblin_jump.as_mut() {
                        jump.velocity_y = GOBLIN_JUMP_SPEED;
                    }
                    velocity.x = jump_direction * GOBLIN_JUMP_HSPEED;
                }
            }
        }

        let dx = velocity.x * dt;
        if is_goblin && goblin_airborne {
            if let Some(jump) = goblin_jump.as_mut() {
                jump.velocity_y += GOBLIN_GRAVITY * dt;
                transform.translation.y += jump.velocity_y * dt;

                let half = ENEMY_DISPLAY_SIZE.y * 0.5;
                let floor_y = DUNGEON_FLOOR_Y + half;
                if jump.velocity_y <= 0.0
                    && transform.translation.y <= floor_y
                    && is_on_ground_floor(transform.translation.x, segments)
                {
                    transform.translation.y = floor_y;
                    jump.velocity_y = 0.0;
                }
            }
            transform.translation.x += dx;
        } else if !airborne && boss.is_none() {
            let (new_x, hit_edge) = constrain_ground_walk(transform.translation.x, dx, segments);
            transform.translation.x = new_x;
            if hit_edge && !is_aggro && !charging {
                patrol.direction = -patrol.direction;
            }
            transform.translation.y += velocity.y * dt;
        } else {
            transform.translation.x += dx;
            transform.translation.y += velocity.y * dt;
        }

        if charging {
            let hit_left = transform.translation.x <= patrol.min_x;
            let hit_right = transform.translation.x >= patrol.max_x;
            if hit_left || hit_right {
                transform.translation.x = transform.translation.x.clamp(patrol.min_x, patrol.max_x);
                commands.entity(entity).remove::<BossCharging>();
            }
        }

        if !airborne
            && !goblin_airborne
            && is_on_ground_floor(transform.translation.x, segments)
        {
            let half = ENEMY_DISPLAY_SIZE.y * 0.5;
            let floor_y = DUNGEON_FLOOR_Y + half;
            if transform.translation.y < floor_y {
                transform.translation.y = floor_y;
            }
        }

        if !charging && !airborne && !goblin_airborne && !is_aggro && boss.is_none() {
            if transform.translation.x <= patrol.min_x {
                transform.translation.x = patrol.min_x;
                patrol.direction = 1.0;
            } else if transform.translation.x >= patrol.max_x {
                transform.translation.x = patrol.max_x;
                patrol.direction = -1.0;
            }
        }
    }
}

fn update_aggro(
    commands: &mut Commands,
    entity: Entity,
    aggro: Option<&mut EnemyAggro>,
    distance: f32,
    dt: f32,
) -> bool {
    if let Some(aggro) = aggro {
        aggro.lock_secs = (aggro.lock_secs - dt).max(0.0);
        if distance > DEAGGRO_RANGE && aggro.lock_secs <= 0.0 {
            commands.entity(entity).remove::<EnemyAggro>();
            return distance < AGGRO_RANGE;
        }
        return true;
    }

    if distance < AGGRO_RANGE {
        commands.entity(entity).insert(EnemyAggro { lock_secs: 0.0 });
        return true;
    }

    false
}

pub fn track_boss_defeat(
    mut progress: ResMut<DungeonProgress>,
    bosses: Query<&Health, With<KingSlimeBoss>>,
) {
    if progress.boss_defeated {
        return;
    }

    let Some(boss) = bosses.iter().next() else {
        return;
    };

    if boss.is_dead() {
        progress.boss_defeated = true;
        info!("King Slime defeated — ladder exit unlocked.");
    }
}