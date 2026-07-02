use bevy::prelude::*;

use crate::dungeon::Patrol;
use crate::dungeon::DungeonPlayer;
use crate::graphics::{enemy_half_extents, player_half_extents};

use super::health::{damage_amount, Health};
use super::weapon::{EquippedWeapon, WeaponKind, WeaponStats};

#[derive(Component)]
pub struct PlayerAttack {
    pub timer: Timer,
    pub weapon: WeaponKind,
    pub hit_entities: Vec<Entity>,
}

impl PlayerAttack {
    pub fn inactive() -> Self {
        Self {
            timer: Timer::from_seconds(0.0, TimerMode::Once),
            weapon: WeaponKind::RustySword,
            hit_entities: Vec::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        !self.timer.finished()
    }

    pub fn in_hit_window(&self) -> bool {
        if !self.is_active() {
            return false;
        }
        let elapsed = self.timer.elapsed_secs();
        let stats = self.weapon.stats();
        elapsed >= stats.hit_start && elapsed <= stats.hit_end
    }
}

#[derive(Component)]
pub struct EnemyCorpse;

#[derive(Component)]
pub struct HitFlash {
    pub timer: Timer,
}

pub fn start_player_attack(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&EquippedWeapon, &mut PlayerAttack), With<DungeonPlayer>>,
) {
    let Ok((weapon, mut attack)) = player.get_single_mut() else {
        return;
    };

    if !keyboard.just_pressed(KeyCode::Digit1) || attack.is_active() {
        return;
    }

    let stats = weapon.0.stats();
    attack.weapon = weapon.0;
    attack.hit_entities.clear();
    attack.timer = Timer::from_seconds(stats.swing_secs, TimerMode::Once);
    attack.timer.reset();
}

pub fn tick_player_attack(time: Res<Time>, mut attacks: Query<&mut PlayerAttack, With<DungeonPlayer>>) {
    let Ok(mut attack) = attacks.get_single_mut() else {
        return;
    };

    if attack.is_active() {
        attack.timer.tick(time.delta());
    }
}

pub fn resolve_weapon_hits(
    mut commands: Commands,
    mut player: Query<(&Transform, &mut PlayerAttack), With<DungeonPlayer>>,
    mut enemies: Query<
        (Entity, &Transform, &mut Health, &mut Sprite),
        (With<Health>, Without<DungeonPlayer>, Without<EnemyCorpse>),
    >,
) {
    let Ok((player_transform, mut attack)) = player.get_single_mut() else {
        return;
    };

    if !attack.in_hit_window() {
        return;
    }

    let stats = attack.weapon.stats();
    let hitbox = swing_hitbox(player_transform, stats, animation_facing(player_transform));

    for (entity, transform, mut health, mut sprite) in &mut enemies {
        if attack.hit_entities.contains(&entity) || health.is_dead() {
            continue;
        }

        if !hitbox_overlaps(hitbox, enemy_bounds(transform)) {
            continue;
        }

        let damage = damage_amount(stats.attack_power, 0.0);
        health.take_damage(damage);
        attack.hit_entities.push(entity);

        sprite.color = Color::srgb(1.0, 0.45, 0.45);
        commands.entity(entity).insert(HitFlash {
            timer: Timer::from_seconds(0.12, TimerMode::Once),
        });

        if health.is_dead() {
            commands.entity(entity).remove::<Patrol>();
            commands.entity(entity).insert(EnemyCorpse);
            sprite.color = Color::srgba(0.55, 0.55, 0.6, 0.85);
        }
    }
}

pub fn tick_hit_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut flashes: Query<(Entity, &mut HitFlash, &mut Sprite, Option<&EnemyCorpse>)>,
) {
    for (entity, mut flash, mut sprite, corpse) in &mut flashes {
        flash.timer.tick(time.delta());
        if flash.timer.finished() {
            if corpse.is_some() {
                sprite.color = Color::srgba(0.55, 0.55, 0.6, 0.85);
            } else {
                sprite.color = Color::WHITE;
            }
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

fn swing_hitbox(player: &Transform, stats: WeaponStats, facing: f32) -> Rect {
    let half = player_half_extents();
    let center = player.translation.truncate();

    let reach = stats.reach;
    if facing >= 0.0 {
        Rect {
            min_x: center.x + 4.0,
            max_x: center.x + reach,
            min_y: center.y - half.y + 4.0,
            max_y: center.y + half.y - 2.0,
        }
    } else {
        Rect {
            min_x: center.x - reach,
            max_x: center.x - 4.0,
            min_y: center.y - half.y + 4.0,
            max_y: center.y + half.y - 2.0,
        }
    }
}

fn enemy_bounds(transform: &Transform) -> Rect {
    let center = transform.translation.truncate();
    let half = enemy_half_extents();
    Rect {
        min_x: center.x - half.x,
        max_x: center.x + half.x,
        min_y: center.y - half.y,
        max_y: center.y + half.y,
    }
}

fn hitbox_overlaps(a: Rect, b: Rect) -> bool {
    a.min_x < b.max_x && a.max_x > b.min_x && a.min_y < b.max_y && a.max_y > b.min_y
}

fn animation_facing(transform: &Transform) -> f32 {
    if transform.scale.x < 0.0 {
        -1.0
    } else {
        1.0
    }
}