use bevy::prelude::*;

use crate::audio::CombatSfx;
use crate::dungeon::{EnemyAggro, EnemyKnockback, Patrol};

use super::attack::{EnemyCorpse, HitFlash};
use super::health::Health;

pub struct EnemyStrike {
    pub damage: f32,
    pub sfx: CombatSfx,
    pub knockback: EnemyKnockback,
}

pub fn apply_enemy_strike(
    commands: &mut Commands,
    sfx: &mut EventWriter<CombatSfx>,
    entity: Entity,
    health: &mut Health,
    sprite: &mut Sprite,
    hit_entities: &mut Vec<Entity>,
    strike: EnemyStrike,
) {
    health.take_damage(strike.damage);
    hit_entities.push(entity);
    sfx.send(strike.sfx);

    sprite.color = Color::srgb(1.0, 0.45, 0.45);
    commands.entity(entity).insert((
        EnemyAggro::from_hit(),
        HitFlash {
            timer: Timer::from_seconds(0.12, TimerMode::Once),
        },
        strike.knockback,
    ));

    if health.is_dead() {
        commands.entity(entity).remove::<Patrol>();
        commands.entity(entity).insert(EnemyCorpse);
        sprite.color = Color::srgba(0.55, 0.55, 0.6, 0.85);
    }
}