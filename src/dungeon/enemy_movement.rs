use bevy::prelude::*;

use crate::combat::EnemyCorpse;
use crate::graphics::{DUNGEON_FLOOR_Y, ENEMY_DISPLAY_SIZE, TILE};

use super::boss::{BossAttackController, BossCharging};
use super::enemy::{
    EnemyAggro, EnemyKind, EnemyKnockback, GoblinJump, KingSlimeBoss, Patrol, PitClearing,
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
        ),
        (Without<EnemyCorpse>, Without<DungeonPlayer>),
    >,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let dt = time.delta_secs();
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
    ) in &mut enemies
    {
        let enemy_pos = transform.translation.truncate();
        let to_player = player_pos - enemy_pos;
        let distance = to_player.length();
        let airborne = kind.is_some_and(|kind| kind.is_airborne());

        let is_aggro = update_aggro(&mut commands, entity, aggro.as_deref_mut(), distance, dt);
        let (mut velocity, under_knockback) = knockback_velocity(
            knockback.as_deref_mut(),
            dt,
            airborne,
            &mut commands,
            entity,
        );

        let boss_winding_up = boss.is_some() && attack_ctrl.is_some_and(|c| c.windup_timer.is_some());

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
        let goblin_airborne = goblin_jump.as_ref().is_some_and(|jump| jump.is_airborne())
            || transform.translation.y > DUNGEON_FLOOR_Y + ENEMY_DISPLAY_SIZE.y * 0.5 + 0.5;

        if is_goblin && !under_knockback && !charging && !goblin_airborne {
            try_start_goblin_jump(
                is_aggro,
                &to_player,
                &mut patrol,
                &mut transform,
                goblin_jump.as_deref_mut(),
                segments,
                pitfalls,
                &mut velocity,
            );
        }

        let dx = velocity.x * dt;
        if is_goblin && goblin_airborne {
            integrate_goblin_airborne(
                &mut transform,
                goblin_jump.as_deref_mut(),
                pitfalls,
                segments,
                dx,
                dt,
            );
        } else if !airborne && boss.is_none() {
            integrate_ground_enemy(
                &mut transform,
                segments,
                dx,
                velocity.y,
                dt,
                is_aggro,
                charging,
                &mut patrol,
            );
        } else {
            transform.translation.x += dx;
            transform.translation.y += velocity.y * dt;
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

        snap_to_ground_floor(&mut transform, segments, airborne, goblin_airborne);
        reverse_patrol_at_bounds(
            &mut transform,
            &mut patrol,
            charging,
            airborne,
            goblin_airborne,
            is_aggro,
            boss.is_some(),
        );
    }
}

fn update_aggro(
    commands: &mut Commands,
    entity: Entity,
    mut aggro: Option<&mut EnemyAggro>,
    distance: f32,
    dt: f32,
) -> bool {
    if let Some(aggro) = aggro.as_mut() {
        aggro.lock_secs = (aggro.lock_secs - dt).max(0.0);
        if distance > DEAGGRO_RANGE && aggro.lock_secs <= 0.0 {
            commands.entity(entity).remove::<EnemyAggro>();
            return distance < AGGRO_RANGE;
        }
        return true;
    }

    if distance < AGGRO_RANGE {
        commands.entity(entity).insert(EnemyAggro { lock_secs: 0.0 });
        return true;
    }

    false
}

fn knockback_velocity(
    mut knockback: Option<&mut EnemyKnockback>,
    dt: f32,
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
    let decay = (-KNOCKBACK_DECAY * dt).exp();
    knockback.velocity.x *= decay;
    if airborne {
        knockback.velocity.y *= decay;
        knockback.velocity.y += KNOCKBACK_GRAVITY * dt;
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
            kind.map(|kind| kind.chase_speed()).unwrap_or(BOSS_CHASE_SPEED)
        };

        if airborne {
            return to_player.normalize_or_zero() * chase_speed;
        }
        return Vec2::new(to_player.x.signum() * chase_speed, 0.0);
    }

    Vec2::new(patrol.direction * patrol.speed, 0.0)
}

fn try_start_goblin_jump(
    is_aggro: bool,
    to_player: &Vec2,
    patrol: &mut Patrol,
    transform: &mut Transform,
    mut goblin_jump: Option<&mut GoblinJump>,
    segments: &[super::level::PlatformSpec],
    pitfalls: &[super::level::PitfallSpec],
    velocity: &mut Vec2,
) {
    let jump_direction = if is_aggro {
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
    dx: f32,
    dt: f32,
) {
    let Some(jump) = goblin_jump.as_mut() else {
        transform.translation.x += dx;
        return;
    };

    jump.velocity_y += GOBLIN_GRAVITY * dt;
    transform.translation.y += jump.velocity_y * dt;

    let half = ENEMY_DISPLAY_SIZE.y * 0.5;
    let floor_y = DUNGEON_FLOOR_Y + half;
    let x = transform.translation.x;
    let over_pit = is_over_pit_gap(x, pitfalls);
    let still_clearing = jump
        .clearing
        .is_some_and(|clearing| !clearing.cleared(x));

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

    transform.translation.x += dx;
}

fn integrate_ground_enemy(
    transform: &mut Transform,
    segments: &[super::level::PlatformSpec],
    dx: f32,
    velocity_y: f32,
    dt: f32,
    is_aggro: bool,
    charging: bool,
    patrol: &mut Patrol,
) {
    let (new_x, hit_edge) = constrain_ground_walk(transform.translation.x, dx, segments);
    transform.translation.x = new_x;
    if hit_edge && !is_aggro && !charging {
        patrol.direction = -patrol.direction;
    }
    transform.translation.y += velocity_y * dt;
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
) {
    if airborne || goblin_airborne || !is_on_ground_floor(transform.translation.x, segments) {
        return;
    }

    let half = ENEMY_DISPLAY_SIZE.y * 0.5;
    let floor_y = DUNGEON_FLOOR_Y + half;
    if transform.translation.y < floor_y {
        transform.translation.y = floor_y;
    }
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