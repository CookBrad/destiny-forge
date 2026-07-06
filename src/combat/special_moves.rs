use bevy::prelude::*;

use std::f32::consts::{FRAC_PI_2, TAU};

use crate::audio::CombatSfx;
use crate::dungeon::{
    player_half_extents, DungeonArt, DungeonPlayer, EnemyAggro, EnemyHitbox, EnemyKind,
    EnemyKnockback, KingSlimeBoss, Patrol, PlayerAnimation, PlayerVelocity,
};

use super::attack::{EnemyCorpse, HitFlash, PlayerAttack};
use super::health::{damage_amount, Health};
use super::hitbox::{
    enemy_aabb, expand_hit_rect, hitbox_overlaps, sword_blade_center_local, sword_sprite_hit_rect,
    HitRect,
};
use crate::graphics::{PIXEL_SCALE, TILE};
use crate::dungeon::SWORD_SPRITE_HEIGHT;
use super::player_block::PlayerBlock;
use crate::combat::{SkillBindings, SkillKind};


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialMoveKind {
    Charge,
    Spin,
}

#[derive(Component)]
pub struct PlayerSpecialMove {
    pub kind: SpecialMoveKind,
    pub timer: Timer,
    pub charge_direction: f32,
    pub hit_entities: Vec<Entity>,
}

impl PlayerSpecialMove {
    pub fn is_active(&self) -> bool {
        !self.timer.finished()
    }

    pub fn duration(&self) -> f32 {
        match self.kind {
            SpecialMoveKind::Charge => CHARGE_SECS,
            SpecialMoveKind::Spin => SPIN_SECS,
        }
    }

    pub fn in_hit_window(&self) -> bool {
        if !self.is_active() {
            return false;
        }
        let elapsed = self.timer.elapsed_secs();
        let (start, end) = match self.kind {
            SpecialMoveKind::Charge => (CHARGE_HIT_START, CHARGE_HIT_END),
            SpecialMoveKind::Spin => (SPIN_HIT_START, SPIN_HIT_END),
        };
        elapsed >= start && elapsed <= end
    }
}

#[derive(Component)]
pub struct WeaponSpecialFx;

const CHARGE_SPEED: f32 = 310.0;
const CHARGE_SECS: f32 = 0.4;
const CHARGE_HIT_START: f32 = 0.04;
const CHARGE_HIT_END: f32 = 0.36;
const CHARGE_ATTACK_POWER: f32 = 22.0;

const SPIN_SECS: f32 = 0.5;
const SPIN_HIT_START: f32 = 0.1;
const SPIN_HIT_END: f32 = 0.42;
const SPIN_ATTACK_POWER: f32 = 18.0;
const SPIN_ARM_RADIUS: f32 = TILE * 1.85;
const SPIN_SWORD_HIT_PADDING: f32 = TILE * 0.85;
const SPIN_PARRY_PADDING: f32 = TILE * 0.45;
const SPIN_PIVOT_Y: f32 = 2.0;

pub fn player_is_busy(
    attack: &PlayerAttack,
    block: &PlayerBlock,
    special: Option<&PlayerSpecialMove>,
) -> bool {
    attack.is_active() || block.is_active() || special.is_some_and(|m| m.is_active())
}

pub fn special_blocks_movement(special: Option<&PlayerSpecialMove>) -> bool {
    special.is_some_and(|m| m.is_active() && m.kind == SpecialMoveKind::Charge)
}

pub fn charge_speed() -> f32 {
    CHARGE_SPEED
}

pub fn special_move_hit_rect(player: &Transform, special: &PlayerSpecialMove) -> Option<HitRect> {
    if !special.in_hit_window() {
        return None;
    }

    match special.kind {
        SpecialMoveKind::Charge => Some(charge_hitbox(player, special.charge_direction)),
        SpecialMoveKind::Spin => spin_parry_hit_rect(player, special),
    }
}

/// Whirlwind deflects for the full spin using the blade path plus sweep volume.
pub fn spin_deflects_projectile(
    player: &Transform,
    special: &PlayerSpecialMove,
    projectile_center: Vec2,
    projectile_hit: HitRect,
) -> bool {
    if special.kind != SpecialMoveKind::Spin || !special.is_active() {
        return false;
    }

    if spin_blade_hit_rect(player, special)
        .map(|rect| expand_hit_rect(rect, SPIN_SWORD_HIT_PADDING))
        .is_some_and(|rect| hitbox_overlaps(rect, projectile_hit))
    {
        return true;
    }

    if hitbox_overlaps(spin_sweep_rect(player), projectile_hit) {
        return true;
    }

    let pivot = spin_pivot_world(player);
    let reach = spin_world_reach() + TILE * 0.25;
    pivot.distance(projectile_center) <= reach
}

pub fn start_player_special_moves(
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    art: Res<DungeonArt>,
    bindings: Res<SkillBindings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: Query<
        (
            Entity,
            &PlayerAnimation,
            &PlayerVelocity,
            &PlayerAttack,
            &PlayerBlock,
            Option<&PlayerSpecialMove>,
        ),
        With<DungeonPlayer>,
    >,
) {
    let Ok((entity, animation, velocity, attack, block, existing)) = player.get_single_mut() else {
        return;
    };

    if !velocity.grounded
        || player_is_busy(attack, block, existing)
        || existing.is_some_and(|m| m.is_active())
    {
        return;
    }

    let facing = animation.facing.signum().max(-1.0).min(1.0);
    let direction = if facing == 0.0 { 1.0 } else { facing };

    let kind = if SkillBindings::skill_just_pressed(&keyboard, &bindings, SkillKind::Charge) {
        Some(SpecialMoveKind::Charge)
    } else if SkillBindings::skill_just_pressed(&keyboard, &bindings, SkillKind::Spin) {
        Some(SpecialMoveKind::Spin)
    } else {
        None
    };

    let Some(kind) = kind else {
        return;
    };

    let duration = match kind {
        SpecialMoveKind::Charge => CHARGE_SECS,
        SpecialMoveKind::Spin => SPIN_SECS,
    };

    commands.entity(entity).insert(PlayerSpecialMove {
        kind,
        timer: Timer::from_seconds(duration, TimerMode::Once),
        charge_direction: direction,
        hit_entities: Vec::new(),
    });

    sfx.send(match kind {
        SpecialMoveKind::Charge => CombatSfx::Charge,
        SpecialMoveKind::Spin => CombatSfx::Spin,
    });

    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: art.weapon_anime_sword.clone(),
                ..default()
            },
            Transform::default(),
            WeaponSpecialFx,
        ));
    });
}

pub fn cleanup_special_weapon(
    mut commands: Commands,
    player: Query<&PlayerSpecialMove, With<DungeonPlayer>>,
    fx: Query<Entity, With<WeaponSpecialFx>>,
) {
    let Ok(special) = player.get_single() else {
        for entity in &fx {
            commands.entity(entity).try_despawn();
        }
        return;
    };

    if !special.is_active() {
        for entity in &fx {
            commands.entity(entity).try_despawn();
        }
    }
}

pub fn tick_player_special_moves(
    time: Res<Time>,
    mut commands: Commands,
    mut player: Query<(Entity, &mut PlayerSpecialMove), With<DungeonPlayer>>,
) {
    let Ok((entity, mut special)) = player.get_single_mut() else {
        return;
    };

    if !special.is_active() {
        return;
    }

    special.timer.tick(time.delta());

    if special.timer.finished() {
        commands.entity(entity).remove::<PlayerSpecialMove>();
    }
}

pub fn animate_special_weapon(
    player: Query<&PlayerSpecialMove, With<DungeonPlayer>>,
    mut fx: Query<&mut Transform, With<WeaponSpecialFx>>,
) {
    let Ok(special) = player.get_single() else {
        return;
    };

    if !special.is_active() {
        return;
    }

    let progress = (special.timer.elapsed_secs() / special.duration()).clamp(0.0, 1.0);

    for mut transform in &mut fx {
        let pose = match special.kind {
            SpecialMoveKind::Charge => charge_weapon_pose(progress, special.charge_direction),
            SpecialMoveKind::Spin => {
                spin_weapon_pose(progress, special.charge_direction.signum())
            }
        };
        transform.translation = pose.translation;
        transform.rotation = pose.rotation;
    }
}

pub fn resolve_special_move_hits(
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    mut player: Query<(&Transform, &mut PlayerSpecialMove), With<DungeonPlayer>>,
    mut enemies: Query<
        (
            Entity,
            &Transform,
            &EnemyHitbox,
            &mut Health,
            &mut Sprite,
            Option<&KingSlimeBoss>,
            Option<&EnemyKind>,
        ),
        (With<Health>, Without<DungeonPlayer>, Without<EnemyCorpse>),
    >,
) {
    let Ok((player_transform, mut special)) = player.get_single_mut() else {
        return;
    };

    if !special.in_hit_window() {
        return;
    }

    let charge_box = charge_hitbox(player_transform, special.charge_direction);

    let attack_power = match special.kind {
        SpecialMoveKind::Charge => CHARGE_ATTACK_POWER,
        SpecialMoveKind::Spin => SPIN_ATTACK_POWER,
    };

    let damage = damage_amount(attack_power, 0.0);

    for (entity, transform, hitbox_extents, mut health, mut sprite, boss, kind) in &mut enemies {
        if special.hit_entities.contains(&entity) || health.is_dead() {
            continue;
        }

        let enemy_center = transform.translation.truncate();
        let enemy_box = enemy_aabb(enemy_center, hitbox_extents.0);
        let overlaps = match special.kind {
            SpecialMoveKind::Charge => hitbox_overlaps(charge_box, enemy_box),
            SpecialMoveKind::Spin => spin_whirlwind_hits_enemy(
                player_transform,
                &special,
                enemy_center,
                hitbox_extents.0,
            ),
        };

        if !overlaps {
            continue;
        }

        health.take_damage(damage);
        special.hit_entities.push(entity);
        sfx.send(CombatSfx::HeavyHit);

        let airborne = kind.is_some_and(|kind| kind.is_airborne());
        let knockback = match special.kind {
            SpecialMoveKind::Charge => EnemyKnockback::from_charge(
                special.charge_direction,
                boss.is_some(),
                airborne,
            ),
            SpecialMoveKind::Spin => EnemyKnockback::away_from_player(
                player_transform,
                transform,
                if boss.is_some() { 0.35 } else { 1.0 },
                airborne,
            ),
        };

        sprite.color = Color::srgb(1.0, 0.45, 0.45);
        commands.entity(entity).insert((
            EnemyAggro::from_hit(),
            HitFlash {
                timer: Timer::from_seconds(0.12, TimerMode::Once),
            },
            knockback,
        ));

        if health.is_dead() {
            commands.entity(entity).remove::<Patrol>();
            commands.entity(entity).insert(EnemyCorpse);
            sprite.color = Color::srgba(0.55, 0.55, 0.6, 0.85);
        }
    }
}

struct WeaponPose {
    translation: Vec3,
    rotation: Quat,
}

fn charge_weapon_pose(_progress: f32, direction: f32) -> WeaponPose {
    let angle = if direction > 0.0 {
        -FRAC_PI_2 * 0.18
    } else {
        FRAC_PI_2 * 0.18
    };
    let forward = direction * TILE * 0.75;
    let blade = sword_blade_center_local(angle) + Vec2::new(forward, 0.0);

    WeaponPose {
        translation: Vec3::new(blade.x, blade.y, 0.55),
        rotation: Quat::from_rotation_z(angle),
    }
}

fn spin_orbit_angle(progress: f32) -> f32 {
    -progress * TAU
}

fn spin_weapon_pose(progress: f32, facing: f32) -> WeaponPose {
    let offset = spin_orbit_offset(progress, facing);
    let sword_angle = spin_orbit_angle(progress) - FRAC_PI_2;

    WeaponPose {
        translation: Vec3::new(offset.x, offset.y, 0.55),
        rotation: Quat::from_rotation_z(sword_angle),
    }
}

/// Arm extended from the torso like a T-pose, sweeping clockwise.
fn spin_orbit_offset(progress: f32, facing: f32) -> Vec2 {
    let angle = spin_orbit_angle(progress);
    Vec2::new(
        facing * SPIN_ARM_RADIUS * angle.cos(),
        SPIN_PIVOT_Y + SPIN_ARM_RADIUS * angle.sin(),
    )
}

fn spin_blade_hit_rect(player: &Transform, special: &PlayerSpecialMove) -> Option<HitRect> {
    if special.kind != SpecialMoveKind::Spin {
        return None;
    }

    let progress = (special.timer.elapsed_secs() / special.duration()).clamp(0.0, 1.0);
    let facing = special.charge_direction.signum();
    let pose = spin_weapon_pose(progress, facing);
    let center = player.translation.truncate();
    let blade_world =
        center + Vec2::new(facing * pose.translation.x, pose.translation.y) * PIXEL_SCALE;

    Some(sword_sprite_hit_rect(
        blade_world,
        spin_orbit_angle(progress) - FRAC_PI_2,
    ))
}

fn spin_parry_hit_rect(player: &Transform, special: &PlayerSpecialMove) -> Option<HitRect> {
    spin_blade_hit_rect(player, special).map(|rect| expand_hit_rect(rect, SPIN_PARRY_PADDING))
}

fn spin_pivot_world(player: &Transform) -> Vec2 {
    let center = player.translation.truncate();
    center + Vec2::new(0.0, SPIN_PIVOT_Y * PIXEL_SCALE)
}

fn spin_world_reach() -> f32 {
    SPIN_ARM_RADIUS * PIXEL_SCALE + SWORD_SPRITE_HEIGHT * 0.5 * PIXEL_SCALE + TILE * 0.35
}

fn spin_sweep_rect(player: &Transform) -> HitRect {
    let pivot = spin_pivot_world(player);
    let reach = spin_world_reach();
    let vertical = reach * 0.72;

    HitRect {
        min_x: pivot.x - reach,
        max_x: pivot.x + reach,
        min_y: pivot.y - vertical,
        max_y: pivot.y + vertical,
    }
}

fn spin_whirlwind_hits_enemy(
    player: &Transform,
    special: &PlayerSpecialMove,
    enemy_center: Vec2,
    enemy_half: Vec2,
) -> bool {
    let enemy_box = enemy_aabb(enemy_center, enemy_half);

    if spin_blade_hit_rect(player, special)
        .map(|rect| expand_hit_rect(rect, SPIN_SWORD_HIT_PADDING))
        .is_some_and(|rect| hitbox_overlaps(rect, enemy_box))
    {
        return true;
    }

    if hitbox_overlaps(spin_sweep_rect(player), enemy_box) {
        return true;
    }

    let pivot = spin_pivot_world(player);
    let reach = spin_world_reach() + enemy_half.x.max(enemy_half.y);
    pivot.distance(enemy_center) <= reach
}

fn charge_hitbox(player: &Transform, direction: f32) -> HitRect {
    let half = player_half_extents();
    let center = player.translation.truncate();
    let reach = half.x + TILE * 1.75;
    let front = center.x + direction * reach;

    HitRect {
        min_x: center.x.min(front) - half.x * 0.35,
        max_x: center.x.max(front) + half.x * 0.35,
        min_y: center.y - half.y,
        max_y: center.y + half.y,
    }
}

