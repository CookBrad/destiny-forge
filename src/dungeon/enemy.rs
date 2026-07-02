use bevy::prelude::*;

use crate::combat::{EnemyCorpse, Health};
use crate::graphics::ENEMY_DISPLAY_SIZE;

#[derive(Component)]
pub struct SlimeEnemy;

#[derive(Component)]
pub struct BatEnemy;

#[derive(Component)]
pub struct KingSlimeBoss;

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

#[derive(Resource, Default)]
pub struct DungeonProgress {
    pub boss_defeated: bool,
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

pub fn patrol_enemies(
    time: Res<Time>,
    mut enemies: Query<(&mut Transform, &mut Patrol), Without<EnemyCorpse>>,
) {
    for (mut transform, mut patrol) in &mut enemies {
        transform.translation.x += patrol.direction * patrol.speed * time.delta_secs();

        if transform.translation.x <= patrol.min_x {
            transform.translation.x = patrol.min_x;
            patrol.direction = 1.0;
        } else if transform.translation.x >= patrol.max_x {
            transform.translation.x = patrol.max_x;
            patrol.direction = -1.0;
        }
    }
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