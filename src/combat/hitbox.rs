use bevy::prelude::*;

use crate::player::Facing;

#[derive(Component)]
pub struct AttackHitbox {
    pub damage: f32,
    pub lifetime: Timer,
    pub facing: Facing,
    pub already_hit: Vec<Entity>,
}

#[derive(Component)]
pub struct AttackCooldown {
    pub timer: Timer,
}

impl Default for AttackCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.35, TimerMode::Once),
        }
    }
}