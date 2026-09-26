use bevy::prelude::*;

use crate::dungeon::{player_half_extents, SWORD_SPRITE_HEIGHT, SWORD_SPRITE_WIDTH};
use crate::graphics::hunter_blade_tip_reach;

#[derive(Clone, Copy, Debug)]
pub struct HitRect {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

/// Raised guard pose used while blocking with the sword.
pub const SWORD_GUARD_ANGLE: f32 = -0.55;

const SWORD_PIVOT_Y: f32 = -10.0;

/// How far a thrust's near edge sits back from the body edge, so the blade meets the hands.
const BLADE_ROOT_OVERLAP: f32 = 8.0;

/// Distance from the body center to where a thrust volume starts.
pub fn blade_root_from_center(body_half_width: f32) -> f32 {
    (body_half_width - BLADE_ROOT_OVERLAP).max(0.0)
}

pub fn sword_guard_aabb(player: &Transform) -> HitRect {
    sword_swing_aabb(player, SWORD_GUARD_ANGLE, hunter_blade_tip_reach())
}

pub fn sword_swing_aabb(player: &Transform, angle: f32, blade_length: f32) -> HitRect {
    let facing = animation_facing(player);
    let player_center = player.translation.truncate();
    let blade_local = blade_center_for_length(angle, blade_length);
    let blade_world = player_center + Vec2::new(facing * blade_local.x, blade_local.y);
    sword_sprite_aabb(blade_world, angle, blade_length)
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

/// Overlay pose for the transparent 12×30 placeholder. Damage uses [`sword_swing_aabb`].
pub fn sword_blade_center_local(angle: f32) -> Vec2 {
    blade_center_for_length(angle, SWORD_SPRITE_HEIGHT)
}

fn blade_center_for_length(angle: f32, blade_length: f32) -> Vec2 {
    let half_height = blade_length * 0.5;
    Vec2::new(
        half_height * (-angle).sin(),
        SWORD_PIVOT_Y + half_height * (-angle).cos(),
    )
}

pub fn sword_sprite_hit_rect(center: Vec2, angle: f32) -> HitRect {
    sword_sprite_aabb(center, angle, hunter_blade_tip_reach())
}

/// Axis-aligned blade. Horizontal reach equals `blade_length` (the painted hit-frame tip).
/// Vertical span covers the body so a thrust meets enemies standing at the hunter's feet.
fn sword_sprite_aabb(center: Vec2, angle: f32, blade_length: f32) -> HitRect {
    let half_w = SWORD_SPRITE_WIDTH * 0.5;
    let half_h = blade_length * 0.5;
    let c = angle.cos().abs();
    let s = angle.sin().abs();
    let extent_x = c * half_w + s * half_h;
    let extent_y = (s * half_w + c * half_h).max(player_half_extents().y);

    HitRect {
        min_x: center.x - extent_x,
        max_x: center.x + extent_x,
        min_y: center.y - extent_y,
        max_y: center.y + extent_y,
    }
}
