use bevy::prelude::*;

use crate::combat::EnemyCorpse;
use crate::graphics::{facing_scale, DUNGEON_FLOOR_Y, ENEMY_DISPLAY_SIZE, TILE};

use super::boss::{BossAttackController, BossCharging};
use super::enemy::{
    EnemyAggro, EnemyHitbox, EnemyKind, EnemyKnockback, GoblinJump, KingSlimeBoss, Patrol,
    PitClearing,
};
use super::level::{
    adjacent_pit_from_edge, constrain_ground_walk, ground_walk_bounds_at, is_on_ground_floor,
    is_over_pit_gap, pit_bounds, DungeonLayout,
};
use super::movement::DungeonPlayer;

const AGGRO_RANGE: f32 = 20.0 * TILE;
const DEAGGRO_RANGE: f32 = 26.0 * TILE;
const BOSS_CHASE_SPEED: f32 = 34.0;
const KNOCKBACK_DECAY: f32 = 6.0;
const KNOCKBACK_GRAVITY: f32 = -360.0;
const KNOCKBACK_STOP_SPEED: f32 = 18.0;
const GOBLIN_JUMP_SPEED: f32 = 430.0;
const GOBLIN_JUMP_HSPEED: f32 = 132.0;
const GOBLIN_GRAVITY: f32 = -700.0;
const GOBLIN_PIT_CLEARANCE_Y: f32 = TILE * 1.1;

pub fn move_enemies(
    time: Res<Time>,
    layout: Res<DungeonLayout>,
    mut commands: Commands,
    player: Query<&Transform, With<DungeonPlayer>>,
    mut enemies: Query<
        (
            Entity,
            &mut Transform,
            &mut Patrol,
            Option<&mut EnemyKnockback>,
            Option<&mut EnemyAggro>,
            Option<&EnemyKind>,
            Option<&mut GoblinJump>,
            Option<&KingSlimeBoss>,
            Option<&BossAttackController>,
            Option<&mut BossCharging>,
            &EnemyHitbox,
        ),
        (Without<EnemyCorpse>, Without<DungeonPlayer>),
    >,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let delta_seconds = time.delta_secs();
    let segments = &layout.floor.ground_segments;
    let pitfalls = &layout.floor.pitfalls;

    for (
        entity,
        mut transform,
        mut patrol,
        mut knockback,
        mut aggro,
        kind,
        mut goblin_jump,
        boss,
        attack_ctrl,
        mut charge,
        hitbox,
    ) in &mut enemies
    {
        let enemy_pos = transform.translation.truncate();
        let to_player = player_pos - enemy_pos;
        let distance = to_player.length();
        let airborne = kind.is_some_and(|kind| kind.is_airborne());

        let is_aggro = update_aggro(&mut commands, entity, aggro.as_deref_mut(), distance, delta_seconds);
        let (mut velocity, under_knockback) = knockback_velocity(
            knockback.as_deref_mut(),
            delta_seconds,
            airborne,
            &mut commands,
            entity,
        );

        let boss_winding_up =
            boss.is_some() && attack_ctrl.is_some_and(|c| c.windup_timer.is_some());

        let charging = apply_boss_charge(
            &mut commands,
            entity,
            charge.as_deref_mut(),
            &time,
            &mut velocity,
        );

        if !under_knockback && !boss_winding_up && !charging {
            velocity = idle_or_chase_velocity(
                is_aggro,
                boss.is_some(),
                kind,
                airborne,
                &to_player,
                &patrol,
            );
        }

        let is_goblin = kind.is_some_and(|kind| *kind == EnemyKind::Goblin);
        let standing_y = standing_center_y(hitbox.0.y * 2.0);
        let goblin_airborne = goblin_jump.as_ref().is_some_and(|jump| jump.is_airborne())
            || transform.translation.y > standing_y + 0.5;

        if is_goblin && !under_knockback && !charging && !goblin_airborne {
            try_start_goblin_jump(
                if is_aggro { GoblinAim::TowardPlayer } else { GoblinAim::AlongPatrol },
                &to_player,
                &mut patrol,
                &mut transform,
                goblin_jump.as_deref_mut(),
                segments,
                pitfalls,
                &mut velocity,
            );
        }

        let horizontal_distance = velocity.x * delta_seconds;
        if is_goblin && goblin_airborne {
            integrate_goblin_airborne(
                &mut transform,
                goblin_jump.as_deref_mut(),
                pitfalls,
                segments,
                horizontal_distance,
                delta_seconds,
            );
        } else if !airborne && boss.is_none() {
            integrate_ground_enemy(
                &mut transform,
                segments,
                horizontal_distance,
                velocity.y,
                delta_seconds,
                if charging { GroundStepMode::Charging }
                    else if is_aggro { GroundStepMode::Chasing }
                    else { GroundStepMode::Patrolling },
                &mut patrol,
            );
        } else {
            transform.translation.x += horizontal_distance;
            transform.translation.y += velocity.y * delta_seconds;
        }

        if charging {
            clamp_boss_charge(
                &mut commands,
                entity,
                &mut transform,
                &patrol,
                charge.as_deref_mut(),
            );
        }

        snap_to_ground_floor(
            &mut transform,
            segments,
            airborne,
            goblin_airborne,
            hitbox.0.y * 2.0,
        );
        reverse_patrol_at_bounds(
            &mut transform,
            &mut patrol,
            charging,
            airborne,
            goblin_airborne,
            is_aggro,
            boss.is_some(),
        );
        if boss.is_none() {
            apply_horizontal_facing(&mut transform, velocity.x);
        }
    }
}

fn apply_horizontal_facing(transform: &mut Transform, velocity_x: f32) {
    if let Some(scale) = facing_from_motion(velocity_x) {
        transform.scale = scale;
    }
}

fn facing_from_motion(velocity_x: f32) -> Option<Vec3> {
    if velocity_x.abs() < 1.0 {
        None
    } else {
        Some(facing_scale(velocity_x))
    }
}

fn update_aggro(
    commands: &mut Commands,
    entity: Entity,
    mut aggro: Option<&mut EnemyAggro>,
    distance: f32,
    delta_seconds: f32,
) -> bool {
    if let Some(aggro) = aggro.as_mut() {
        aggro.lock_secs = (aggro.lock_secs - delta_seconds).max(0.0);
        if distance > DEAGGRO_RANGE && aggro.lock_secs <= 0.0 {
            commands.entity(entity).remove::<EnemyAggro>();
            return distance < AGGRO_RANGE;
        }
        return true;
    }

    if distance < AGGRO_RANGE {
        commands
            .entity(entity)
            .insert(EnemyAggro { lock_secs: 0.0 });
        return true;
    }

    false
}

fn knockback_velocity(
    mut knockback: Option<&mut EnemyKnockback>,
    delta_seconds: f32,
    airborne: bool,
    commands: &mut Commands,
    entity: Entity,
) -> (Vec2, bool) {
    let Some(knockback) = knockback.as_mut() else {
        return (Vec2::ZERO, false);
    };

    let knockback_active = if airborne {
        knockback.velocity.length() > KNOCKBACK_STOP_SPEED
    } else {
        knockback.velocity.x.abs() > KNOCKBACK_STOP_SPEED
    };

    if !knockback_active {
        commands.entity(entity).remove::<EnemyKnockback>();
        return (Vec2::ZERO, false);
    }

    let velocity = knockback.velocity;
    let decay = (-KNOCKBACK_DECAY * delta_seconds).exp();
    knockback.velocity.x *= decay;
    if airborne {
        knockback.velocity.y *= decay;
        knockback.velocity.y += KNOCKBACK_GRAVITY * delta_seconds;
    } else {
        knockback.velocity.y = 0.0;
    }

    let still_active = if airborne {
        knockback.velocity.length() > KNOCKBACK_STOP_SPEED
    } else {
        knockback.velocity.x.abs() > KNOCKBACK_STOP_SPEED
    };
    if !still_active {
        commands.entity(entity).remove::<EnemyKnockback>();
    }

    (velocity, true)
}

fn apply_boss_charge(
    commands: &mut Commands,
    entity: Entity,
    mut charge: Option<&mut BossCharging>,
    time: &Time,
    velocity: &mut Vec2,
) -> bool {
    let Some(charge) = charge.as_mut() else {
        return false;
    };

    charge.timer.tick(time.delta());
    *velocity = charge.velocity;
    if charge.timer.finished() {
        commands.entity(entity).remove::<BossCharging>();
        return false;
    }
    true
}

fn idle_or_chase_velocity(
    is_aggro: bool,
    is_boss: bool,
    kind: Option<&EnemyKind>,
    airborne: bool,
    to_player: &Vec2,
    patrol: &Patrol,
) -> Vec2 {
    if is_aggro {
        let chase_speed = if is_boss {
            BOSS_CHASE_SPEED
        } else {
            kind.map(|kind| kind.chase_speed())
                .unwrap_or(BOSS_CHASE_SPEED)
        };

        if airborne {
            return to_player.normalize_or_zero() * chase_speed;
        }
        return Vec2::new(to_player.x.signum() * chase_speed, 0.0);
    }

    Vec2::new(patrol.direction * patrol.speed, 0.0)
}

enum GoblinAim { TowardPlayer, AlongPatrol }

fn try_start_goblin_jump(
    aim: GoblinAim,
    to_player: &Vec2,
    patrol: &mut Patrol,
    transform: &mut Transform,
    mut goblin_jump: Option<&mut GoblinJump>,
    segments: &[super::level::PlatformSpec],
    pitfalls: &[super::level::PitfallSpec],
    velocity: &mut Vec2,
) {
    let jump_direction = if matches!(aim, GoblinAim::TowardPlayer) {
        to_player.x.signum()
    } else {
        patrol.direction
    };
    if jump_direction == 0.0 {
        return;
    }

    let x = transform.translation.x;
    let Some(pit) = adjacent_pit_from_edge(x, jump_direction, segments, pitfalls) else {
        return;
    };

    let (pit_left, pit_right) = pit_bounds(&pit);
    if let Some((walk_min, walk_max)) = ground_walk_bounds_at(x, segments) {
        transform.translation.x = if jump_direction > 0.0 {
            walk_max
        } else {
            walk_min
        };
    }
    if let Some(jump) = goblin_jump.as_mut() {
        jump.velocity_y = GOBLIN_JUMP_SPEED;
        jump.clearing = Some(PitClearing {
            pit_left,
            pit_right,
            direction: jump_direction,
        });
    }
    velocity.x = jump_direction * GOBLIN_JUMP_HSPEED;
}

fn integrate_goblin_airborne(
    transform: &mut Transform,
    mut goblin_jump: Option<&mut GoblinJump>,
    pitfalls: &[super::level::PitfallSpec],
    segments: &[super::level::PlatformSpec],
    horizontal_distance: f32,
    delta_seconds: f32,
) {
    let Some(jump) = goblin_jump.as_mut() else {
        transform.translation.x += horizontal_distance;
        return;
    };

    jump.velocity_y += GOBLIN_GRAVITY * delta_seconds;
    transform.translation.y += jump.velocity_y * delta_seconds;

    let half = ENEMY_DISPLAY_SIZE.y * 0.5;
    let floor_y = DUNGEON_FLOOR_Y + half;
    let x = transform.translation.x;
    let over_pit = is_over_pit_gap(x, pitfalls);
    let still_clearing = jump.clearing.is_some_and(|clearing| !clearing.cleared(x));

    if still_clearing || over_pit {
        let min_y = floor_y + GOBLIN_PIT_CLEARANCE_Y;
        if transform.translation.y < min_y {
            transform.translation.y = min_y;
            jump.velocity_y = jump.velocity_y.max(0.0);
        }
    } else if jump.velocity_y <= 0.0
        && transform.translation.y <= floor_y
        && is_on_ground_floor(x, segments)
    {
        transform.translation.y = floor_y;
        jump.velocity_y = 0.0;
        jump.clearing = None;
    }

    transform.translation.x += horizontal_distance;
}

enum GroundStepMode { Patrolling, Chasing, Charging }
struct GroundEnemyStep<'a> {
    transform: &'a mut Transform, segments: &'a [super::level::PlatformSpec],
    horizontal_distance: f32, velocity_y: f32, delta_seconds: f32,
    motion: GroundStepMode, patrol: &'a mut Patrol,
}

fn integrate_ground_enemy(
    transform: &mut Transform,
    segments: &[super::level::PlatformSpec],
    horizontal_distance: f32,
    velocity_y: f32,
    delta_seconds: f32,
    motion: GroundStepMode,
    patrol: &mut Patrol,
) {
    let GroundEnemyStep { transform, segments, horizontal_distance, velocity_y, delta_seconds, motion, patrol } =
        GroundEnemyStep { transform, segments, horizontal_distance, velocity_y, delta_seconds, motion, patrol };
    let (new_x, hit_edge) = constrain_ground_walk(transform.translation.x, horizontal_distance, segments);
    transform.translation.x = new_x;
    if hit_edge && matches!(motion, GroundStepMode::Patrolling) {
        patrol.direction = -patrol.direction;
    }
    transform.translation.y += velocity_y * delta_seconds;
}

fn clamp_boss_charge(
    commands: &mut Commands,
    entity: Entity,
    transform: &mut Transform,
    patrol: &Patrol,
    charge: Option<&mut BossCharging>,
) {
    if charge.is_none() {
        return;
    }

    let hit_left = transform.translation.x <= patrol.min_x;
    let hit_right = transform.translation.x >= patrol.max_x;
    if hit_left || hit_right {
        transform.translation.x = transform.translation.x.clamp(patrol.min_x, patrol.max_x);
        commands.entity(entity).remove::<BossCharging>();
    }
}

fn snap_to_ground_floor(
    transform: &mut Transform,
    segments: &[super::level::PlatformSpec],
    airborne: bool,
    goblin_airborne: bool,
    body_height: f32,
) {
    if airborne || goblin_airborne || !is_on_ground_floor(transform.translation.x, segments) {
        return;
    }

    let floor_y = standing_center_y(body_height);
    if transform.translation.y < floor_y {
        transform.translation.y = floor_y;
    }
}

fn standing_center_y(body_height: f32) -> f32 {
    DUNGEON_FLOOR_Y + body_height * 0.5
}

fn reverse_patrol_at_bounds(
    transform: &mut Transform,
    patrol: &mut Patrol,
    charging: bool,
    airborne: bool,
    goblin_airborne: bool,
    is_aggro: bool,
    is_boss: bool,
) {
    if charging || airborne || goblin_airborne || is_aggro || is_boss {
        return;
    }

    if transform.translation.x <= patrol.min_x {
        transform.translation.x = patrol.min_x;
        patrol.direction = 1.0;
    } else if transform.translation.x >= patrol.max_x {
        transform.translation.x = patrol.max_x;
        patrol.direction = -1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::level::{PitfallSpec, PlatformSpec};
    use crate::graphics::KING_SLIME_GAMEPLAY_SCALE;

    fn ground_step(start_x: f32, horizontal_distance: f32, motion: GroundStepMode) -> (f32, f32, f32) {
        let segments = [PlatformSpec { left: 0.0, width_tiles: 8, top_y: DUNGEON_FLOOR_Y }];
        let mut patrol = Patrol { min_x: 0.0, max_x: 10_000.0, speed: 30.0, direction: 1.0 };
        let mut transform = Transform::from_xyz(start_x, 20.0, 0.0);
        integrate_ground_enemy(&mut transform, &segments, horizontal_distance, -6.0, 0.5, motion, &mut patrol);
        (transform.translation.x, transform.translation.y, patrol.direction)
    }

    fn goblin_leap(x: f32, aim: GoblinAim, patrol_direction: f32, player_offset_x: f32) -> (f32, Vec2, GoblinJump) {
        let segments = [
            PlatformSpec { left: 0.0, width_tiles: 8, top_y: DUNGEON_FLOOR_Y },
            PlatformSpec { left: 12.0 * TILE, width_tiles: 10, top_y: DUNGEON_FLOOR_Y },
        ];
        let pitfalls = [PitfallSpec { left: 8.0 * TILE, width_tiles: 4 }];
        let mut patrol = Patrol { min_x: 0.0, max_x: 10_000.0, speed: 30.0, direction: patrol_direction };
        let mut transform = Transform::from_xyz(x, 10.0, 0.0);
        let mut jump = GoblinJump::default();
        let mut velocity = Vec2::new(3.0, 4.0);
        try_start_goblin_jump(aim, &Vec2::new(player_offset_x, 0.0), &mut patrol, &mut transform, Some(&mut jump), &segments, &pitfalls, &mut velocity);
        (transform.translation.x, velocity, jump)
    }

    #[test]
    fn patrolling_ground_enemy_reverses_when_the_step_hits_the_segment_edge() {
        assert_eq!(ground_step(239.0, 8.0, GroundStepMode::Patrolling), (240.0, 17.0, -1.0));
        assert_eq!(ground_step(40.0, 5.0, GroundStepMode::Patrolling), (45.0, 17.0, 1.0));
    }

    #[test]
    fn ground_enemy_keeps_direction_while_chasing_or_charging() {
        assert_eq!(ground_step(239.0, 8.0, GroundStepMode::Chasing), (240.0, 17.0, 1.0));
        assert_eq!(ground_step(17.0, -8.0, GroundStepMode::Charging), (16.0, 17.0, 1.0));
    }

    #[test]
    fn goblin_aiming_toward_the_player_leaps_from_the_pit_edge() {
        let (x, velocity, jump) = goblin_leap(240.0, GoblinAim::TowardPlayer, -1.0, 80.0);
        let clearing = jump.clearing.expect("pit clearing");
        assert_eq!((x, velocity, jump.velocity_y), (240.0, Vec2::new(GOBLIN_JUMP_HSPEED, 4.0), GOBLIN_JUMP_SPEED));
        assert_eq!((clearing.pit_left, clearing.pit_right, clearing.direction), (8.0 * TILE, 12.0 * TILE, 1.0));
    }

    #[test]
    fn goblin_aiming_along_patrol_leaps_when_the_player_stands_opposite() {
        let (x, velocity, jump) = goblin_leap(240.0, GoblinAim::AlongPatrol, 1.0, -80.0);
        assert_eq!((x, velocity.x, jump.velocity_y, jump.clearing.expect("pit clearing").direction), (240.0, GOBLIN_JUMP_HSPEED, GOBLIN_JUMP_SPEED, 1.0));
    }

    #[test]
    fn goblin_standing_inside_the_segment_keeps_its_ground_velocity() {
        let (x, velocity, jump) = goblin_leap(80.0, GoblinAim::TowardPlayer, 1.0, 40.0);
        assert_eq!((x, velocity, jump.velocity_y, jump.clearing.is_none()), (80.0, Vec2::new(3.0, 4.0), 0.0, true));
    }

    #[test]
    fn standard_enemy_stands_one_tile_above_the_floor_line() {
        let center = standing_center_y(ENEMY_DISPLAY_SIZE.y);
        assert_eq!(center, DUNGEON_FLOOR_Y + ENEMY_DISPLAY_SIZE.y * 0.5);
        assert_eq!(center - ENEMY_DISPLAY_SIZE.y * 0.5, DUNGEON_FLOOR_Y);
    }

    #[test]
    fn aggro_distances_stay_on_the_tile() {
        assert_eq!(AGGRO_RANGE, 20.0 * TILE);
        assert_eq!(DEAGGRO_RANGE, 26.0 * TILE);
    }

    #[test]
    fn facing_follows_horizontal_motion_at_scale_one() {
        assert_eq!(facing_from_motion(0.0), None);
        assert_eq!(facing_from_motion(0.5), None);
        assert_eq!(facing_from_motion(22.0), Some(Vec3::ONE));
        let left = facing_from_motion(-35.0).expect("left facing");
        assert_eq!(left.x, -1.0);
        assert_eq!(left.y, 1.0);
        assert_eq!(left.z, 1.0);
    }

    #[test]
    fn king_slime_standing_height_keeps_feet_on_the_floor() {
        let height = ENEMY_DISPLAY_SIZE.y * KING_SLIME_GAMEPLAY_SCALE;
        let center = standing_center_y(height);
        let half = height * 0.5;
        assert_eq!(center - half, DUNGEON_FLOOR_Y);
        assert_eq!(half, TILE);
    }
}
