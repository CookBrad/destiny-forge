use bevy::prelude::*;

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

const SWORD_PIVOT_Y: f32 = -40.0;

pub fn sword_guard_aabb(player: &Transform) -> HitRect {
    let facing = animation_facing(player);
    let player_center = player.translation.truncate();
    let blade_local = sword_blade_center_local(SWORD_GUARD_ANGLE);
    // World units match native art; display zoom is camera-only.
    let blade_world = player_center + Vec2::new(facing * blade_local.x, blade_local.y);
    sword_sprite_aabb(blade_world, SWORD_GUARD_ANGLE)
}

pub fn sword_swing_aabb(player: &Transform, angle: f32) -> HitRect {
    let facing = animation_facing(player);
    let player_center = player.translation.truncate();
    let blade_local = sword_blade_center_local(angle);
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

pub fn sword_blade_center_local(angle: f32) -> Vec2 {
    let half_height = SWORD_SPRITE_HEIGHT * 0.5;
    Vec2::new(
        half_height * (-angle).sin(),
        SWORD_PIVOT_Y + half_height * (-angle).cos(),
    )
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
        // Angle 0: AABB half-size matches native sword pixels (not a baked display scale).
        let rect = sword_sprite_aabb(Vec2::ZERO, 0.0);
        assert!((rect.max_x - rect.min_x - SWORD_SPRITE_WIDTH).abs() < 0.01);
        assert!((rect.max_y - rect.min_y - SWORD_SPRITE_HEIGHT).abs() < 0.01);
    }

    #[test]
    fn swing_aabb_offsets_from_player_in_native_units() {
        let player = world_transform(Vec2::new(100.0, 50.0), 0.0);
        let mut facing_right = player;
        facing_right.scale = facing_scale(1.0);
        let rect = sword_swing_aabb(&facing_right, 0.0);
        // Blade center is above pivot in local space; rect must sit near player X.
        let mid_x = (rect.min_x + rect.max_x) * 0.5;
        assert!((mid_x - 100.0).abs() < SWORD_SPRITE_HEIGHT);
        assert!(rect.max_y > 50.0 - SWORD_SPRITE_HEIGHT);
    }
}