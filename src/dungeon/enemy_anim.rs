use bevy::prelude::*;

use crate::combat::EnemyCorpse;

use super::enemy::{EnemyKind, KingSlimeBoss, SpriteCanvas};

pub const ENEMY_FRAME_COUNT: usize = 4;

const WALKER_FRAME_SECONDS: f32 = 0.12;
const SLIME_FRAME_SECONDS: f32 = 0.14;
const BAT_FRAME_SECONDS: f32 = 0.10;
const KING_FRAME_SECONDS: f32 = 0.18;

/// Slower than one world pixel per second counts as standing.
/// That is the same deadzone the facing mirror already uses.
const CLEAR_HORIZONTAL_SPEED: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct EnemyPlayback {
    frame: usize,
    elapsed: f32,
    prev_x: f32,
}

#[derive(Component)]
pub struct EnemyAnimation {
    playback: EnemyPlayback,
}

impl EnemyAnimation {
    pub fn standing_at(x: f32) -> Self {
        Self {
            playback: EnemyPlayback {
                frame: 0,
                elapsed: 0.0,
                prev_x: x,
            },
        }
    }
}

pub fn enemy_cell_rect(frame: usize, cell: Vec2) -> Rect {
    let index = frame % ENEMY_FRAME_COUNT;
    let x = index as f32 * cell.x;
    Rect {
        min: Vec2::new(x, 0.0),
        max: Vec2::new(x + cell.x, cell.y),
    }
}

pub fn animate_enemies(
    time: Res<Time>,
    mut enemies: Query<
        (
            &Transform,
            &SpriteCanvas,
            &mut EnemyAnimation,
            &mut Sprite,
            Option<&EnemyKind>,
            Option<&KingSlimeBoss>,
        ),
        Without<EnemyCorpse>,
    >,
) {
    let dt = time.delta_secs();
    for (transform, canvas, mut animation, mut sprite, kind, boss) in &mut enemies {
        apply_enemy_frame(
            &mut animation,
            &mut sprite,
            transform.translation.x,
            canvas.size,
            dt,
            kind.copied(),
            boss.is_some(),
        );
    }
}

fn apply_enemy_frame(
    animation: &mut EnemyAnimation,
    sprite: &mut Sprite,
    x: f32,
    cell: Vec2,
    dt: f32,
    kind: Option<EnemyKind>,
    boss: bool,
) {
    animation.playback = step_enemy_playback(
        animation.playback,
        x,
        dt,
        frame_seconds(kind, boss),
        flaps_while_still(kind),
    );
    sprite.rect = Some(enemy_cell_rect(animation.playback.frame, cell));
}

fn frame_seconds(kind: Option<EnemyKind>, boss: bool) -> f32 {
    if boss {
        return KING_FRAME_SECONDS;
    }
    match kind {
        Some(EnemyKind::Slime) => SLIME_FRAME_SECONDS,
        Some(EnemyKind::Bat) => BAT_FRAME_SECONDS,
        _ => WALKER_FRAME_SECONDS,
    }
}

fn flaps_while_still(kind: Option<EnemyKind>) -> bool {
    matches!(kind, Some(EnemyKind::Bat))
}

fn step_enemy_playback(
    playback: EnemyPlayback,
    x: f32,
    dt: f32,
    frame_seconds: f32,
    always_loop: bool,
) -> EnemyPlayback {
    if dt <= 0.0 {
        return EnemyPlayback {
            prev_x: x,
            ..playback
        };
    }

    let speed = (x - playback.prev_x) / dt;
    let looping = always_loop || speed.abs() > CLEAR_HORIZONTAL_SPEED;
    let (frame, elapsed) = played_frame(playback, dt, frame_seconds, looping);
    EnemyPlayback {
        frame,
        elapsed,
        prev_x: x,
    }
}

fn played_frame(
    playback: EnemyPlayback,
    dt: f32,
    frame_seconds: f32,
    looping: bool,
) -> (usize, f32) {
    if !looping {
        return (0, 0.0);
    }
    advance_frame(playback.frame, playback.elapsed, dt, frame_seconds)
}

fn advance_frame(frame: usize, elapsed: f32, dt: f32, frame_seconds: f32) -> (usize, f32) {
    if frame_seconds <= 0.0 {
        return (frame % ENEMY_FRAME_COUNT, elapsed);
    }

    let mut elapsed = elapsed + dt.max(0.0);
    let mut frame = frame % ENEMY_FRAME_COUNT;
    while elapsed >= frame_seconds {
        elapsed -= frame_seconds;
        frame = (frame + 1) % ENEMY_FRAME_COUNT;
    }
    (frame, elapsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_rects_follow_the_cell_grid_and_motion_gates_playback() {
        let cell = Vec2::new(64.0, 144.0);
        let frame_0 = enemy_cell_rect(0, cell);
        let frame_3 = enemy_cell_rect(3, cell);
        assert_eq!(frame_0.min, Vec2::new(0.0, 0.0));
        assert_eq!(frame_0.max, Vec2::new(64.0, 144.0));
        assert_eq!(frame_3.min, Vec2::new(192.0, 0.0));
        assert_eq!(frame_3.max, Vec2::new(256.0, 144.0));

        let stride = EnemyPlayback {
            frame: 2,
            elapsed: 0.05,
            prev_x: 40.0,
        };
        let still = step_enemy_playback(stride, 40.0, 0.16, WALKER_FRAME_SECONDS, false);
        assert_eq!(still.frame, 0);
        assert_eq!(still.elapsed, 0.0);

        let contact = EnemyPlayback {
            frame: 0,
            elapsed: 0.0,
            prev_x: 0.0,
        };
        let moving = step_enemy_playback(
            contact,
            30.0,
            WALKER_FRAME_SECONDS,
            WALKER_FRAME_SECONDS,
            false,
        );
        assert_eq!(moving.frame, 1);

        let hovering =
            step_enemy_playback(contact, 0.0, BAT_FRAME_SECONDS, BAT_FRAME_SECONDS, true);
        assert_eq!(hovering.frame, 1);
    }
}
