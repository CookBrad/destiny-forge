use bevy::prelude::*;

use crate::combat::{PlayerAttack, PlayerKnockback};

use super::movement::{DungeonPlayer, PlayerVelocity};
use super::sprites::{
    player_frame_rect, DungeonArt, PLAYER_ATTACK_FRAMES, PLAYER_IDLE_FRAMES, PLAYER_RUN_FRAMES,
};

#[derive(Component)]
pub struct PlayerAnimation {
    pub frame: usize,
    pub timer: Timer,
    pub facing: f32,
}

impl Default for PlayerAnimation {
    fn default() -> Self {
        Self {
            frame: 0,
            timer: Timer::from_seconds(0.12, TimerMode::Repeating),
            facing: 1.0,
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
            Option<&PlayerKnockback>,
            &mut PlayerAnimation,
            &mut Sprite,
            &mut Transform,
        ),
        With<DungeonPlayer>,
    >,
) {
    let Ok((velocity, attack, knockback, mut animation, mut sprite, mut transform)) =
        player.get_single_mut()
    else {
        return;
    };

    if attack.is_active() {
        preserve_facing(&mut animation, &transform);

        let progress =
            (attack.timer.elapsed_secs() / attack.weapon.stats().swing_secs).clamp(0.0, 1.0);
        let frame = (progress * PLAYER_ATTACK_FRAMES as f32)
            .floor()
            .min((PLAYER_ATTACK_FRAMES - 1) as f32) as usize;

        apply_sheet_frame(&mut sprite, &art.player_attack, frame);
        apply_facing(&mut transform, animation.facing);
        return;
    }

    if knockback.is_none() && velocity.x.abs() > 1.0 {
        animation.facing = velocity.x.signum();
    }

    apply_facing(&mut transform, animation.facing);

    if velocity.grounded && velocity.x.abs() > 1.0 {
        animation.timer.tick(time.delta());
        if animation.timer.just_finished() {
            animation.frame = (animation.frame + 1) % PLAYER_RUN_FRAMES;
        }
        apply_sheet_frame(&mut sprite, &art.player_run, animation.frame);
        return;
    }

    animation.timer.tick(time.delta());
    if animation.timer.just_finished() {
        animation.frame = (animation.frame + 1) % PLAYER_IDLE_FRAMES;
    }
    apply_sheet_frame(&mut sprite, &art.player_idle, animation.frame);
}

fn apply_sheet_frame(sprite: &mut Sprite, image: &Handle<Image>, frame: usize) {
    sprite.image = image.clone();
    sprite.rect = Some(player_frame_rect(frame));
}

fn apply_facing(transform: &mut Transform, facing: f32) {
    transform.scale = Vec3::new(
        facing * crate::graphics::PIXEL_SCALE,
        crate::graphics::PIXEL_SCALE,
        crate::graphics::PIXEL_SCALE,
    );
}

fn preserve_facing(animation: &mut PlayerAnimation, transform: &Transform) {
    if animation.facing == 0.0 {
        animation.facing = transform.scale.x.signum().max(-1.0).min(1.0);
    }
    if animation.facing == 0.0 {
        animation.facing = 1.0;
    }
}