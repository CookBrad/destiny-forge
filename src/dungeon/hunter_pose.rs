use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::graphics::{hunter_body_anchor, HUNTER_CELL_PX, TILE};

/// Signed idle sole, relative to the body origin. The cell is 160 px tall and that
/// sole sits one pixel above the cell bottom, so the contact is 79 px below the origin.
const IDLE_SOLE_WORLD_Y: f32 = -(HUNTER_CELL_PX.y * 0.5 - IDLE_SOLE_GAP);

const IDLE_PIN_X: f32 = 86.0;
const IDLE_SOLE_GAP: f32 = 1.0;
/// New grounded frames plant here. It is the body-center x, sole one pixel up.
const FOOT_PIN_X: f32 = 81.5;
const FOOT_SOLE_GAP: f32 = 1.0;
const CHAMBER_PIN_X: f32 = 69.0;
const CHAMBER_SOLE_GAP: f32 = 0.0;
const THRUST_PIN_X: f32 = 81.5;
const THRUST_SOLE_GAP: f32 = 0.0;

/// Sheet file still has four identical cells. Only cell 0 is shown.
pub const IDLE_PLAYED_FRAMES: usize = 1;
pub const IDLE_FRAME_ORIGIN: usize = 0;
pub const IDLE_ANIMATES: bool = false;
pub const IDLE_FRAME_SECONDS: f32 = 0.12;

/// Cell 0 is the signed lean and is not played. Cells 1–8 are the cycle.
pub const RUN_PLAYED_FRAMES: usize = 8;
pub const RUN_FRAME_ORIGIN: usize = 1;
pub const RUN_ANIMATES: bool = true;
pub const RUN_FRAME_SECONDS: f32 = 0.08;

/// Quarter-turn fraction while the body is still airborne (~25°).
/// Landing shows the crumple drawing, so the tilt goes to zero.
pub const DEATH_AIR_TILT: f32 = 0.28;

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
    Jump,
    Fall,
    Death,
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
        HunterPose::Jump => HunterSheet::Jump,
        HunterPose::Fall => HunterSheet::Fall,
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
        AttackPhase::Between => 1,
        AttackPhase::Thrust => 2,
        AttackPhase::Recover => 3,
    }
}

pub fn pose_anchor_local(pose: HunterPose) -> Vec2 {
    match pose {
        HunterPose::Idle => anchor_local(IDLE_PIN_X, IDLE_SOLE_GAP),
        // Tucked feet are not the pin. The body center keeps the torso on the hurtbox.
        HunterPose::Jump => anchor_local(FOOT_PIN_X, FOOT_SOLE_GAP),
        // The down boot is drawn on the shared foot point.
        HunterPose::Fall => anchor_local(FOOT_PIN_X, FOOT_SOLE_GAP),
        HunterPose::Run => anchor_local(FOOT_PIN_X, FOOT_SOLE_GAP),
        HunterPose::Attack(AttackPhase::Windup) => {
            anchor_local(CHAMBER_PIN_X, CHAMBER_SOLE_GAP)
        }
        // Between and recover are drawn on the thrust pin so the tip stays put.
        HunterPose::Attack(
            AttackPhase::Between | AttackPhase::Thrust | AttackPhase::Recover,
        ) => anchor_local(THRUST_PIN_X, THRUST_SOLE_GAP),
    }
}

pub fn pose_anchor(pose: HunterPose) -> Anchor {
    let mut local = pose_anchor_local(pose);
    if matches!(
        pose,
        HunterPose::Attack(
            AttackPhase::Between | AttackPhase::Thrust | AttackPhase::Recover
        )
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

/// Airborne: stagger cell and a small tilt. Landed with `velocity_y == 0`: crumple, no tilt.
pub fn death_pose(
    physics_center: Vec2,
    velocity_y: f32,
    landed: bool,
    facing: f32,
    fall: f32,
) -> DeathPose {
    let facing = facing.signum().clamp(-1.0, 1.0);
    let fall = fall.clamp(0.0, 1.0);
    let sink = fall * TILE * 0.35;
    let on_ground = landed && velocity_y == 0.0;
    let (cell, tilt_frac) = if on_ground {
        (1, 0.0)
    } else {
        (0, DEATH_AIR_TILT)
    };
    let tilt = -facing * fall * FRAC_PI_2 * tilt_frac;
    DeathPose {
        sheet: HunterSheet::Death,
        cell,
        tilt,
        visual_center: death_visual_center(physics_center, tilt, sink),
        anchor: hunter_body_anchor(),
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
        assert_eq!(pose_anchor_local(HunterPose::Run), Vec2::new(-90.0, 0.0));
        assert_eq!(pose_anchor_local(HunterPose::Jump), Vec2::new(-90.0, 0.0));
        assert_eq!(pose_anchor_local(HunterPose::Fall), Vec2::new(-90.0, 0.0));
        assert_eq!(
            pose_anchor_local(HunterPose::Attack(AttackPhase::Windup)),
            Vec2::new(-102.5, -1.0)
        );
        assert_eq!(
            pose_anchor_local(HunterPose::Attack(AttackPhase::Between)),
            Vec2::new(-90.0, -1.0)
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
                FOOT_PIN_X,
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
        assert_eq!(attack_cell(phase(0.20)), 1);
        assert_eq!(phase(0.40), AttackPhase::Thrust);
        assert_eq!(attack_cell(phase(0.40)), 2);
        assert_eq!(phase(0.80), AttackPhase::Recover);
        assert_eq!(attack_cell(phase(0.80)), 3);

        let chained = attack_phase(1, 0.0, 0.08, 0.18, 0.28);
        assert_eq!(chained, AttackPhase::Between);
        assert_eq!(attack_cell(chained), 1);
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
        assert!(run.animates);
        assert_eq!(run.played_frames, 8);
        assert_eq!(run.origin_cell, 1);
        assert_eq!(run.frame_seconds, RUN_FRAME_SECONDS);
        let idle = playback_for(HunterPose::Idle);
        assert!(!idle.animates);
        assert_eq!(idle.played_frames, 1);
        assert_eq!(idle.origin_cell, 0);
    }

    #[test]
    fn airborne_vertical_speed_picks_the_jump_or_fall_sheet() {
        let jump = locomotion_pose(false, 1.0, 0.0);
        let crest = locomotion_pose(false, 0.0, 40.0);
        let falling = locomotion_pose(false, -20.0, 0.0);
        assert_eq!(jump, HunterPose::Jump);
        assert_eq!(crest, HunterPose::Fall);
        assert_eq!(falling, HunterPose::Fall);
        assert_eq!(sheet_for(jump), HunterSheet::Jump);
        assert_eq!(sheet_for(crest), HunterSheet::Fall);
        assert_eq!(sheet_for(falling), HunterSheet::Fall);
        assert_eq!(pose_anchor_local(jump), Vec2::new(-90.0, 0.0));
        assert_eq!(pose_anchor_local(falling), Vec2::new(-90.0, 0.0));
        assert_ne!(
            pose_anchor_local(jump),
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
        assert_eq!(upright.sheet, HunterSheet::Death);
        assert_eq!(upright.anchor, hunter_body_anchor());

        let tilt = -DEATH_AIR_TILT * FRAC_PI_2;
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

        let air = death_pose(center, -20.0, false, 1.0, 1.0);
        assert_eq!(air.cell, 0);
        assert_eq!(air.sheet, HunterSheet::Death);
        assert!((air.tilt - tilt).abs() < 1e-5);
        let sink = TILE * 0.35;
        let air_sole = air.visual_center
            + Quat::from_rotation_z(air.tilt)
                .mul_vec3(Vec3::new(0.0, -79.0, 0.0))
                .truncate();
        assert!(
            air_sole.distance(planted + Vec2::new(0.0, -sink)) < 1e-3,
            "airborne sole {air_sole:?} left the sunk foot"
        );

        let crumple = death_pose(center, 0.0, true, 1.0, 1.0);
        assert_eq!(crumple.cell, 1);
        assert_eq!(crumple.tilt, 0.0);
        assert_eq!(crumple.sheet, HunterSheet::Death);
        assert_eq!(crumple.anchor, hunter_body_anchor());
        let sink = TILE * 0.35;
        assert_eq!(
            crumple.visual_center,
            death_visual_center(center, 0.0, sink)
        );
        assert_eq!(
            crumple.visual_center,
            center + Vec2::new(0.0, -sink)
        );

        let left = death_pose(center, -5.0, false, -1.0, 1.0);
        assert!((left.tilt - (DEATH_AIR_TILT * FRAC_PI_2)).abs() < 1e-5);
    }

    const IDLE_SHA256: &str = "cc45126200af2c2298ad6ecfac517556939ae4598d81b3f9f58adabf7eb1a37e";

    #[test]
    fn hunter_sheets_match_the_cell_table_and_idle_stays_signed() {
        let root = env!("CARGO_MANIFEST_DIR");
        let sheets = [
            ("knight_idle_side.png", 1372_u32, 160_u32),
            ("knight_run_side.png", 3087, 160),
            ("knight_attack_side.png", 1372, 160),
            ("knight_jump_side.png", 343, 160),
            ("knight_fall_side.png", 343, 160),
            ("knight_death_side.png", 686, 160),
        ];
        let abc = sha256(b"abc");
        let abc_text = abc.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        assert_eq!(
            abc_text,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        for (name, width, height) in sheets {
            let path = format!("{root}/assets/player/combat/{name}");
            let bytes = std::fs::read(&path).unwrap_or_else(|err| panic!("read {path}: {err}"));
            let (got_w, got_h) = png_ihdr_size(&bytes, &path);
            assert_eq!((got_w, got_h), (width, height), "{name}");
            if name == "knight_idle_side.png" {
                let digest = sha256(&bytes);
                let text = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                assert_eq!(text, IDLE_SHA256);
            }
        }
    }

    fn png_ihdr_size(bytes: &[u8], path: &str) -> (u32, u32) {
        assert!(bytes.len() >= 24, "{path} is too small to be a PNG");
        assert_eq!(
            &bytes[..8],
            &[137, 80, 78, 71, 13, 10, 26, 10],
            "{path} is not a PNG"
        );
        assert_eq!(&bytes[12..16], b"IHDR", "{path} is missing IHDR");
        let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        (width, height)
    }

    fn sha256(message: &[u8]) -> [u8; 32] {
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];
        let mut hash: [u32; 8] = [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
            0x5be0cd19,
        ];
        let bit_len = (message.len() as u64).saturating_mul(8);
        let mut data = message.to_vec();
        data.push(0x80);
        while data.len() % 64 != 56 {
            data.push(0);
        }
        data.extend_from_slice(&bit_len.to_be_bytes());
        for chunk in data.chunks(64) {
            let mut word = [0u32; 64];
            for index in 0..16 {
                let start = index * 4;
                word[index] = u32::from_be_bytes(chunk[start..start + 4].try_into().unwrap());
            }
            for index in 16..64 {
                let s0 = word[index - 15].rotate_right(7)
                    ^ word[index - 15].rotate_right(18)
                    ^ (word[index - 15] >> 3);
                let s1 = word[index - 2].rotate_right(17)
                    ^ word[index - 2].rotate_right(19)
                    ^ (word[index - 2] >> 10);
                word[index] = word[index - 16]
                    .wrapping_add(s0)
                    .wrapping_add(word[index - 7])
                    .wrapping_add(s1);
            }
            let mut a = hash[0];
            let mut b = hash[1];
            let mut c = hash[2];
            let mut d = hash[3];
            let mut e = hash[4];
            let mut f = hash[5];
            let mut g = hash[6];
            let mut h = hash[7];
            for index in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let choose = (e & f) ^ (!e & g);
                let temp1 = h
                    .wrapping_add(s1)
                    .wrapping_add(choose)
                    .wrapping_add(K[index])
                    .wrapping_add(word[index]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let majority = (a & b) ^ (a & c) ^ (b & c);
                let temp2 = s0.wrapping_add(majority);
                h = g;
                g = f;
                f = e;
                e = d.wrapping_add(temp1);
                d = c;
                c = b;
                b = a;
                a = temp1.wrapping_add(temp2);
            }
            hash[0] = hash[0].wrapping_add(a);
            hash[1] = hash[1].wrapping_add(b);
            hash[2] = hash[2].wrapping_add(c);
            hash[3] = hash[3].wrapping_add(d);
            hash[4] = hash[4].wrapping_add(e);
            hash[5] = hash[5].wrapping_add(f);
            hash[6] = hash[6].wrapping_add(g);
            hash[7] = hash[7].wrapping_add(h);
        }
        let mut out = [0u8; 32];
        for (index, value) in hash.iter().enumerate() {
            out[index * 4..index * 4 + 4].copy_from_slice(&value.to_be_bytes());
        }
        out
    }
}
