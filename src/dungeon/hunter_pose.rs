use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::graphics::{hunter_body_anchor, HUNTER_CELL_PX, TILE};

/// Signed idle sole, relative to the body origin. The cell is 160 px tall and that
/// sole sits one pixel above the cell bottom, so the contact is 79 px below the origin.
const IDLE_SOLE_WORLD_Y: f32 = -(HUNTER_CELL_PX.y * 0.5 - IDLE_SOLE_GAP);

const IDLE_PIN_X: f32 = 86.0;
const IDLE_SOLE_GAP: f32 = 1.0;
const RUN_PIN_X: f32 = 65.0;
const RUN_SOLE_GAP: f32 = 1.0;
const CHAMBER_PIN_X: f32 = 69.0;
const CHAMBER_SOLE_GAP: f32 = 0.0;
const THRUST_PIN_X: f32 = 81.5;
const THRUST_SOLE_GAP: f32 = 0.0;

/// Sheet file still has four identical cells. Only cell 0 is shown.
pub const IDLE_PLAYED_FRAMES: usize = 1;
pub const IDLE_FRAME_ORIGIN: usize = 0;
pub const IDLE_ANIMATES: bool = false;
pub const IDLE_FRAME_SECONDS: f32 = 0.12;

/// Cell 0 is the signed lean, held until the run cycle exists.
pub const RUN_PLAYED_FRAMES: usize = 1;
pub const RUN_FRAME_ORIGIN: usize = 0;
pub const RUN_ANIMATES: bool = false;
pub const RUN_FRAME_SECONDS: f32 = 0.08;

/// Fraction of a quarter-turn the body flops. The death sheet drops this.
pub const DEATH_TILT: f32 = 0.9;
pub const DEATH_CELL: usize = 0;

/// Horizontal speed that counts as a run or a turn. Below this the pose stays idle.
const MOVE_SPEED: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackPhase {
    Windup,
    Between,
    Thrust,
    Recover,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HunterPose {
    Idle,
    Run,
    Jump,
    Fall,
    Attack(AttackPhase),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HunterSheet {
    Idle,
    Run,
    Attack,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Playback {
    pub played_frames: usize,
    pub origin_cell: usize,
    pub animates: bool,
    pub frame_seconds: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct DeathPose {
    pub sheet: HunterSheet,
    pub cell: usize,
    pub tilt: f32,
    pub visual_center: Vec2,
    pub anchor: Anchor,
}

pub fn is_moving(speed_x: f32) -> bool {
    speed_x.abs() > MOVE_SPEED
}

pub fn locomotion_pose(grounded: bool, velocity_y: f32, velocity_x: f32) -> HunterPose {
    if !grounded && velocity_y > 0.0 {
        HunterPose::Jump
    } else if !grounded {
        HunterPose::Fall
    } else if is_moving(velocity_x) {
        HunterPose::Run
    } else {
        HunterPose::Idle
    }
}

pub fn sheet_for(pose: HunterPose) -> HunterSheet {
    match pose {
        HunterPose::Idle => HunterSheet::Idle,
        HunterPose::Run => HunterSheet::Run,
        // Jump and fall keep the idle sheet until those images exist.
        HunterPose::Jump => HunterSheet::Idle,
        HunterPose::Fall => HunterSheet::Idle,
        HunterPose::Attack(_) => HunterSheet::Attack,
    }
}

pub fn playback_for(pose: HunterPose) -> Playback {
    match pose {
        HunterPose::Run => Playback {
            played_frames: RUN_PLAYED_FRAMES,
            origin_cell: RUN_FRAME_ORIGIN,
            animates: RUN_ANIMATES,
            frame_seconds: RUN_FRAME_SECONDS,
        },
        HunterPose::Idle | HunterPose::Jump | HunterPose::Fall | HunterPose::Attack(_) => {
            Playback {
                played_frames: IDLE_PLAYED_FRAMES,
                origin_cell: IDLE_FRAME_ORIGIN,
                animates: IDLE_ANIMATES,
                frame_seconds: IDLE_FRAME_SECONDS,
            }
        }
    }
}

/// `hit_start` and `hit_end` are seconds on the swing. `progress` is elapsed / duration.
pub fn attack_phase(
    step_index: usize,
    progress: f32,
    hit_start: f32,
    hit_end: f32,
    duration: f32,
) -> AttackPhase {
    let start = (hit_start / duration).clamp(0.0, 1.0);
    let end = (hit_end / duration).clamp(start, 1.0);
    let mut phase = if progress < start * 0.5 {
        AttackPhase::Windup
    } else if progress < start {
        AttackPhase::Between
    } else if progress < end {
        AttackPhase::Thrust
    } else {
        AttackPhase::Recover
    };
    if step_index > 0 && phase == AttackPhase::Windup {
        phase = AttackPhase::Between;
    }
    phase
}

pub fn attack_cell(phase: AttackPhase) -> usize {
    match phase {
        AttackPhase::Windup => 0,
        // No in-between drawing yet, so the chamber cell holds.
        AttackPhase::Between => 0,
        AttackPhase::Thrust => 2,
        // No recover drawing yet, so the thrust cell holds through the end of the step.
        AttackPhase::Recover => 2,
    }
}

pub fn pose_anchor_local(pose: HunterPose) -> Vec2 {
    match pose {
        HunterPose::Idle => anchor_local(IDLE_PIN_X, IDLE_SOLE_GAP),
        // Airborne poses share the idle contact until their sheets exist.
        HunterPose::Jump => anchor_local(IDLE_PIN_X, IDLE_SOLE_GAP),
        HunterPose::Fall => anchor_local(IDLE_PIN_X, IDLE_SOLE_GAP),
        HunterPose::Run => anchor_local(RUN_PIN_X, RUN_SOLE_GAP),
        HunterPose::Attack(AttackPhase::Windup | AttackPhase::Between) => {
            anchor_local(CHAMBER_PIN_X, CHAMBER_SOLE_GAP)
        }
        HunterPose::Attack(AttackPhase::Thrust | AttackPhase::Recover) => {
            anchor_local(THRUST_PIN_X, THRUST_SOLE_GAP)
        }
    }
}

pub fn pose_anchor(pose: HunterPose) -> Anchor {
    let mut local = pose_anchor_local(pose);
    if matches!(
        pose,
        HunterPose::Attack(AttackPhase::Thrust | AttackPhase::Recover)
    ) {
        // Thrust x is the body center. Read it from the shared anchor so the
        // blade tip cannot drift from `hunter_blade_tip_reach`.
        local.x = hunter_body_anchor().as_vec().x * HUNTER_CELL_PX.x;
    }
    Anchor::Custom(local / HUNTER_CELL_PX)
}

/// Holds frame 0 when the clip does not animate. Otherwise advances within `played_frames`.
pub fn next_played_frame(
    frame: usize,
    elapsed_in_frame: f32,
    playback: Playback,
    dt: f32,
) -> (usize, f32) {
    if !playback.animates || playback.played_frames <= 1 || playback.frame_seconds <= 0.0 {
        return (0, 0.0);
    }
    let mut frame = frame % playback.played_frames;
    let mut elapsed = elapsed_in_frame + dt;
    while elapsed >= playback.frame_seconds {
        elapsed -= playback.frame_seconds;
        frame = (frame + 1) % playback.played_frames;
    }
    (frame, elapsed)
}

/// Upright body center after orbiting `(0, 79)` around the planted sole.
/// At tilt 0 and sink 0 this is `physics_center`.
pub fn death_visual_center(physics_center: Vec2, tilt: f32, sink: f32) -> Vec2 {
    let planted = physics_center + Vec2::new(0.0, IDLE_SOLE_WORLD_Y - sink);
    let upright = Vec3::new(0.0, -IDLE_SOLE_WORLD_Y, 0.0);
    planted + Quat::from_rotation_z(tilt).mul_vec3(upright).truncate()
}

/// `velocity_y` and `landed` select the death sheet once those cells exist.
/// The signed strip has no death drawing, so both arguments stay idle cell 0.
pub fn death_pose(
    physics_center: Vec2,
    _velocity_y: f32,
    _landed: bool,
    facing: f32,
    fall: f32,
) -> DeathPose {
    let facing = facing.signum().clamp(-1.0, 1.0);
    let fall = fall.clamp(0.0, 1.0);
    let sink = fall * TILE * 0.35;
    let tilt = -facing * fall * FRAC_PI_2 * DEATH_TILT;
    DeathPose {
        sheet: HunterSheet::Idle,
        cell: DEATH_CELL,
        tilt,
        visual_center: death_visual_center(physics_center, tilt, sink),
        anchor: pose_anchor(HunterPose::Idle),
    }
}

fn anchor_local(pin_x: f32, sole_gap: f32) -> Vec2 {
    let center_x = HUNTER_CELL_PX.x * 0.5;
    let sole_local_y = -HUNTER_CELL_PX.y * 0.5 + sole_gap;
    Vec2::new(pin_x - center_x, sole_local_y - IDLE_SOLE_WORLD_Y)
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use super::*;
    use crate::graphics::{hunter_blade_tip_reach, hunter_body_anchor, TILE};

    fn point_world(
        origin: Vec2,
        facing: f32,
        anchor_local: Vec2,
        cell_x: f32,
        local_y: f32,
    ) -> Vec2 {
        let point = Vec2::new(cell_x - HUNTER_CELL_PX.x * 0.5, local_y);
        let delta = point - anchor_local;
        let sign = if facing < 0.0 { -1.0 } else { 1.0 };
        origin + Vec2::new(sign * delta.x, delta.y)
    }

    #[test]
    fn pose_anchors_match_the_measured_contacts() {
        assert_eq!(pose_anchor_local(HunterPose::Idle), Vec2::new(-85.5, 0.0));
        assert_eq!(pose_anchor_local(HunterPose::Run), Vec2::new(-106.5, 0.0));
        assert_eq!(
            pose_anchor_local(HunterPose::Attack(AttackPhase::Windup)),
            Vec2::new(-102.5, -1.0)
        );
        assert_eq!(
            pose_anchor_local(HunterPose::Attack(AttackPhase::Between)),
            Vec2::new(-102.5, -1.0)
        );
        assert_eq!(
            pose_anchor_local(HunterPose::Attack(AttackPhase::Thrust)),
            Vec2::new(-90.0, -1.0)
        );
        assert_eq!(
            pose_anchor_local(HunterPose::Attack(AttackPhase::Recover)),
            Vec2::new(-90.0, -1.0)
        );
        assert_eq!(IDLE_SOLE_WORLD_Y, -79.0);

        let idle = pose_anchor_local(HunterPose::Idle);
        assert_eq!(
            pose_anchor(HunterPose::Idle),
            Anchor::Custom(idle / HUNTER_CELL_PX)
        );
    }

    #[test]
    fn locomotion_feet_stay_put_when_the_hunter_turns() {
        let origin = Vec2::ZERO;
        let sole_y = -HUNTER_CELL_PX.y * 0.5 + IDLE_SOLE_GAP;
        for facing in [1.0, -1.0] {
            let idle = point_world(
                origin,
                facing,
                pose_anchor_local(HunterPose::Idle),
                IDLE_PIN_X,
                sole_y,
            );
            let run = point_world(
                origin,
                facing,
                pose_anchor_local(HunterPose::Run),
                RUN_PIN_X,
                sole_y,
            );
            assert_eq!(idle.x, 0.0, "idle facing {facing}");
            assert_eq!(run.x, 0.0, "run facing {facing}");
            assert_eq!(idle.y, IDLE_SOLE_WORLD_Y);
            assert_eq!(run.y, IDLE_SOLE_WORLD_Y);
        }
    }

    #[test]
    fn thrust_keeps_the_blade_tip_on_the_reach_constant() {
        let origin = Vec2::new(40.0, 10.0);
        let local = pose_anchor(HunterPose::Attack(AttackPhase::Thrust)).as_vec() * HUNTER_CELL_PX;
        let right = point_world(origin, 1.0, local, HUNTER_CELL_PX.x, 0.0);
        let left = point_world(origin, -1.0, local, HUNTER_CELL_PX.x, 0.0);
        let reach = hunter_blade_tip_reach();
        assert_eq!(reach, 261.5);
        assert_eq!(right.x, origin.x + reach);
        assert_eq!(left.x, origin.x - reach);
        assert_eq!(
            pose_anchor(HunterPose::Attack(AttackPhase::Thrust))
                .as_vec()
                .x,
            hunter_body_anchor().as_vec().x
        );
    }

    #[test]
    fn sword_step_zero_changes_drawing_with_the_hit_window() {
        let phase = |progress| attack_phase(0, progress, 0.07, 0.16, 0.26);
        assert_eq!(phase(0.0), AttackPhase::Windup);
        assert_eq!(attack_cell(phase(0.0)), 0);
        assert_eq!(phase(0.20), AttackPhase::Between);
        assert_eq!(attack_cell(phase(0.20)), 0);
        assert_eq!(phase(0.40), AttackPhase::Thrust);
        assert_eq!(attack_cell(phase(0.40)), 2);
        assert_eq!(phase(0.80), AttackPhase::Recover);
        assert_eq!(attack_cell(phase(0.80)), 2);

        let chained = attack_phase(1, 0.0, 0.08, 0.18, 0.28);
        assert_eq!(chained, AttackPhase::Between);
        assert_eq!(attack_cell(chained), 0);
    }

    #[test]
    fn held_clips_stay_on_frame_zero_and_a_cycle_advances() {
        let held = Playback {
            played_frames: 4,
            origin_cell: 0,
            animates: false,
            frame_seconds: 0.08,
        };
        let (frame, elapsed) = next_played_frame(3, 0.4, held, 1.0);
        assert_eq!(frame, 0);
        assert_eq!(elapsed, 0.0);

        let cycle = Playback {
            played_frames: 8,
            origin_cell: 1,
            animates: true,
            frame_seconds: 0.25,
        };
        let (frame, elapsed) = next_played_frame(0, 0.0, cycle, 1.0);
        assert_eq!(frame, 4);
        assert_eq!(elapsed, 0.0);

        let run = playback_for(HunterPose::Run);
        assert!(!run.animates);
        assert_eq!(run.played_frames, 1);
        assert_eq!(run.origin_cell, 0);
        assert_eq!(run.frame_seconds, RUN_FRAME_SECONDS);
        let idle = playback_for(HunterPose::Idle);
        assert!(!idle.animates);
        assert_eq!(idle.played_frames, 1);
        assert_eq!(idle.origin_cell, 0);
    }

    #[test]
    fn airborne_vertical_speed_picks_jump_or_fall_on_the_idle_sheet() {
        let jump = locomotion_pose(false, 1.0, 0.0);
        let crest = locomotion_pose(false, 0.0, 40.0);
        let falling = locomotion_pose(false, -20.0, 0.0);
        assert_eq!(jump, HunterPose::Jump);
        assert_eq!(crest, HunterPose::Fall);
        assert_eq!(falling, HunterPose::Fall);
        assert_eq!(sheet_for(jump), HunterSheet::Idle);
        assert_eq!(sheet_for(crest), HunterSheet::Idle);
        assert_eq!(sheet_for(falling), HunterSheet::Idle);
        assert_eq!(pose_anchor_local(jump), pose_anchor_local(HunterPose::Idle));
        assert_eq!(
            pose_anchor_local(falling),
            pose_anchor_local(HunterPose::Idle)
        );
        assert_eq!(sheet_for(HunterPose::Run), HunterSheet::Run);
        assert_eq!(
            sheet_for(HunterPose::Attack(AttackPhase::Thrust)),
            HunterSheet::Attack
        );
    }

    #[test]
    fn death_tilt_orbits_the_body_around_the_planted_foot() {
        let center = Vec2::new(12.0, 80.0);
        assert_eq!(death_visual_center(center, 0.0, 0.0), center);

        let upright = death_pose(center, 40.0, false, 1.0, 0.0);
        assert_eq!(upright.visual_center, center);
        assert_eq!(upright.tilt, 0.0);
        assert_eq!(upright.cell, 0);
        assert_eq!(upright.sheet, HunterSheet::Idle);
        assert_eq!(upright.anchor, pose_anchor(HunterPose::Idle));

        let tilt = -DEATH_TILT * FRAC_PI_2;
        let visual = death_visual_center(center, tilt, 0.0);
        let planted = center + Vec2::new(0.0, -79.0);
        let sole = visual
            + Quat::from_rotation_z(tilt)
                .mul_vec3(Vec3::new(0.0, -79.0, 0.0))
                .truncate();
        assert!(
            sole.distance(planted) < 1e-3,
            "sole {sole:?} left the planted foot {planted:?}"
        );
        assert!(visual.distance(center) > 1.0);

        let flop = death_pose(center, 0.0, true, 1.0, 1.0);
        assert!((flop.tilt - tilt).abs() < 1e-5);
        let sink = TILE * 0.35;
        assert_eq!(
            flop.visual_center,
            death_visual_center(center, flop.tilt, sink)
        );
        let sunk = center + Vec2::new(0.0, -79.0 - sink);
        let sunk_sole = flop.visual_center
            + Quat::from_rotation_z(flop.tilt)
                .mul_vec3(Vec3::new(0.0, -79.0, 0.0))
                .truncate();
        assert!(sunk_sole.distance(sunk) < 1e-3);

        let left = death_pose(center, 0.0, true, -1.0, 1.0);
        assert!((left.tilt - (DEATH_TILT * FRAC_PI_2)).abs() < 1e-5);
    }
}
