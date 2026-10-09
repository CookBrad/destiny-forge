use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::combat::Health;
use crate::graphics::ENEMY_DISPLAY_SIZE;

const HIT_AGGRO_LOCK_SECS: f32 = 3.0;
const KNOCKBACK_FORCE_X: f32 = 130.0;
const KNOCKBACK_FORCE_Y: f32 = 85.0;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Logical collision half-extents in world pixels (independent of the sheet size).
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

/// Painted sheet. The hitbox stays [`EnemyHitbox`]; this only places the
/// sprite and lifts the health bar to the painted top.
#[derive(Component, Clone, Copy, Debug)]
pub struct SpriteCanvas {
    pub size: Vec2,
    /// Canvas bottom edge, in pixels below the entity origin.
    pub sole_below_origin: f32,
}

impl SpriteCanvas {
    pub fn grounded(size: Vec2, gameplay_height: f32) -> Self {
        Self {
            size,
            sole_below_origin: gameplay_height * 0.5,
        }
    }

    pub fn centered(size: Vec2) -> Self {
        Self {
            size,
            sole_below_origin: size.y * 0.5,
        }
    }

    pub fn top_above_origin(self) -> f32 {
        self.size.y - self.sole_below_origin
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct EnemyKnockback {
    pub velocity: Vec2,
}

/// How a struck enemy responds to knockback: bosses barely budge, flyers get lifted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KnockbackTarget {
    is_boss: bool,
    is_airborne: bool,
}

impl KnockbackTarget {
    const BOSS_STRENGTH: f32 = 0.35;
    const BOSS_CHARGE_LAUNCH_SPEED: f32 = 780.0;
    /// Decay-integrated travel ≈ speed / KNOCKBACK_DECAY; a full charge dash is ≈ 124px.
    const CHARGE_LAUNCH_SPEED: f32 = 1_020.0;

    pub fn of(kind: Option<&EnemyKind>, boss: Option<&KingSlimeBoss>) -> Self {
        Self {
            is_boss: boss.is_some(),
            is_airborne: kind.is_some_and(|kind| kind.is_airborne()),
        }
    }

    fn strength(self) -> f32 {
        if self.is_boss {
            Self::BOSS_STRENGTH
        } else {
            1.0
        }
    }

    fn charge_launch_speed(self) -> f32 {
        if self.is_boss {
            Self::BOSS_CHARGE_LAUNCH_SPEED
        } else {
            Self::CHARGE_LAUNCH_SPEED
        }
    }

    fn lift(self, force: f32) -> f32 {
        if self.is_airborne {
            force
        } else {
            0.0
        }
    }
}

impl EnemyKnockback {
    pub fn away_from_player(
        player: &Transform,
        enemy: &Transform,
        target: KnockbackTarget,
    ) -> Self {
        let delta = enemy.translation.truncate() - player.translation.truncate();
        let horizontal = if delta.x.abs() > 0.5 {
            delta.x.signum()
        } else if player.scale.x < 0.0 {
            -1.0
        } else {
            1.0
        };

        let strength = target.strength();
        Self {
            velocity: Vec2::new(
                horizontal * KNOCKBACK_FORCE_X * strength,
                target.lift(KNOCKBACK_FORCE_Y * strength),
            ),
        }
    }

    /// Launches enemies farther than a full player charge dash travels.
    pub fn from_charge(direction: f32, target: KnockbackTarget) -> Self {
        Self {
            velocity: Vec2::new(
                direction.signum() * target.charge_launch_speed(),
                target.lift(KNOCKBACK_FORCE_Y * 0.5),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hitboxes_stay_on_the_gameplay_body() {
        assert_eq!(EnemyHitbox::standard().0, ENEMY_DISPLAY_SIZE * 0.5);
        assert_eq!(EnemyHitbox::scaled(2.0).0, Vec2::splat(32.0));
        let skeleton = SpriteCanvas::grounded(Vec2::new(64.0, 144.0), ENEMY_DISPLAY_SIZE.y);
        assert_eq!(skeleton.sole_below_origin, 16.0);
        let bat = SpriteCanvas::centered(Vec2::new(96.0, 48.0));
        assert_eq!(bat.sole_below_origin, 24.0);
        assert_eq!(bat.top_above_origin(), 24.0);
    }

    #[test]
    fn bosses_resist_knockback_and_only_flyers_get_lifted() {
        let player = Transform::from_xyz(0.0, 0.0, 0.0);
        let enemy = Transform::from_xyz(10.0, 0.0, 0.0);
        let goblin = KnockbackTarget::of(Some(&EnemyKind::Goblin), None);
        let bat = KnockbackTarget::of(Some(&EnemyKind::Bat), None);
        let king = KnockbackTarget::of(Some(&EnemyKind::Slime), Some(&KingSlimeBoss));

        let goblin_hit = EnemyKnockback::away_from_player(&player, &enemy, goblin).velocity;
        assert_eq!(goblin_hit, Vec2::new(KNOCKBACK_FORCE_X, 0.0));
        let bat_hit = EnemyKnockback::away_from_player(&player, &enemy, bat).velocity;
        assert_eq!(bat_hit, Vec2::new(KNOCKBACK_FORCE_X, KNOCKBACK_FORCE_Y));
        let king_hit = EnemyKnockback::away_from_player(&player, &enemy, king).velocity;
        assert!(king_hit.x < goblin_hit.x);

        let charged_goblin = EnemyKnockback::from_charge(-1.0, goblin).velocity;
        let charged_king = EnemyKnockback::from_charge(-1.0, king).velocity;
        assert!(charged_goblin.x < charged_king.x && charged_king.x < 0.0);
        assert_eq!(charged_goblin.y, 0.0);
    }
}
