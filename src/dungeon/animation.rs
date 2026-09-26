use std::time::Duration;

use bevy::prelude::*;

use crate::combat::{
    PlayerAttack, PlayerDeath, PlayerKnockback, PlayerSpecialMove, SpecialMoveKind,
};

use super::hunter_pose::{
    attack_cell, attack_phase, is_moving, locomotion_pose, next_played_frame, playback_for,
    pose_anchor, sheet_for, HunterPose,
};
use super::movement::{DungeonPlayer, PlayerVelocity};
use super::sprites::{player_frame_rect, DungeonArt};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Clip {
    Idle,
    Run,
    Jump,
    Fall,
}

#[derive(Component)]
pub struct PlayerAnimation {
    pub frame: usize,
    pub timer: Timer,
    pub facing: f32,
    clip: Option<Clip>,
}

impl Default for PlayerAnimation {
    fn default() -> Self {
        Self {
            frame: 0,
            timer: Timer::from_seconds(
                super::hunter_pose::IDLE_FRAME_SECONDS,
                TimerMode::Repeating,
            ),
            facing: 1.0,
            clip: None,
        }
    }
}

pub fn animate_player(
    time: Res<Time>,
    art: Res<DungeonArt>,
    mut player: Query<
        (
            &PlayerVelocity,
            &PlayerAttack,
            Option<&PlayerSpecialMove>,
            Option<&PlayerKnockback>,
            &mut PlayerAnimation,
            &mut Sprite,
            &mut Transform,
        ),
        (With<DungeonPlayer>, Without<PlayerDeath>),
    >,
) {
    let Ok((velocity, attack, special, knockback, mut animation, mut sprite, mut transform)) =
        player.get_single_mut()
    else {
        return;
    };

    let pose = current_pose(velocity, attack, special);
    update_facing(
        &mut animation,
        &mut transform,
        velocity,
        attack,
        special,
        knockback,
    );
    apply_pose(&mut animation, &mut sprite, &art, pose, time.delta_secs());
}

fn current_pose(
    velocity: &PlayerVelocity,
    attack: &PlayerAttack,
    special: Option<&PlayerSpecialMove>,
) -> HunterPose {
    if let Some(special) = special.filter(|special| special.is_active()) {
        if running_spin(special, velocity) {
            return HunterPose::Run;
        }
        return special_pose(special);
    }
    if attack.is_active() {
        return combo_pose(attack);
    }
    locomotion_pose(velocity.grounded, velocity.y, velocity.x)
}

fn running_spin(special: &PlayerSpecialMove, velocity: &PlayerVelocity) -> bool {
    special.kind == SpecialMoveKind::Spin && velocity.grounded && is_moving(velocity.x)
}

fn combo_pose(attack: &PlayerAttack) -> HunterPose {
    let step = attack.step();
    let duration = step.duration.max(0.01);
    let progress = (attack.timer.elapsed_secs() / duration).clamp(0.0, 1.0);
    let phase = attack_phase(
        attack.step_index,
        progress,
        step.hit_start,
        step.hit_end,
        duration,
    );
    HunterPose::Attack(phase)
}

fn special_pose(special: &PlayerSpecialMove) -> HunterPose {
    let duration = special.duration().max(0.01);
    let progress = (special.timer.elapsed_secs() / duration).clamp(0.0, 1.0);
    let (hit_start, hit_end) = special.hit_window();
    HunterPose::Attack(attack_phase(0, progress, hit_start, hit_end, duration))
}

fn update_facing(
    animation: &mut PlayerAnimation,
    transform: &mut Transform,
    velocity: &PlayerVelocity,
    attack: &PlayerAttack,
    special: Option<&PlayerSpecialMove>,
    knockback: Option<&PlayerKnockback>,
) {
    let special_active = special.is_some_and(|special| special.is_active());
    if special_active {
        if knockback.is_none() && is_moving(velocity.x) {
            animation.facing = velocity.x.signum();
        } else {
            preserve_facing(animation, transform);
        }
    } else if attack.is_active() {
        preserve_facing(animation, transform);
    } else if knockback.is_none() && is_moving(velocity.x) {
        animation.facing = velocity.x.signum();
    }
    apply_facing(transform, animation.facing);
}

fn apply_pose(
    animation: &mut PlayerAnimation,
    sprite: &mut Sprite,
    art: &DungeonArt,
    pose: HunterPose,
    dt: f32,
) {
    let cell = sheet_cell(animation, pose, dt);
    sprite.image = art.hunter_image(sheet_for(pose));
    sprite.rect = Some(player_frame_rect(cell));
    sprite.anchor = pose_anchor(pose);
}

fn sheet_cell(animation: &mut PlayerAnimation, pose: HunterPose, dt: f32) -> usize {
    if let HunterPose::Attack(phase) = pose {
        animation.clip = None;
        return attack_cell(phase);
    }

    let clip = clip_of(pose);
    if animation.clip != Some(clip) {
        animation.frame = 0;
        animation.timer.reset();
        animation.clip = Some(clip);
    }

    let playback = playback_for(pose);
    animation
        .timer
        .set_duration(Duration::from_secs_f32(playback.frame_seconds));
    let (frame, elapsed) = next_played_frame(
        animation.frame,
        animation.timer.elapsed_secs(),
        playback,
        dt,
    );
    animation.frame = frame;
    animation
        .timer
        .set_elapsed(Duration::from_secs_f32(elapsed));
    playback.origin_cell + frame
}

fn clip_of(pose: HunterPose) -> Clip {
    match pose {
        HunterPose::Idle => Clip::Idle,
        HunterPose::Run => Clip::Run,
        HunterPose::Jump => Clip::Jump,
        HunterPose::Fall => Clip::Fall,
        HunterPose::Attack(_) => unreachable!("attack cells are not a locomotion clip"),
    }
}

fn apply_facing(transform: &mut Transform, facing: f32) {
    transform.scale = crate::graphics::facing_scale(facing);
}

fn preserve_facing(animation: &mut PlayerAnimation, transform: &Transform) {
    if animation.facing == 0.0 {
        animation.facing = transform.scale.x.signum().clamp(-1.0, 1.0);
    }
    if animation.facing == 0.0 {
        animation.facing = 1.0;
    }
}
