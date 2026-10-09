use bevy::prelude::*;

use std::f32::consts::{FRAC_PI_2, TAU};

use crate::audio::CombatSfx;
use crate::dungeon::{
    player_half_extents, DungeonPlayer, EnemyHitbox, EnemyKind, EnemyKnockback, KingSlimeBoss,
    KnockbackTarget, PlayerAnimation, PlayerVelocity,
};
use crate::graphics::{hunter_blade_tip_reach, TILE};
use crate::player::Loadout;

use super::attack::{EnemyCorpse, PlayerAttack};
use super::health::{damage_amount, Health};
use super::hit_stop::{HitStop, HIT_STOP_HEAVY};
use super::hitbox::{
    blade_root_from_center, enemy_aabb, expand_hit_rect, hitbox_overlaps, sword_sprite_hit_rect,
    HitRect,
};
use super::hits::{apply_enemy_strike, EnemyStrike};
use super::player_block::PlayerBlock;
use super::skills::{SkillBindings, SkillKind};
use super::weapon::{spear_special_reach, EquippedWeapon, WeaponFamily, WeaponKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpecialMoveKind {
    Charge,
    Spin,
    /// Spear reach special (skill bar Spin slot).
    Thrust,
}

/// Remaining cooldown seconds for specials (Charge / Spin-or-Thrust).
#[derive(Resource, Debug, Default)]
pub struct SpecialCooldownState {
    pub charge: f32,
    pub spin_slot: f32,
}

impl SpecialCooldownState {
    pub fn remaining(&self, kind: SpecialMoveKind) -> f32 {
        match kind {
            SpecialMoveKind::Charge => self.charge,
            SpecialMoveKind::Spin | SpecialMoveKind::Thrust => self.spin_slot,
        }
    }

    pub fn is_ready(&self, kind: SpecialMoveKind) -> bool {
        self.remaining(kind) <= 0.0
    }

    pub fn start(&mut self, kind: SpecialMoveKind, duration: f32) {
        match kind {
            SpecialMoveKind::Charge => self.charge = duration,
            SpecialMoveKind::Spin | SpecialMoveKind::Thrust => self.spin_slot = duration,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.charge = (self.charge - dt).max(0.0);
        self.spin_slot = (self.spin_slot - dt).max(0.0);
    }

    pub fn remaining_for_skill(&self, skill: SkillKind) -> f32 {
        match skill {
            SkillKind::Charge => self.charge,
            SkillKind::Spin => self.spin_slot,
            _ => 0.0,
        }
    }
}

impl SpecialMoveKind {
    pub fn base_cooldown(self) -> f32 {
        match self {
            Self::Charge => 4.0,
            Self::Spin => 5.0,
            Self::Thrust => 3.5,
        }
    }
}

pub fn special_for_weapon(weapon: WeaponKind, skill: SkillKind) -> Option<SpecialMoveKind> {
    match (weapon.family(), skill) {
        (WeaponFamily::Sword, SkillKind::Charge) => Some(SpecialMoveKind::Charge),
        (WeaponFamily::Sword, SkillKind::Spin) => Some(SpecialMoveKind::Spin),
        (WeaponFamily::Spear, SkillKind::Charge) => Some(SpecialMoveKind::Charge),
        (WeaponFamily::Spear, SkillKind::Spin) => Some(SpecialMoveKind::Thrust),
        _ => None,
    }
}

pub fn weapon_allows_special(weapon: WeaponKind, kind: SpecialMoveKind) -> bool {
    match weapon.family() {
        WeaponFamily::Sword => matches!(kind, SpecialMoveKind::Charge | SpecialMoveKind::Spin),
        WeaponFamily::Spear => matches!(kind, SpecialMoveKind::Charge | SpecialMoveKind::Thrust),
    }
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
            SpecialMoveKind::Thrust => THRUST_SECS,
        }
    }

    pub fn hit_window(&self) -> (f32, f32) {
        match self.kind {
            SpecialMoveKind::Charge => (CHARGE_HIT_START, CHARGE_HIT_END),
            SpecialMoveKind::Spin => (SPIN_HIT_START, SPIN_HIT_END),
            SpecialMoveKind::Thrust => (THRUST_HIT_START, THRUST_HIT_END),
        }
    }

    pub fn in_hit_window(&self) -> bool {
        if !self.is_active() {
            return false;
        }
        let elapsed = self.timer.elapsed_secs();
        let (start, end) = self.hit_window();
        elapsed >= start && elapsed <= end
    }
}

const CHARGE_SPEED: f32 = 310.0;
const CHARGE_SECS: f32 = 0.4;
const CHARGE_HIT_START: f32 = 0.04;
const CHARGE_HIT_END: f32 = 0.36;
const CHARGE_ATTACK_POWER: f32 = 22.0;

const SPIN_SECS: f32 = 0.5;
const SPIN_HIT_START: f32 = 0.1;
const SPIN_HIT_END: f32 = 0.42;
const SPIN_ATTACK_POWER: f32 = 18.0;
/// Spin distances were tuned against the 160 px hunter when a tile was 16 px.
/// Multiplying by TILE would double the whirl on the 32 px module.
const SPIN_TUNED_TILE: f32 = 16.0;
const SPIN_ARM_RADIUS: f32 = SPIN_TUNED_TILE * 1.85;
const SPIN_SWORD_HIT_PADDING: f32 = SPIN_TUNED_TILE * 0.85;
const SPIN_REACH_PAD: f32 = SPIN_TUNED_TILE * 0.35;
const SPIN_DEFLECT_PAD: f32 = SPIN_TUNED_TILE * 0.25;
const SPIN_PIVOT_Y: f32 = 2.0;

const THRUST_SECS: f32 = 0.38;
const THRUST_HIT_START: f32 = 0.06;
const THRUST_HIT_END: f32 = 0.3;
const THRUST_ATTACK_POWER: f32 = 24.0;
const THRUST_REACH: f32 = spear_special_reach();

pub fn player_is_busy(
    attack: &PlayerAttack,
    block: &PlayerBlock,
    special: Option<&PlayerSpecialMove>,
) -> bool {
    attack.is_active() || block.is_active() || special.is_some_and(|m| m.is_active())
}

pub fn special_blocks_movement(special: Option<&PlayerSpecialMove>) -> bool {
    special.is_some_and(|m| {
        m.is_active() && matches!(m.kind, SpecialMoveKind::Charge | SpecialMoveKind::Thrust)
    })
}

/// Horizontal velocity applied while a movement-locking special is active.
pub fn special_move_speed(special: &PlayerSpecialMove) -> f32 {
    match special.kind {
        SpecialMoveKind::Charge => special.charge_direction * CHARGE_SPEED,
        SpecialMoveKind::Thrust => special.charge_direction * 95.0,
        SpecialMoveKind::Spin => 0.0,
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
    let reach = spin_world_reach() + SPIN_DEFLECT_PAD;
    pivot.distance(projectile_center) <= reach
}

pub fn tick_special_cooldowns(
    time: Res<Time>,
    hit_stop: Res<HitStop>,
    mut cooldowns: ResMut<SpecialCooldownState>,
) {
    if hit_stop.is_active() {
        return;
    }
    cooldowns.tick(time.delta_secs());
}

pub fn start_player_special_moves(
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    mut cooldowns: ResMut<SpecialCooldownState>,
    bindings: Res<SkillBindings>,
    loadout: Res<Loadout>,
    keyboard: Res<ButtonInput<KeyCode>>,
    hit_stop: Res<HitStop>,
    mut player: Query<
        (
            Entity,
            &EquippedWeapon,
            &PlayerAnimation,
            &PlayerVelocity,
            &PlayerAttack,
            &PlayerBlock,
            Option<&PlayerSpecialMove>,
        ),
        With<DungeonPlayer>,
    >,
) {
    if hit_stop.is_active() {
        return;
    }

    let Ok((entity, weapon, animation, velocity, attack, block, existing)) =
        player.get_single_mut()
    else {
        return;
    };

    if !velocity.grounded
        || player_is_busy(attack, block, existing)
        || existing.is_some_and(|m| m.is_active())
    {
        return;
    }

    let skill = if SkillBindings::skill_just_pressed(&keyboard, &bindings, SkillKind::Charge) {
        Some(SkillKind::Charge)
    } else if SkillBindings::skill_just_pressed(&keyboard, &bindings, SkillKind::Spin) {
        Some(SkillKind::Spin)
    } else {
        None
    };

    let Some(skill) = skill else {
        return;
    };

    let Some(kind) = special_for_weapon(weapon.0, skill) else {
        return;
    };

    if !weapon_allows_special(weapon.0, kind) {
        return;
    }

    let cd_mult = loadout.special_cooldown_multiplier();
    if !cooldowns.is_ready(kind) {
        return;
    }

    let facing = animation.facing.signum().max(-1.0).min(1.0);
    let direction = if facing == 0.0 { 1.0 } else { facing };
    let duration = match kind {
        SpecialMoveKind::Charge => CHARGE_SECS,
        SpecialMoveKind::Spin => SPIN_SECS,
        SpecialMoveKind::Thrust => THRUST_SECS,
    };

    commands.entity(entity).insert(PlayerSpecialMove {
        kind,
        timer: Timer::from_seconds(duration, TimerMode::Once),
        charge_direction: direction,
        hit_entities: Vec::new(),
    });

    cooldowns.start(kind, kind.base_cooldown() * cd_mult);

    sfx.send(match kind {
        SpecialMoveKind::Charge => CombatSfx::Charge,
        SpecialMoveKind::Spin | SpecialMoveKind::Thrust => CombatSfx::Spin,
    });
}

pub fn tick_player_special_moves(
    time: Res<Time>,
    mut commands: Commands,
    hit_stop: Res<HitStop>,
    mut player: Query<(Entity, &mut PlayerSpecialMove), With<DungeonPlayer>>,
) {
    if hit_stop.is_active() {
        return;
    }

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

pub fn resolve_special_move_hits(
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    mut hit_stop: ResMut<HitStop>,
    loadout: Res<Loadout>,
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
    let thrust_box = thrust_hitbox(player_transform, special.charge_direction);
    let mult = loadout.attack_power_multiplier();

    let attack_power = match special.kind {
        SpecialMoveKind::Charge => CHARGE_ATTACK_POWER,
        SpecialMoveKind::Spin => SPIN_ATTACK_POWER,
        SpecialMoveKind::Thrust => THRUST_ATTACK_POWER,
    } * mult;

    let mut landed = false;

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
            SpecialMoveKind::Thrust => hitbox_overlaps(thrust_box, enemy_box),
        };

        if !overlaps {
            continue;
        }

        let target = KnockbackTarget::of(kind, boss);
        let knockback = match special.kind {
            SpecialMoveKind::Charge | SpecialMoveKind::Thrust => {
                EnemyKnockback::from_charge(special.charge_direction, target)
            }
            SpecialMoveKind::Spin => {
                EnemyKnockback::away_from_player(player_transform, transform, target)
            }
        };

        apply_enemy_strike(
            &mut commands,
            &mut sfx,
            entity,
            &mut health,
            &mut sprite,
            &mut special.hit_entities,
            EnemyStrike {
                damage: damage_amount(attack_power, 0.0),
                sfx: CombatSfx::HeavyHit,
                knockback,
            },
        );
        landed = true;
    }

    if landed {
        hit_stop.request(HIT_STOP_HEAVY);
    }
}

struct WeaponPose {
    translation: Vec3,
    angle: f32,
}

fn spin_orbit_angle(progress: f32) -> f32 {
    -progress * TAU
}

fn spin_weapon_pose(progress: f32, facing: f32) -> WeaponPose {
    let offset = spin_orbit_offset(progress, facing);
    let sword_angle = spin_orbit_angle(progress) - FRAC_PI_2;

    WeaponPose {
        translation: Vec3::new(offset.x, offset.y, 0.55),
        angle: sword_angle,
    }
}

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
    let blade_world = center + Vec2::new(facing * pose.translation.x, pose.translation.y);

    Some(sword_sprite_hit_rect(blade_world, pose.angle))
}

fn spin_pivot_world(player: &Transform) -> Vec2 {
    let center = player.translation.truncate();
    center + Vec2::new(0.0, SPIN_PIVOT_Y)
}

fn spin_world_reach() -> f32 {
    SPIN_ARM_RADIUS + hunter_blade_tip_reach() * 0.5 + SPIN_REACH_PAD
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

fn thrust_hitbox(player: &Transform, direction: f32) -> HitRect {
    let half = player_half_extents();
    let center = player.translation.truncate();
    let front = center.x + direction * blade_root_from_center(half.x);
    let tip = center.x + direction * THRUST_REACH;

    HitRect {
        min_x: front.min(tip),
        max_x: front.max(tip),
        min_y: center.y - half.y,
        max_y: center.y + half.y * 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spear_maps_spin_skill_to_thrust() {
        assert_eq!(
            special_for_weapon(WeaponKind::RustySpear, SkillKind::Spin),
            Some(SpecialMoveKind::Thrust)
        );
        assert!(!weapon_allows_special(
            WeaponKind::RustySpear,
            SpecialMoveKind::Spin
        ));
    }

    #[test]
    fn spear_thrust_special_clears_the_body_and_the_lunge() {
        let player = Transform::from_xyz(0.0, 0.0, 0.0);
        let rect = thrust_hitbox(&player, 1.0);
        let body_half = player_half_extents().x;
        let lunge = WeaponKind::RustySpear.moveset().steps[2].reach;
        assert!(rect.min_x < body_half);
        assert!(rect.max_x > body_half);
        assert!(rect.max_x > lunge);
        assert!(rect.max_x > rect.min_x);
    }

    #[test]
    fn spin_sweep_clears_the_body() {
        let player = Transform::from_xyz(25.0, 0.0, 0.0);
        let sweep = spin_sweep_rect(&player);
        let body_half = player_half_extents().x;
        assert!(sweep.max_x - player.translation.x > body_half);
        assert!(player.translation.x - sweep.min_x > body_half);
    }

    #[test]
    fn spin_arm_stays_on_the_sixteen_px_orbit() {
        assert_eq!(SPIN_ARM_RADIUS, 16.0 * 1.85);
        assert!(SPIN_ARM_RADIUS < TILE * 1.85);
        assert_eq!(SPIN_SWORD_HIT_PADDING, 16.0 * 0.85);
    }

    #[test]
    fn sword_keeps_spin() {
        assert_eq!(
            special_for_weapon(WeaponKind::IronSword, SkillKind::Spin),
            Some(SpecialMoveKind::Spin)
        );
    }
}
