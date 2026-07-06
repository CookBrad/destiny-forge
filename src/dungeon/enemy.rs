use bevy::prelude::*;

use crate::combat::Health;
use crate::graphics::ENEMY_DISPLAY_SIZE;

const HIT_AGGRO_LOCK_SECS: f32 = 3.0;
const KNOCKBACK_FORCE_X: f32 = 130.0;
const KNOCKBACK_FORCE_Y: f32 = 85.0;

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
    pub fn stats(self) -> super::enemy_stats::EnemyStats {
        super::enemy_stats::EnemyStats::for_kind(self)
    }

    pub fn movement(self) -> EnemyMovement {
        self.stats().movement
    }

    pub fn max_health(self) -> f32 {
        self.stats().max_health
    }

    pub fn contact_damage(self) -> f32 {
        self.stats().contact_damage
    }

    pub fn patrol_speed(self) -> f32 {
        self.stats().patrol_speed
    }

    pub fn chase_speed(self) -> f32 {
        self.stats().chase_speed
    }

    /// Half-width of the idle patrol region in tiles.
    pub fn patrol_radius_tiles(self) -> f32 {
        self.stats().patrol_radius_tiles
    }

    pub fn is_airborne(self) -> bool {
        self.movement() == EnemyMovement::Flying
    }

    pub fn shoots_projectiles(self) -> bool {
        self.stats().shoot_cooldown > 0.0
    }

    pub fn projectile_damage(self) -> f32 {
        self.stats().projectile_damage
    }

    pub fn projectile_speed(self) -> f32 {
        self.stats().projectile_speed
    }

    pub fn shoot_cooldown(self) -> f32 {
        self.stats().shoot_cooldown
    }

    pub fn shoot_range(self) -> f32 {
        self.stats().shoot_range
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

#[derive(Clone, Copy, Debug)]
pub struct PitClearing {
    pub pit_left: f32,
    pub pit_right: f32,
    pub direction: f32,
}

impl PitClearing {
    pub fn cleared(&self, x: f32) -> bool {
        let half = ENEMY_DISPLAY_SIZE.x * 0.5;
        if self.direction > 0.0 {
            x >= self.pit_right - half
        } else {
            x <= self.pit_left + half
        }
    }
}

#[derive(Component, Default)]
pub struct GoblinJump {
    pub velocity_y: f32,
    pub clearing: Option<PitClearing>,
}

impl GoblinJump {
    pub fn is_airborne(&self) -> bool {
        self.velocity_y.abs() > 1.0 || self.clearing.is_some()
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

pub fn track_boss_defeat(
    mut progress: ResMut<DungeonProgress>,
    mut world_progress: ResMut<crate::player::WorldProgress>,
    mut profile_dirty: ResMut<crate::core::ProfileDirty>,
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
        world_progress.record_boss_defeated_floor_1();
        profile_dirty.mark();
        info!("King Slime defeated — ladder exit unlocked.");
    }
}