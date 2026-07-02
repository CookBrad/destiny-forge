use bevy::prelude::*;

use crate::combat::EnemyCorpse;

#[derive(Component)]
pub struct SlimeEnemy;

#[derive(Component)]
pub struct BatEnemy;

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