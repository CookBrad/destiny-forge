use crate::graphics::TILE;

use super::enemy::{EnemyKind, EnemyMovement};

#[derive(Clone, Copy, Debug)]
pub struct EnemyStats {
    pub movement: EnemyMovement,
    pub max_health: f32,
    pub contact_damage: f32,
    pub patrol_speed: f32,
    pub chase_speed: f32,
    pub patrol_radius_tiles: f32,
    pub projectile_damage: f32,
    pub projectile_speed: f32,
    pub shoot_cooldown: f32,
    pub shoot_range: f32,
}

impl EnemyStats {
    pub const fn for_kind(kind: EnemyKind) -> Self {
        match kind {
            EnemyKind::Slime => SLIME,
            EnemyKind::Bat => BAT,
            EnemyKind::Goblin => GOBLIN,
            EnemyKind::Skeleton => SKELETON,
            EnemyKind::Zombie => ZOMBIE,
        }
    }
}

const SLIME: EnemyStats = EnemyStats {
    movement: EnemyMovement::Ground,
    max_health: 30.0,
    contact_damage: 8.0,
    patrol_speed: 140.0,
    chase_speed: 220.0,
    patrol_radius_tiles: 2.0,
    projectile_damage: 0.0,
    projectile_speed: 0.0,
    shoot_cooldown: 0.0,
    shoot_range: 0.0,
};

const BAT: EnemyStats = EnemyStats {
    movement: EnemyMovement::Flying,
    max_health: 20.0,
    contact_damage: 6.0,
    patrol_speed: 200.0,
    chase_speed: 288.0,
    patrol_radius_tiles: 1.0,
    projectile_damage: 5.0,
    projectile_speed: 660.0,
    shoot_cooldown: 2.1,
    shoot_range: 11.0 * TILE,
};

const GOBLIN: EnemyStats = EnemyStats {
    movement: EnemyMovement::Ground,
    max_health: 22.0,
    contact_damage: 10.0,
    patrol_speed: 180.0,
    chase_speed: 312.0,
    patrol_radius_tiles: 3.0,
    projectile_damage: 6.0,
    projectile_speed: 760.0,
    shoot_cooldown: 2.4,
    shoot_range: 9.0 * TILE,
};

const SKELETON: EnemyStats = EnemyStats {
    movement: EnemyMovement::Ground,
    max_health: 35.0,
    contact_damage: 9.0,
    patrol_speed: 112.0,
    chase_speed: 192.0,
    patrol_radius_tiles: 2.5,
    projectile_damage: 8.0,
    projectile_speed: 880.0,
    shoot_cooldown: 1.9,
    shoot_range: 14.0 * TILE,
};

const ZOMBIE: EnemyStats = EnemyStats {
    movement: EnemyMovement::Ground,
    max_health: 45.0,
    contact_damage: 7.0,
    patrol_speed: 80.0,
    chase_speed: 152.0,
    patrol_radius_tiles: 1.5,
    projectile_damage: 0.0,
    projectile_speed: 0.0,
    shoot_cooldown: 0.0,
    shoot_range: 0.0,
};