use bevy::prelude::*;

use std::f32::consts::FRAC_PI_2;

use crate::audio::CombatSfx;
use crate::dungeon::{
    DungeonArt, DungeonPlayer, EnemyHitbox, EnemyKind, EnemyKnockback, KingSlimeBoss,
    PlayerAnimation, PlayerVelocity, PLAYER_IDLE_FRAMES, PLAYER_RUN_FRAMES,
};
use crate::player::Loadout;

use super::hit_stop::{HitStop, HIT_STOP_HEAVY, HIT_STOP_LIGHT};
use super::hitbox::{
    animation_facing, enemy_aabb, hitbox_overlaps, sword_blade_center_local, sword_swing_aabb,
    HitRect,
};
use super::hits::{apply_enemy_strike, EnemyStrike};
use super::player_block::PlayerBlock;
use super::skills::{SkillBindings, SkillKind};
use super::special_moves::{player_is_busy, PlayerSpecialMove};

use crate::dungeon::player_half_extents;
use super::health::{damage_amount, Health};
use super::weapon::{ComboStep, EquippedWeapon, HitShape, WeaponKind};

#[derive(Component)]
pub struct PlayerAttack {
    pub timer: Timer,
    pub weapon: WeaponKind,
    pub step_index: usize,
    pub hit_entities: Vec<Entity>,
    /// Buffered input to continue the combo on the next step.
    pub queue_next: bool,
}

impl PlayerAttack {
    pub fn inactive() -> Self {
        Self {
            timer: Timer::from_seconds(0.0, TimerMode::Once),
            weapon: WeaponKind::RustySword,
            step_index: 0,
            hit_entities: Vec::new(),
            queue_next: false,
        }
    }

    pub fn is_active(&self) -> bool {
        !self.timer.finished()
    }

    pub fn step(&self) -> ComboStep {
        let steps = self.weapon.moveset().steps;
        steps[self.step_index.min(steps.len().saturating_sub(1))]
    }

    pub fn in_hit_window(&self) -> bool {
        if !self.is_active() {
            return false;
        }
        let elapsed = self.timer.elapsed_secs();
        let step = self.step();
        elapsed >= step.hit_start && elapsed <= step.hit_end
    }

    pub fn in_chain_window(&self) -> bool {
        if !self.is_active() {
            return false;
        }
        let step = self.step();
        let elapsed = self.timer.elapsed_secs();
        elapsed >= step.chain_start && elapsed <= step.duration
    }

    pub fn can_chain(&self) -> bool {
        self.in_chain_window() && self.step_index + 1 < self.weapon.moveset().steps.len()
    }

    pub fn step_power(&self, loadout: &Loadout) -> f32 {
        self.weapon.base_power() * self.step().power_mult * loadout.attack_power_multiplier()
    }
}

/// Active sword swing volume during the hit window (for projectile parries).
pub fn player_sword_hit_rect(player: &Transform, attack: &PlayerAttack) -> Option<HitRect> {
    if !attack.in_hit_window() {
        return None;
    }

    match attack.step().shape {
        HitShape::SwordArc => Some(sword_swing_aabb(
            player,
            swing_angle(sword_arc_progress(attack)),
        )),
        HitShape::SpearThrust | HitShape::SpearLunge => None,
    }
}

#[derive(Component)]
pub struct EnemyCorpse;

#[derive(Component)]
pub struct HitFlash {
    pub timer: Timer,
}

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
    hit_stop: Res<HitStop>,
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
    if hit_stop.is_active() {
        return;
    }

    let Ok((entity, weapon, mut attack, block, special)) = player.get_single_mut() else {
        return;
    };

    if !SkillBindings::skill_just_pressed(&keyboard, &bindings, SkillKind::Attack) {
        return;
    }

    // Buffer next combo step during chain window.
    if attack.is_active() {
        if attack.can_chain() {
            attack.queue_next = true;
        }
        return;
    }

    if player_is_busy(&attack, block, special) {
        return;
    }

    begin_combo_step(
        &mut commands,
        &mut sfx,
        &art,
        entity,
        &mut attack,
        weapon.0,
        0,
    );
}

fn begin_combo_step(
    commands: &mut Commands,
    sfx: &mut EventWriter<CombatSfx>,
    art: &DungeonArt,
    entity: Entity,
    attack: &mut PlayerAttack,
    weapon: WeaponKind,
    step_index: usize,
) {
    let steps = weapon.moveset().steps;
    let step = steps[step_index.min(steps.len() - 1)];

    attack.weapon = weapon;
    attack.step_index = step_index;
    attack.hit_entities.clear();
    attack.queue_next = false;
    attack.timer = Timer::from_seconds(step.duration, TimerMode::Once);
    attack.timer.reset();
    sfx.send(CombatSfx::SwordSwing);

    // Refresh swing FX for this step.
    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: art.weapon_anime_sword.clone(),
                ..default()
            },
            Transform {
                translation: pose_for_step(step, 0.0).translation,
                rotation: pose_for_step(step, 0.0).rotation,
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

    let visible = !player_is_busy(attack, block, special) && weapon.0.is_sword();
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
            commands.entity(entity).try_despawn();
        }
        return;
    }

    let step = attack.step();
    let progress = step_visual_progress(attack);

    for (_, _swing, mut transform) in &mut swings {
        let pose = pose_for_step(step, progress);
        transform.translation = pose.translation;
        transform.rotation = pose.rotation;
    }
}

pub fn tick_player_attack(
    time: Res<Time>,
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    art: Res<DungeonArt>,
    hit_stop: Res<HitStop>,
    mut player: Query<(Entity, &mut PlayerAttack), With<DungeonPlayer>>,
    swing_fx: Query<Entity, With<WeaponSwingFx>>,
) {
    if hit_stop.is_active() {
        return;
    }

    let Ok((entity, mut attack)) = player.get_single_mut() else {
        return;
    };

    if !attack.is_active() {
        return;
    }

    attack.timer.tick(time.delta());

    if !attack.timer.finished() {
        return;
    }

    // Advance combo or end.
    let weapon = attack.weapon;
    let next = attack.step_index + 1;
    if attack.queue_next && next < weapon.moveset().steps.len() {
        for fx in &swing_fx {
            commands.entity(fx).try_despawn();
        }
        begin_combo_step(
            &mut commands,
            &mut sfx,
            &art,
            entity,
            &mut attack,
            weapon,
            next,
        );
    } else {
        attack.queue_next = false;
        attack.step_index = 0;
    }
}

pub fn resolve_weapon_hits(
    mut commands: Commands,
    mut sfx: EventWriter<CombatSfx>,
    mut hit_stop: ResMut<HitStop>,
    loadout: Res<Loadout>,
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

    let power = attack.step_power(&loadout);
    let facing = animation_facing(player_transform);
    let hitbox = swing_hitbox(player_transform, &attack, facing);
    let mut landed = false;

    for (entity, transform, hitbox_extents, mut health, mut sprite, boss, kind) in &mut enemies {
        if attack.hit_entities.contains(&entity) || health.is_dead() {
            continue;
        }

        if !hitbox_overlaps(hitbox, enemy_bounds(transform, hitbox_extents.0)) {
            continue;
        }

        apply_enemy_strike(
            &mut commands,
            &mut sfx,
            entity,
            &mut health,
            &mut sprite,
            &mut attack.hit_entities,
            EnemyStrike {
                damage: damage_amount(power, 0.0),
                sfx: CombatSfx::SwordHit,
                knockback: EnemyKnockback::away_from_player(
                    player_transform,
                    transform,
                    if boss.is_some() { 0.35 } else { 1.0 },
                    kind.is_some_and(|kind| kind.is_airborne()),
                ),
            },
        );
        landed = true;
    }

    if landed {
        let heavy = attack.step().power_mult >= 1.35;
        hit_stop.request(if heavy { HIT_STOP_HEAVY } else { HIT_STOP_LIGHT });
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
    let step = attack.step();
    match step.shape {
        HitShape::SwordArc => sword_swing_aabb(player, swing_angle(sword_arc_progress(attack))),
        HitShape::SpearThrust | HitShape::SpearLunge => {
            spear_thrust_hitbox(player, step.reach, facing, step.shape)
        }
    }
}

fn spear_thrust_hitbox(player: &Transform, reach: f32, facing: f32, shape: HitShape) -> HitRect {
    let half = player_half_extents();
    let center = player.translation.truncate();
    let front = center.x + facing * half.x * 0.6;
    let tip_x = center.x + facing * reach;
    let height = match shape {
        HitShape::SpearLunge => half.y * 0.55,
        _ => half.y * 0.4,
    };

    HitRect {
        min_x: front.min(tip_x),
        max_x: front.max(tip_x),
        min_y: center.y - height,
        max_y: center.y + height * 0.85,
    }
}

fn enemy_bounds(transform: &Transform, half: Vec2) -> HitRect {
    enemy_aabb(transform.translation.truncate(), half)
}

fn sword_arc_progress(attack: &PlayerAttack) -> f32 {
    let step = attack.step();
    (attack.timer.elapsed_secs() / step.duration * SWORD_ARC_SPEED).clamp(0.0, 1.0)
}

fn step_visual_progress(attack: &PlayerAttack) -> f32 {
    let step = attack.step();
    match step.shape {
        HitShape::SwordArc => sword_arc_progress(attack),
        HitShape::SpearThrust | HitShape::SpearLunge => {
            (attack.timer.elapsed_secs() / step.duration).clamp(0.0, 1.0)
        }
    }
}

/// Vertical sword starts raised and sweeps 90° downward in local space.
fn swing_angle(progress: f32) -> f32 {
    -progress * FRAC_PI_2
}

fn pose_for_step(step: ComboStep, progress: f32) -> SwingPose {
    match step.shape {
        HitShape::SwordArc => {
            let angle = swing_angle(progress);
            let center = sword_blade_center_local(angle);
            SwingPose {
                translation: Vec3::new(center.x, center.y, 0.5),
                rotation: Quat::from_rotation_z(angle),
            }
        }
        HitShape::SpearThrust | HitShape::SpearLunge => {
            // Horizontal poke: extend forward over the thrust.
            let extend = progress.clamp(0.0, 1.0);
            let forward = 6.0 + extend * (if matches!(step.shape, HitShape::SpearLunge) {
                14.0
            } else {
                10.0
            });
            SwingPose {
                translation: Vec3::new(forward, 2.0, 0.5),
                rotation: Quat::from_rotation_z(-FRAC_PI_2 * 0.95),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Loadout;

    #[test]
    fn combo_chains_within_window() {
        let mut attack = PlayerAttack::inactive();
        attack.weapon = WeaponKind::RustySword;
        attack.step_index = 0;
        let duration = WeaponKind::RustySword.moveset().steps[0].duration;
        attack.timer = Timer::from_seconds(duration, TimerMode::Once);
        // Tick past chain_start (0.14)
        for _ in 0..20 {
            attack.timer.tick(std::time::Duration::from_secs_f32(0.01));
        }
        assert!(attack.is_active());
        assert!(attack.can_chain());
    }

    #[test]
    fn step_power_scales_with_set_bonus() {
        let mut attack = PlayerAttack::inactive();
        attack.weapon = WeaponKind::RustySword;
        attack.step_index = 0;
        let base = Loadout::default();
        let mut full = Loadout::default();
        use crate::player::{ArmorKind, ArmorSlots};
        full.armor = ArmorSlots {
            head: Some(ArmorKind::SlimeHelm),
            chest: Some(ArmorKind::SlimeMail),
            arms: Some(ArmorKind::SlimeGauntlets),
            legs: Some(ArmorKind::SlimeGreaves),
        };
        assert!(attack.step_power(&full) > attack.step_power(&base));
    }
}
