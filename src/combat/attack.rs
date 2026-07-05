use bevy::prelude::*;

use std::f32::consts::FRAC_PI_2;

use crate::audio::CombatSfx;
use crate::dungeon::{
    DungeonArt, DungeonPlayer, EnemyAggro, EnemyHitbox, EnemyKind, EnemyKnockback, KingSlimeBoss,
    Patrol, PlayerAnimation, PlayerVelocity, SWORD_SPRITE_HEIGHT,
    PLAYER_IDLE_FRAMES, PLAYER_RUN_FRAMES,
};

use super::hitbox::{enemy_aabb, hitbox_overlaps, sword_swing_aabb, HitRect};
use super::player_block::PlayerBlock;
use super::skills::{SkillBindings, SkillKind};
use super::special_moves::{player_is_busy, PlayerSpecialMove};

use crate::dungeon::player_half_extents;
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

/// Active sword swing volume during the hit window (for projectile parries).
pub fn player_sword_hit_rect(player: &Transform, attack: &PlayerAttack) -> Option<HitRect> {
    if !attack.in_hit_window() {
        return None;
    }

    match attack.weapon {
        WeaponKind::RustySword => Some(sword_swing_aabb(
            player,
            swing_angle(sword_arc_progress(attack)),
        )),
        WeaponKind::RustySpear => None,
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

#[derive(Component)]
pub struct WeaponOnBack;

/// Sheathed sword pose in player-local pixels (parent scale mirrors with facing).
const SHEATHED_SWORD_X: f32 = -4.0;
const SHEATHED_SWORD_Y: f32 = 8.0;
const SHEATHED_SWORD_Z: f32 = -0.2;
const SHEATHED_SWORD_ANGLE: f32 = 0.45;

/// Per-frame Y offsets matching knight idle/run sprite bob (native pixels).
const IDLE_SHEATHED_BOB: [f32; 4] = [0.0, -0.5, -1.0, -0.5];
const RUN_SHEATHED_BOB: [f32; 4] = [-1.5, 0.5, 1.5, -1.0];

struct SwingPose {
    translation: Vec3,
    rotation: Quat,
}

pub fn start_player_attack(
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    art: Res<DungeonArt>,
    bindings: Res<SkillBindings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: Query<
        (
            Entity,
            &EquippedWeapon,
            &mut PlayerAttack,
            &PlayerBlock,
            Option<&PlayerSpecialMove>,
        ),
        With<DungeonPlayer>,
    >,
) {
    let Ok((entity, weapon, mut attack, block, special)) = player.get_single_mut() else {
        return;
    };

    if !SkillBindings::skill_just_pressed(&keyboard, &bindings, SkillKind::Attack)
        || player_is_busy(&attack, block, special)
    {
        return;
    }

    let stats = weapon.0.stats();
    attack.weapon = weapon.0;
    attack.hit_entities.clear();
    attack.timer = Timer::from_seconds(stats.swing_secs, TimerMode::Once);
    attack.timer.reset();
    sfx.send(CombatSfx::SwordSwing);

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

pub fn spawn_sheathed_sword(image: Handle<Image>) -> impl Bundle {
    (
        WeaponOnBack,
        Sprite {
            image,
            ..default()
        },
        Transform {
            translation: Vec3::new(SHEATHED_SWORD_X, SHEATHED_SWORD_Y, SHEATHED_SWORD_Z),
            rotation: Quat::from_rotation_z(SHEATHED_SWORD_ANGLE),
            ..default()
        },
    )
}

pub fn sync_sheathed_weapon(
    player: Query<
        (
            &PlayerAttack,
            &EquippedWeapon,
            &PlayerBlock,
            Option<&PlayerSpecialMove>,
            &PlayerAnimation,
            &PlayerVelocity,
        ),
        With<DungeonPlayer>,
    >,
    mut sheathed: Query<(&mut Transform, &mut Visibility), With<WeaponOnBack>>,
) {
    let Ok((attack, weapon, block, special, animation, velocity)) = player.get_single() else {
        return;
    };

    let visible =
        !player_is_busy(attack, block, special) && weapon.0 == WeaponKind::RustySword;
    let bob = sheathed_bob_offset(animation, velocity);

    for (mut transform, mut visibility) in &mut sheathed {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.translation.x = SHEATHED_SWORD_X;
        transform.translation.y = SHEATHED_SWORD_Y + bob;
    }
}

fn sheathed_bob_offset(animation: &PlayerAnimation, velocity: &PlayerVelocity) -> f32 {
    if !velocity.grounded {
        return 0.0;
    }

    if velocity.x.abs() > 1.0 {
        let frame = animation.frame % PLAYER_RUN_FRAMES;
        return RUN_SHEATHED_BOB[frame];
    }

    let frame = animation.frame % PLAYER_IDLE_FRAMES;
    IDLE_SHEATHED_BOB[frame]
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
    mut sfx: EventWriter<CombatSfx>,
    mut player: Query<(&Transform, &mut PlayerAttack), With<DungeonPlayer>>,
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
    let Ok((player_transform, mut attack)) = player.get_single_mut() else {
        return;
    };

    if !attack.in_hit_window() {
        return;
    }

    let stats = attack.weapon.stats();
    let facing = animation_facing(player_transform);
    let hitbox = swing_hitbox(player_transform, &attack, facing);

    for (entity, transform, hitbox_extents, mut health, mut sprite, boss, kind) in &mut enemies {
        if attack.hit_entities.contains(&entity) || health.is_dead() {
            continue;
        }

        if !hitbox_overlaps(hitbox, enemy_bounds(transform, hitbox_extents.0)) {
            continue;
        }

        let damage = damage_amount(stats.attack_power, 0.0);
        health.take_damage(damage);
        attack.hit_entities.push(entity);
        sfx.send(CombatSfx::SwordHit);

        sprite.color = Color::srgb(1.0, 0.45, 0.45);
        commands.entity(entity).insert((
            EnemyAggro::from_hit(),
            HitFlash {
                timer: Timer::from_seconds(0.12, TimerMode::Once),
            },
            EnemyKnockback::away_from_player(
                player_transform,
                transform,
                if boss.is_some() { 0.35 } else { 1.0 },
                kind.is_some_and(|kind| kind.is_airborne()),
            ),
        ));

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

fn swing_hitbox(player: &Transform, attack: &PlayerAttack, facing: f32) -> HitRect {
    match attack.weapon {
        WeaponKind::RustySword => sword_swing_hitbox(player, attack, facing),
        WeaponKind::RustySpear => spear_swing_hitbox(player, attack.weapon.stats(), facing),
    }
}

fn sword_swing_hitbox(player: &Transform, attack: &PlayerAttack, _facing: f32) -> HitRect {
    sword_swing_aabb(player, swing_angle(sword_arc_progress(attack)))
}

fn spear_swing_hitbox(player: &Transform, stats: WeaponStats, facing: f32) -> HitRect {
    let half = player_half_extents();
    let center = player.translation.truncate();
    let front = center.x + facing * half.x;
    let tip_x = center.x + facing * stats.reach;

    HitRect {
        min_x: front.min(tip_x),
        max_x: front.max(tip_x),
        min_y: center.y - half.y,
        max_y: center.y + 4.0,
    }
}

fn enemy_bounds(transform: &Transform, half: Vec2) -> HitRect {
    enemy_aabb(transform.translation.truncate(), half)
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