use bevy::prelude::*;

/// Gameplay positions use native sprite pixels as world units (one tile = [`TILE`]).
/// On-screen size is controlled by camera orthographic scale via [`DISPLAY_SCALE`],
/// not by baking multi-pixel scale into entity transforms.
///
/// Higher-res art: tiles are 64×64 (4× classic 16px pixel tiles).
pub const TILE: f32 = 64.0;

/// Higher-resolution 2D on-screen magnification.
/// Applied uniformly through the camera for exploration and dungeon.
/// With 64px native art, 1.0 keeps readable on-screen size without extra zoom.
pub const DISPLAY_SCALE: f32 = 1.0;

// Physics scaled with TILE so feel matches the classic 16px world (×4).
pub const PLAYER_WALK_SPEED: f32 = 552.0;
pub const DUNGEON_JUMP_SPEED: f32 = 1_540.0;
pub const DUNGEON_AIR_JUMP_MULT: f32 = 0.88;
pub const DUNGEON_GRAVITY: f32 = -3_040.0;
pub const DUNGEON_FLOOR_Y: f32 = 256.0;
pub const INTERACT_DISTANCE: f32 = 80.0;

pub const ENEMY_DISPLAY_SIZE: Vec2 = Vec2::new(TILE, TILE);

pub fn to_world(pixels: Vec2, z: f32) -> Vec3 {
    Vec3::new(pixels.x, pixels.y, z)
}

/// Place a sprite center so its feet sit on `surface_y`.
pub fn center_on_surface(surface_y: f32, sprite_height: f32) -> f32 {
    surface_y + sprite_height * 0.5
}

/// Orthographic projection scale for higher-res 2D: smaller scale zooms in.
/// Pure helper for tests and camera setup.
pub fn camera_ortho_scale(display_scale: f32) -> f32 {
    1.0 / display_scale.max(1.0)
}

/// Default game camera orthographic scale from [`DISPLAY_SCALE`].
pub fn game_camera_ortho_scale() -> f32 {
    camera_ortho_scale(DISPLAY_SCALE)
}

/// Horizontal/vertical flip for sprites without baking display scale into transform.
pub fn facing_scale(facing: f32) -> Vec3 {
    let sign = if facing < 0.0 { -1.0 } else { 1.0 };
    Vec3::new(sign, 1.0, 1.0)
}

/// Place a sprite in native world units. Display size comes from camera zoom.
pub fn world_transform(pixels: Vec2, z: f32) -> Transform {
    Transform {
        translation: to_world(pixels, z),
        ..default()
    }
}

/// Same as [`world_transform`] — unified higher-res 2D placement for all scenes.
pub fn scaled_transform(pixels: Vec2, z: f32) -> Transform {
    world_transform(pixels, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_ortho_scale_inverts_display_scale() {
        assert!((camera_ortho_scale(1.0) - 1.0).abs() < f32::EPSILON);
        assert!((camera_ortho_scale(3.0) - 1.0 / 3.0).abs() < f32::EPSILON);
        assert!((camera_ortho_scale(4.0) - 0.25).abs() < f32::EPSILON);
        // Sub-1.0 clamps to 1.0 so we never invert zoom direction unexpectedly.
        assert!((camera_ortho_scale(0.5) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn game_camera_uses_display_scale() {
        assert!((game_camera_ortho_scale() - camera_ortho_scale(DISPLAY_SCALE)).abs() < f32::EPSILON);
        assert!(DISPLAY_SCALE >= 1.0);
    }

    #[test]
    fn world_and_scaled_transform_match_and_stay_unscaled() {
        let pos = Vec2::new(48.0, 64.0);
        let a = world_transform(pos, 5.0);
        let b = scaled_transform(pos, 5.0);

        assert_eq!(a.translation, b.translation);
        assert_eq!(a.translation, Vec3::new(48.0, 64.0, 5.0));
        assert_eq!(a.scale, Vec3::ONE);
        assert_eq!(b.scale, Vec3::ONE);
    }

    #[test]
    fn facing_scale_flips_x_only() {
        assert_eq!(facing_scale(1.0), Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(facing_scale(-1.0), Vec3::new(-1.0, 1.0, 1.0));
        assert_eq!(facing_scale(0.0), Vec3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn center_on_surface_places_feet_on_ground() {
        let y = center_on_surface(256.0, 112.0);
        assert!((y - 312.0).abs() < f32::EPSILON);
    }

    #[test]
    fn to_world_preserves_xy_and_z() {
        assert_eq!(to_world(Vec2::new(10.0, 20.0), 3.0), Vec3::new(10.0, 20.0, 3.0));
    }

    #[test]
    fn tile_matches_higher_res_art_contract() {
        // Shipped env tiles are 64×64; world tile must match.
        assert!((TILE - 64.0).abs() < f32::EPSILON);
    }
}
