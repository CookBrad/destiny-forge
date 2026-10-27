use bevy::ecs::system::SystemParam;
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
            Self::Charge => CHARGE_COOLDOWN_SECS,
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
const CHARGE_COOLDOWN_SECS: f32 = 2.0;

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

#[derive(SystemParam)]
pub(crate) struct SpecialMoveStartWriters<'w, 's> {
    commands: Commands<'w, 's>,
    combat_sound: EventWriter<'w, CombatSfx>,
}

pub(crate) type SpecialMoveActor<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static EquippedWeapon,
        &'static PlayerAnimation,
        &'static PlayerVelocity,
        &'static PlayerAttack,
        &'static PlayerBlock,
        Option<&'static PlayerSpecialMove>,
    ),
    With<DungeonPlayer>,
>;

pub fn start_player_special_moves(
    writers: SpecialMoveStartWriters,
    mut cooldowns: ResMut<SpecialCooldownState>,
    bindings: Res<SkillBindings>,
    loadout: Res<Loadout>,
    actions: Res<crate::input::ActionState>,
    hit_stop: Res<HitStop>,
    mut player: SpecialMoveActor,
) {
    let SpecialMoveStartWriters {
        mut commands,
        mut combat_sound,
    } = writers;
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

    let skill = if SkillBindings::skill_just_pressed(&actions, &bindings, SkillKind::Charge) {
        Some(SkillKind::Charge)
    } else if SkillBindings::skill_just_pressed(&actions, &bindings, SkillKind::Spin) {
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

    let sign = animation.facing.signum();
    let facing = if sign.is_nan() || sign < -1.0 {
        -1.0
    } else if sign > 1.0 {
        1.0
    } else {
        sign
    };
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

    combat_sound.send(match kind {
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

pub(crate) type SpecialMoveHitEnemies<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static EnemyHitbox,
        &'static mut Health,
        &'static mut Sprite,
        Option<&'static KingSlimeBoss>,
        Option<&'static EnemyKind>,
    ),
    (With<Health>, Without<DungeonPlayer>, Without<EnemyCorpse>),
>;

pub fn resolve_special_move_hits(
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    mut hit_stop: ResMut<HitStop>,
    loadout: Res<Loadout>,
    mut player: Query<(&Transform, &mut PlayerSpecialMove), With<DungeonPlayer>>,
    mut enemies: SpecialMoveHitEnemies,
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
        const {
            assert!(SPIN_ARM_RADIUS < TILE * 1.85);
        }
        assert_eq!(SPIN_SWORD_HIT_PADDING, 16.0 * 0.85);
    }

    #[test]
    fn charge_cooldown_is_shorter_than_thrust_and_spin() {
        let charge = SpecialMoveKind::Charge.base_cooldown();
        assert_eq!(charge, CHARGE_COOLDOWN_SECS);
        assert!(charge < SpecialMoveKind::Thrust.base_cooldown());
        assert!(charge < SpecialMoveKind::Spin.base_cooldown());
    }

    #[test]
    fn sword_keeps_spin() {
        assert_eq!(
            special_for_weapon(WeaponKind::IronSword, SkillKind::Spin),
            Some(SpecialMoveKind::Spin)
        );
    }

    #[test]
    fn a_grounded_sword_charge_starts_toward_the_hunter_facing() {
        let mut app = special_app(crate::input::action_state_after_pressing(
            &[KeyCode::Digit3],
            &[],
        ));
        let facing = 1.0;
        let hunter = app.world_mut().spawn(sword_hunter(facing)).id();
        app.add_systems(Update, start_player_special_moves);
        app.update();
        let special = app.world().get::<PlayerSpecialMove>(hunter).unwrap();
        let cooldown = SpecialMoveKind::Charge.base_cooldown()
            * Loadout::default().special_cooldown_multiplier();
        assert_eq!(special.kind, SpecialMoveKind::Charge);
        assert_eq!(special.charge_direction, facing);
        assert_eq!(
            app.world()
                .resource::<SpecialCooldownState>()
                .remaining(SpecialMoveKind::Charge),
            cooldown
        );
    }

    fn special_app(actions: crate::input::ActionState) -> App {
        let mut app = App::new();
        app.insert_resource(SpecialCooldownState::default());
        app.insert_resource(SkillBindings::default());
        app.insert_resource(Loadout::default());
        app.insert_resource(actions);
        app.insert_resource(HitStop::default());
        app.add_event::<CombatSfx>();
        app
    }

    fn resting_attack() -> PlayerAttack {
        let mut attack = PlayerAttack::inactive();
        attack.timer.tick(std::time::Duration::from_secs(1));
        attack
    }

    fn sword_hunter(facing: f32) -> impl Bundle {
        let mut animation = PlayerAnimation::default();
        animation.facing = facing;
        (
            DungeonPlayer,
            Transform::default(),
            EquippedWeapon(WeaponKind::RustySword),
            animation,
            PlayerVelocity {
                x: 0.0,
                y: 0.0,
                grounded: true,
            },
            resting_attack(),
            PlayerBlock::default(),
        )
    }

    #[test]
    fn an_undefined_facing_charges_left() {
        let mut app = special_app(crate::input::action_state_after_pressing(
            &[KeyCode::Digit3],
            &[],
        ));
        let hunter = app.world_mut().spawn(sword_hunter(f32::NAN)).id();
        app.add_systems(Update, start_player_special_moves);
        app.update();
        let special = app.world().get::<PlayerSpecialMove>(hunter).unwrap();
        assert_eq!(special.kind, SpecialMoveKind::Charge);
        assert_eq!(special.charge_direction, -1.0);
    }

    #[test]
    fn a_charge_press_in_the_air_does_not_start() {
        let mut app = special_app(crate::input::action_state_after_pressing(
            &[KeyCode::Digit3],
            &[],
        ));
        let hunter = app.world_mut().spawn(sword_hunter(1.0)).id();
        app.world_mut()
            .get_mut::<PlayerVelocity>(hunter)
            .unwrap()
            .grounded = false;
        app.add_systems(Update, start_player_special_moves);
        app.update();
        assert!(app.world().get::<PlayerSpecialMove>(hunter).is_none());
        assert_eq!(
            app.world()
                .resource::<SpecialCooldownState>()
                .remaining(SpecialMoveKind::Charge),
            0.0
        );
    }

    #[test]
    fn no_skill_press_leaves_the_hunter_without_a_special() {
        let mut app = special_app(crate::input::action_state_after_pressing(&[], &[]));
        let hunter = app.world_mut().spawn(sword_hunter(1.0)).id();
        app.add_systems(Update, start_player_special_moves);
        app.update();
        assert!(app.world().get::<PlayerSpecialMove>(hunter).is_none());
    }

    #[test]
    fn a_charge_in_the_hit_window_damages_the_enemy_it_reaches() {
        let mut app = hit_app();
        let dealt = damage_amount(
            CHARGE_ATTACK_POWER * Loadout::default().attack_power_multiplier(),
            0.0,
        );
        let starting = dealt + dealt;
        app.world_mut().spawn((
            DungeonPlayer,
            Transform::default(),
            charge_at(opening_charge_hit_midpoint()),
        ));
        app.world_mut().spawn((
            Transform::from_xyz(player_half_extents().x, 0.0, 0.0),
            EnemyHitbox(Vec2::splat(TILE * 0.5)),
            Health::new(starting),
            Sprite::default(),
        ));
        app.add_systems(Update, resolve_special_move_hits);
        app.update();
        let health = app
            .world_mut()
            .query::<&Health>()
            .single(app.world())
            .current;
        assert_eq!(health, starting - dealt);
        assert_eq!(app.world().resource::<HitStop>().remaining, HIT_STOP_HEAVY);
    }

    fn hit_app() -> App {
        let mut app = App::new();
        app.insert_resource(Loadout::default());
        app.insert_resource(HitStop::default());
        app.add_event::<CombatSfx>();
        app
    }

    fn opening_charge_hit_midpoint() -> f32 {
        (CHARGE_HIT_START + CHARGE_HIT_END) * 0.5
    }

    fn charge_at(elapsed: f32) -> PlayerSpecialMove {
        let mut timer = Timer::from_seconds(CHARGE_SECS, TimerMode::Once);
        timer.tick(std::time::Duration::from_secs_f32(elapsed));
        PlayerSpecialMove {
            kind: SpecialMoveKind::Charge,
            timer,
            charge_direction: 1.0,
            hit_entities: Vec::new(),
        }
    }

    #[test]
    fn a_charge_misses_an_enemy_standing_apart_from_it() {
        let mut app = hit_app();
        let starting = CHARGE_ATTACK_POWER + CHARGE_ATTACK_POWER;
        app.world_mut().spawn((
            DungeonPlayer,
            Transform::default(),
            charge_at(opening_charge_hit_midpoint()),
        ));
        app.world_mut().spawn((
            Transform::from_xyz(TILE * 40.0, 0.0, 0.0),
            EnemyHitbox(Vec2::splat(TILE * 0.5)),
            Health::new(starting),
            Sprite::default(),
        ));
        app.add_systems(Update, resolve_special_move_hits);
        app.update();
        let health = app
            .world_mut()
            .query::<&Health>()
            .single(app.world())
            .current;
        assert_eq!(health, starting);
        assert_eq!(app.world().resource::<HitStop>().remaining, 0.0);
    }

    #[test]
    fn a_charge_before_its_hit_window_does_not_damage() {
        let mut app = hit_app();
        let starting = CHARGE_ATTACK_POWER + CHARGE_ATTACK_POWER;
        app.world_mut()
            .spawn((DungeonPlayer, Transform::default(), charge_at(0.0)));
        app.world_mut().spawn((
            Transform::from_xyz(player_half_extents().x, 0.0, 0.0),
            EnemyHitbox(Vec2::splat(TILE * 0.5)),
            Health::new(starting),
            Sprite::default(),
        ));
        app.add_systems(Update, resolve_special_move_hits);
        app.update();
        let health = app
            .world_mut()
            .query::<&Health>()
            .single(app.world())
            .current;
        assert_eq!(health, starting);
        assert_eq!(app.world().resource::<HitStop>().remaining, 0.0);
    }
}
