use bevy::prelude::*;

use std::f32::consts::FRAC_PI_2;

use crate::dungeon::{player_half_extents, SWORD_SPRITE_HEIGHT, SWORD_SPRITE_WIDTH};

#[derive(Clone, Copy, Debug)]
pub struct HitRect {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

/// Raised guard pose used while blocking with the sword.
pub const SWORD_GUARD_ANGLE: f32 = -0.55;

/// Fixed grip used for block guard (hip/side hold).
const SWORD_GUARD_GRIP: Vec2 = Vec2::new(14.0, -8.0);

/// Hand/hilt anchors in player-local pixels (+Y up), measured from
/// `assets/player/combat/knight_attack_side.png` hand clusters (f0 / f1 / f2).
/// Bezier control points for slash progress 0 → 0.5 → 1.
pub const SWORD_GRIP_WINDUP: Vec2 = Vec2::new(18.0, 12.0);
pub const SWORD_GRIP_MID: Vec2 = Vec2::new(22.5, 0.0);
pub const SWORD_GRIP_STRIKE: Vec2 = Vec2::new(25.0, 6.0);

/// Max distance (px) from measured hand anchor — used by tests and tuning.
pub const SWORD_GRIP_HAND_TOLERANCE: f32 = 10.0;

/// Vertical sword starts raised and sweeps 90° down/forward in local space.
/// Progress 0 → angle 0 (blade up); progress 1 → angle −π/2 (blade horizontal forward).
pub fn swing_angle(progress: f32) -> f32 {
    -progress.clamp(0.0, 1.0) * FRAC_PI_2
}

/// Hand/hilt position in player-local pixels at slash progress (quadratic Bezier).
pub fn sword_grip_local(progress: f32) -> Vec2 {
    let p = progress.clamp(0.0, 1.0);
    let a = SWORD_GRIP_WINDUP;
    let b = SWORD_GRIP_MID;
    let c = SWORD_GRIP_STRIKE;
    let u = 1.0 - p;
    a * (u * u) + b * (2.0 * u * p) + c * (p * p)
}

/// Offset from grip to sprite/blade center for a sword pointing along `angle`
/// (0 = up, −π/2 = forward). Matches grip-pivoted art (handle at bottom).
pub fn sword_blade_offset_from_grip(angle: f32) -> Vec2 {
    let half_height = SWORD_SPRITE_HEIGHT * 0.5;
    Vec2::new(
        half_height * (-angle).sin(),
        half_height * (-angle).cos(),
    )
}

/// Blade center in player-local space at slash progress (shared by visuals + hit volume).
pub fn sword_blade_center_at_progress(progress: f32) -> Vec2 {
    let angle = swing_angle(progress);
    sword_grip_local(progress) + sword_blade_offset_from_grip(angle)
}

/// Blade center for a fixed grip (block) or angle-only callers (specials).
pub fn sword_blade_center_local(angle: f32) -> Vec2 {
    // Guard uses a fixed side grip; specials that pass angle alone keep hip-ish pivot.
    let grip = if (angle - SWORD_GUARD_ANGLE).abs() < 0.05 {
        SWORD_GUARD_GRIP
    } else {
        // Legacy fixed pivot for charge/special helpers that only know angle.
        Vec2::new(0.0, -16.0)
    };
    grip + sword_blade_offset_from_grip(angle)
}

/// Full visual + hit pose for a sword slash at `progress` (0..=1).
/// Translation is the **grip** (hilt) in player-local space when using bottom-center anchor.
pub fn sword_swing_pose(progress: f32) -> (Vec2, f32) {
    let p = progress.clamp(0.0, 1.0);
    (sword_grip_local(p), swing_angle(p))
}

pub fn sword_guard_aabb(player: &Transform) -> HitRect {
    let facing = animation_facing(player);
    let player_center = player.translation.truncate();
    let blade_local = sword_blade_center_local(SWORD_GUARD_ANGLE);
    let blade_world = player_center + Vec2::new(facing * blade_local.x, blade_local.y);
    sword_sprite_aabb(blade_world, SWORD_GUARD_ANGLE)
}

/// Active slash volume at swing progress (same path as the weapon overlay).
pub fn sword_swing_aabb(player: &Transform, progress: f32) -> HitRect {
    let facing = animation_facing(player);
    let player_center = player.translation.truncate();
    let angle = swing_angle(progress);
    let blade_local = sword_blade_center_at_progress(progress);
    let blade_world = player_center + Vec2::new(facing * blade_local.x, blade_local.y);
    sword_sprite_aabb(blade_world, angle)
}

pub fn player_body_rect(player: &Transform) -> HitRect {
    let center = player.translation.truncate();
    let half = player_half_extents();
    HitRect {
        min_x: center.x - half.x,
        max_x: center.x + half.x,
        min_y: center.y - half.y,
        max_y: center.y + half.y,
    }
}

pub fn enemy_aabb(center: Vec2, half: Vec2) -> HitRect {
    HitRect {
        min_x: center.x - half.x,
        max_x: center.x + half.x,
        min_y: center.y - half.y,
        max_y: center.y + half.y,
    }
}

pub fn hitbox_overlaps(a: HitRect, b: HitRect) -> bool {
    a.min_x < b.max_x && a.max_x > b.min_x && a.min_y < b.max_y && a.max_y > b.min_y
}

pub fn expand_hit_rect(rect: HitRect, margin: f32) -> HitRect {
    HitRect {
        min_x: rect.min_x - margin,
        max_x: rect.max_x + margin,
        min_y: rect.min_y - margin,
        max_y: rect.max_y + margin,
    }
}

pub fn animation_facing(transform: &Transform) -> f32 {
    if transform.scale.x < 0.0 {
        -1.0
    } else {
        1.0
    }
}

pub fn sword_sprite_hit_rect(center: Vec2, angle: f32) -> HitRect {
    sword_sprite_aabb(center, angle)
}

fn sword_sprite_aabb(center: Vec2, angle: f32) -> HitRect {
    let half_w = SWORD_SPRITE_WIDTH * 0.5;
    let half_h = SWORD_SPRITE_HEIGHT * 0.5;
    let c = angle.cos().abs();
    let s = angle.sin().abs();
    let extent_x = c * half_w + s * half_h;
    let extent_y = s * half_w + c * half_h;

    HitRect {
        min_x: center.x - extent_x,
        max_x: center.x + extent_x,
        min_y: center.y - extent_y,
        max_y: center.y + extent_y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::{facing_scale, world_transform};

    #[test]
    fn sword_aabb_uses_native_sprite_extents() {
        let rect = sword_sprite_aabb(Vec2::ZERO, 0.0);
        assert!((rect.max_x - rect.min_x - SWORD_SPRITE_WIDTH).abs() < 0.01);
        assert!((rect.max_y - rect.min_y - SWORD_SPRITE_HEIGHT).abs() < 0.01);
    }

    #[test]
    fn swing_angle_is_raised_to_forward_quarter_turn() {
        assert!((swing_angle(0.0) - 0.0).abs() < 1e-5);
        assert!((swing_angle(1.0) + FRAC_PI_2).abs() < 1e-5);
        assert!((swing_angle(0.5) + FRAC_PI_2 * 0.5).abs() < 1e-5);
        // Monotone (more negative as progress increases).
        assert!(swing_angle(0.25) > swing_angle(0.75));
    }

    #[test]
    fn sword_grip_moves_toward_strike_along_arc() {
        let g0 = sword_grip_local(0.0);
        let g1 = sword_grip_local(0.5);
        let g2 = sword_grip_local(1.0);
        // Wind-up is higher than mid; strike is further forward than wind-up.
        assert!(g0.y > g1.y);
        assert!(g2.x > g0.x);
    }

    #[test]
    fn sword_hilt_tracks_documented_body_hand_anchors() {
        // Tight contract: grip at keyframes must match measured hand anchors
        // from knight_attack_side.png (not a loose body-wide band).
        let cases = [
            (0.0_f32, SWORD_GRIP_WINDUP),
            (0.5, SWORD_GRIP_MID),
            (1.0, SWORD_GRIP_STRIKE),
        ];
        for (p, anchor) in cases {
            let grip = sword_grip_local(p);
            let dist = (grip - anchor).length();
            assert!(
                dist <= SWORD_GRIP_HAND_TOLERANCE,
                "p={p}: grip {grip:?} is {dist:.1}px from hand anchor {anchor:?} (tol {})",
                SWORD_GRIP_HAND_TOLERANCE
            );
            // Blade center is offset from that same grip along the sword axis.
            let angle = swing_angle(p);
            let center = sword_blade_center_at_progress(p);
            assert!((center - grip - sword_blade_offset_from_grip(angle)).length() < 0.05);
        }
    }

    #[test]
    fn sword_grip_keyframes_match_constants() {
        // Bezier endpoints are exact; mid control is exact at p=0.5 only if
        // the curve is evaluated — endpoints must equal documented anchors.
        assert_eq!(sword_grip_local(0.0), SWORD_GRIP_WINDUP);
        assert_eq!(sword_grip_local(1.0), SWORD_GRIP_STRIKE);
        // Mid of quadratic is not necessarily the control point; ensure nearby.
        let mid = sword_grip_local(0.5);
        assert!((mid - SWORD_GRIP_MID).length() < SWORD_GRIP_HAND_TOLERANCE);
    }

    #[test]
    fn swing_pose_arc_is_monotone_and_continuous() {
        let mut prev_angle = swing_angle(0.0) + 1.0;
        let mut prev_grip_x = sword_grip_local(0.0).x - 1.0;
        for i in 0..=10 {
            let p = i as f32 / 10.0;
            let (grip, angle) = sword_swing_pose(p);
            assert!(angle <= prev_angle + 1e-4);
            assert!(grip.x >= prev_grip_x - 0.5);
            prev_angle = angle;
            prev_grip_x = grip.x;
        }
        let (g0, a0) = sword_swing_pose(0.0);
        let (g1, a1) = sword_swing_pose(1.0);
        assert!(g0.y > g1.y);
        assert!(a0 > a1);
    }

    #[test]
    fn swing_aabb_offsets_from_player_in_native_units() {
        let player = world_transform(Vec2::new(100.0, 50.0), 0.0);
        let mut facing_right = player;
        facing_right.scale = facing_scale(1.0);
        let rect = sword_swing_aabb(&facing_right, 0.0);
        let mid_x = (rect.min_x + rect.max_x) * 0.5;
        assert!((mid_x - 100.0).abs() < SWORD_SPRITE_HEIGHT + 40.0);
        assert!(rect.max_y > 50.0 - SWORD_SPRITE_HEIGHT);
    }
}
