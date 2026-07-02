use bevy::prelude::*;

use std::f32::consts::FRAC_PI_2;

use crate::dungeon::{DungeonArt, DungeonPlayer, Patrol, SWORD_SPRITE_HEIGHT, SWORD_SPRITE_WIDTH};
use crate::dungeon::player_half_extents;
use crate::graphics::enemy_half_extents;

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

/// Handle pivot on the player in local space (matches swing overlay).
const SWORD_PIVOT_Y: f32 = -10.0;
/// Visual arc completes faster than the full attack timer (hit window unchanged).
const SWORD_ARC_SPEED: f32 = 2.2;

#[derive(Component)]
pub struct WeaponSwingFx;

struct SwingPose {
    translation: Vec3,
    rotation: Quat,
}

pub fn start_player_attack(
    mut commands: Commands,
    art: Res<DungeonArt>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: Query<(Entity, &EquippedWeapon, &mut PlayerAttack), With<DungeonPlayer>>,
) {
    let Ok((entity, weapon, mut attack)) = player.get_single_mut() else {
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

    let pose = swing_pose(0.0);
    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: art.weapon_anime_sword.clone(),
                ..default()
            },
            Transform {
                translation: pose.translation,
                rotation: pose.rotation,
                ..default()
            },
            WeaponSwingFx,
        ));
    });
}

pub fn animate_weapon_swing(
    mut commands: Commands,
    player: Query<&PlayerAttack, With<DungeonPlayer>>,
    mut swings: Query<(Entity, &WeaponSwingFx, &mut Transform)>,
) {
    let Ok(attack) = player.get_single() else {
        return;
    };

    if !attack.is_active() {
        for (entity, _, _) in &swings {
            commands.entity(entity).despawn();
        }
        return;
    }

    let progress = sword_arc_progress(attack);

    for (_, swing, mut transform) in &mut swings {
        let pose = swing_pose(progress);
        transform.translation = pose.translation;
        transform.rotation = pose.rotation;
    }
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
    let facing = animation_facing(player_transform);
    let hitbox = swing_hitbox(player_transform, &attack, facing);

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

fn swing_hitbox(player: &Transform, attack: &PlayerAttack, facing: f32) -> Rect {
    match attack.weapon {
        WeaponKind::RustySword => sword_swing_hitbox(player, attack, facing),
        WeaponKind::RustySpear => spear_swing_hitbox(player, attack.weapon.stats(), facing),
    }
}

fn sword_swing_hitbox(player: &Transform, attack: &PlayerAttack, facing: f32) -> Rect {
    let player_center = player.translation.truncate();
    let angle = swing_angle(sword_arc_progress(attack));
    let blade_local = sword_blade_center_local(angle);
    let blade_world =
        player_center + Vec2::new(facing * blade_local.x, blade_local.y);

    sword_sprite_aabb(blade_world, angle)
}

/// Axis-aligned bounds of the rotated sword sprite (12×30 native pixels).
fn sword_sprite_aabb(center: Vec2, angle: f32) -> Rect {
    let half_w = SWORD_SPRITE_WIDTH * 0.5;
    let half_h = SWORD_SPRITE_HEIGHT * 0.5;
    let c = angle.cos().abs();
    let s = angle.sin().abs();
    let extent_x = c * half_w + s * half_h;
    let extent_y = s * half_w + c * half_h;

    Rect {
        min_x: center.x - extent_x,
        max_x: center.x + extent_x,
        min_y: center.y - extent_y,
        max_y: center.y + extent_y,
    }
}

fn spear_swing_hitbox(player: &Transform, stats: WeaponStats, facing: f32) -> Rect {
    let half = player_half_extents();
    let center = player.translation.truncate();
    let front = center.x + facing * half.x;
    let tip_x = center.x + facing * stats.reach;

    Rect {
        min_x: front.min(tip_x),
        max_x: front.max(tip_x),
        min_y: center.y - half.y,
        max_y: center.y + 4.0,
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

fn sword_arc_progress(attack: &PlayerAttack) -> f32 {
    (attack.timer.elapsed_secs() / attack.weapon.stats().swing_secs * SWORD_ARC_SPEED).clamp(0.0, 1.0)
}

/// Vertical sword starts raised and sweeps 90° downward in local space.
/// Parent scale flip mirrors the arc when the player faces left.
fn swing_angle(progress: f32) -> f32 {
    -progress * FRAC_PI_2
}

fn sword_blade_center_local(angle: f32) -> Vec2 {
    let half_height = SWORD_SPRITE_HEIGHT * 0.5;
    Vec2::new(
        half_height * (-angle).sin(),
        SWORD_PIVOT_Y + half_height * (-angle).cos(),
    )
}

/// Tip traces a circular arc around the waist pivot (not in-place rotation).
fn swing_pose(progress: f32) -> SwingPose {
    let angle = swing_angle(progress);
    let center = sword_blade_center_local(angle);

    SwingPose {
        translation: Vec3::new(center.x, center.y, 0.5),
        rotation: Quat::from_rotation_z(angle),
    }
}