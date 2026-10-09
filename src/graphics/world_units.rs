use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::Anchor;

pub const WORLD_UNITS_PER_SOURCE_PX: f32 = 1.0;
pub const CAMERA_ORTHO_SCALE: f32 = 1.0;

pub const TILE: f32 = 32.0;

pub const PLAYER_WALK_SPEED: f32 = 138.0;
pub const DUNGEON_JUMP_SPEED: f32 = 385.0;
pub const DUNGEON_AIR_JUMP_MULT: f32 = 0.88;
pub const DUNGEON_GRAVITY: f32 = -760.0;
pub const DUNGEON_FLOOR_Y: f32 = 64.0;
/// 1.5 tiles. 20 px was short of an 80 px forge station on the new module.
pub const INTERACT_DISTANCE: f32 = 48.0;

/// Gameplay body for every non-boss enemy: one tile. Sheets are larger.
/// The hitbox follows this size. Sprites are anchored so the painted sole
/// sits on this box's floor edge. Not a texture size.
pub const ENEMY_DISPLAY_SIZE: Vec2 = Vec2::new(TILE, TILE);

/// Uniform hunter sheet cell (hit frame width). The full cell still draws.
pub const HUNTER_CELL_PX: Vec2 = Vec2::new(343.0, 160.0);
/// GDD body inside that cell, left-aligned. Hurtbox and feet use this, not the cell.
pub const HUNTER_BODY_PX: Vec2 = Vec2::new(163.0, 160.0);

/// King slime's gameplay body is two tiles. The sheet is [`KING_SLIME_CANVAS_PX`].
/// This is not a texture scale; sprite transforms stay at magnitude 1.
pub const KING_SLIME_GAMEPLAY_SCALE: f32 = 2.0;

/// Locked king canvas: 6 tiles, taller than the ~160 px hunter.
pub const KING_SLIME_CANVAS_PX: Vec2 = Vec2::new(192.0, 192.0);

/// Distance from the body center to the hit-frame blade tip.
/// The cell is left-aligned on the body, so the tip is one cell width from the body's left edge.
pub const fn hunter_blade_tip_reach() -> f32 {
    HUNTER_CELL_PX.x - HUNTER_BODY_PX.x * 0.5
}

pub fn to_world(pixels: Vec2, z: f32) -> Vec3 {
    Vec3::new(pixels.x, pixels.y, z)
}

/// Place a sprite center so its feet sit on `surface_y`.
pub fn center_on_surface(surface_y: f32, sprite_height: f32) -> f32 {
    surface_y + sprite_height * 0.5
}

/// Authored sprites omit `custom_size` and use transform scale magnitude 1.
pub fn world_transform(pixels: Vec2, z: f32) -> Transform {
    Transform {
        translation: to_world(pixels, z),
        scale: Vec3::splat(WORLD_UNITS_PER_SOURCE_PX),
        ..default()
    }
}

/// 1×1 white image for solid fills (grid stroke, health bar, placeholder markers).
/// Not a gameplay sprite.
pub fn solid_white_pixel(images: &mut Assets<Image>) -> Handle<Image> {
    images.add(Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255, 255, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}

/// A solid fill of `size` logical pixels. The image stays 1×1; `custom_size` is not a resample.
pub fn solid_fill(pixel: Handle<Image>, size: Vec2, color: Color) -> Sprite {
    Sprite {
        image: pixel,
        color,
        custom_size: Some(size),
        ..default()
    }
}

/// Logical screen pixels for one axis. Test oracle for the 1× contract.
///
/// Sprites omit `custom_size` and use scale magnitude 1, so nothing in the game calls this.
/// A `custom_size` is a solid fill, not a resampled sprite.
#[cfg(test)]
fn logical_screen_px(
    source_px: f32,
    transform_scale: f32,
    camera_ortho_scale: f32,
    custom_size: Option<f32>,
) -> f32 {
    let drawn_px = custom_size.unwrap_or(source_px);
    drawn_px * transform_scale.abs() / camera_ortho_scale
}

/// Sign of `scale.x` mirrors children (the sword). Magnitude stays 1 because `flip_x` would not.
pub fn facing_scale(facing: f32) -> Vec3 {
    let sign = if facing < 0.0 { -1.0 } else { 1.0 };
    Vec3::new(
        sign * WORLD_UNITS_PER_SOURCE_PX,
        WORLD_UNITS_PER_SOURCE_PX,
        WORLD_UNITS_PER_SOURCE_PX,
    )
}

/// Pixel offset from cell center to body center: `(body_w - cell_w) / 2`.
pub fn hunter_body_anchor_offset_px() -> Vec2 {
    Vec2::new((HUNTER_BODY_PX.x - HUNTER_CELL_PX.x) * 0.5, 0.0)
}

/// `Anchor::Custom` is normalized to the cell, so the body center sits on the entity origin.
pub fn hunter_body_anchor() -> Anchor {
    let offset = hunter_body_anchor_offset_px();
    Anchor::Custom(Vec2::new(
        offset.x / HUNTER_CELL_PX.x,
        offset.y / HUNTER_CELL_PX.y,
    ))
}

/// Puts the canvas bottom `sole_below_origin` pixels under the entity.
/// The entity stays the hitbox center, so a taller sheet is not a scale.
pub fn sole_anchor(canvas_height: f32, sole_below_origin: f32) -> Anchor {
    Anchor::Custom(Vec2::new(0.0, sole_below_origin / canvas_height - 0.5))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bevy places the quad's min corner at `size * (-anchor - 0.5)`.
    fn quad_bottom(canvas_height: f32, anchor: Anchor) -> f32 {
        canvas_height * (-anchor.as_vec().y - 0.5)
    }

    #[test]
    fn tile_is_thirty_two_logical_px() {
        let px = logical_screen_px(TILE, WORLD_UNITS_PER_SOURCE_PX, CAMERA_ORTHO_SCALE, None);
        assert_eq!(px, 32.0);
        assert_eq!(TILE, 32.0);
        assert_eq!(WORLD_UNITS_PER_SOURCE_PX, 1.0);
        assert_eq!(CAMERA_ORTHO_SCALE, 1.0);
    }

    #[test]
    fn facing_scale_magnitude_is_one() {
        let left = facing_scale(-2.0);
        let right = facing_scale(1.0);
        assert_eq!(left.x.abs(), 1.0);
        assert_eq!(left.y, 1.0);
        assert_eq!(left.z, 1.0);
        assert!(left.x < 0.0);
        assert_eq!(right, Vec3::ONE);
    }

    #[test]
    fn feet_sit_on_surface_when_quad_height_matches() {
        let surface = DUNGEON_FLOOR_Y;
        let height = HUNTER_BODY_PX.y;
        let center = center_on_surface(surface, height);
        assert_eq!(center - height * 0.5, surface);
    }

    #[test]
    fn hunter_quad_is_cell_and_body_anchor_offset_is_minus_ninety() {
        assert_eq!(HUNTER_CELL_PX, Vec2::new(343.0, 160.0));
        assert_eq!(HUNTER_BODY_PX, Vec2::new(163.0, 160.0));
        let offset = hunter_body_anchor_offset_px();
        assert_eq!(offset.x, (163.0 - 343.0) / 2.0);
        assert_eq!(offset, Vec2::new(-90.0, 0.0));
    }

    #[test]
    fn hunter_and_enemy_feet_share_the_floor() {
        // Anchor y is 0 and the body is as tall as the cell, so the cell bottom is the feet.
        let hunter_center = center_on_surface(DUNGEON_FLOOR_Y, HUNTER_BODY_PX.y);
        let hunter_feet = hunter_center - HUNTER_CELL_PX.y * 0.5;
        let enemy_center = center_on_surface(DUNGEON_FLOOR_Y, ENEMY_DISPLAY_SIZE.y);
        let enemy_feet = enemy_center - ENEMY_DISPLAY_SIZE.y * 0.5;
        assert_eq!(hunter_feet, DUNGEON_FLOOR_Y);
        assert_eq!(enemy_feet, hunter_feet);
        assert_eq!(HUNTER_BODY_PX.y, HUNTER_CELL_PX.y);
    }

    #[test]
    fn taller_sheet_sole_stays_on_the_gameplay_floor() {
        let canvas_h = 64.0;
        let origin_above_floor = ENEMY_DISPLAY_SIZE.y * 0.5;
        let bottom = quad_bottom(canvas_h, sole_anchor(canvas_h, origin_above_floor));
        let center = center_on_surface(DUNGEON_FLOOR_Y, ENEMY_DISPLAY_SIZE.y);
        assert!((center + bottom - DUNGEON_FLOOR_Y).abs() < 0.01);
        let top = center + bottom + canvas_h;
        assert!((top - (DUNGEON_FLOOR_Y + canvas_h)).abs() < 0.01);
    }

    #[test]
    fn king_slime_sheet_tops_the_wall_and_the_body_stays_two_tiles() {
        assert_eq!(KING_SLIME_CANVAS_PX, Vec2::new(192.0, 192.0));
        let gameplay_h = ENEMY_DISPLAY_SIZE.y * KING_SLIME_GAMEPLAY_SCALE;
        assert_eq!(gameplay_h, 64.0);
        assert_eq!(KING_SLIME_GAMEPLAY_SCALE, 2.0);
        let bottom = quad_bottom(
            KING_SLIME_CANVAS_PX.y,
            sole_anchor(KING_SLIME_CANVAS_PX.y, gameplay_h * 0.5),
        );
        let center = center_on_surface(DUNGEON_FLOOR_Y, gameplay_h);
        assert!((center + bottom - DUNGEON_FLOOR_Y).abs() < 0.01);
        let top = center + bottom + KING_SLIME_CANVAS_PX.y;
        assert!((top - 256.0).abs() < 0.01);
        let logical = logical_screen_px(
            KING_SLIME_CANVAS_PX.y,
            WORLD_UNITS_PER_SOURCE_PX,
            CAMERA_ORTHO_SCALE,
            None,
        );
        assert_eq!(logical, 192.0);
    }

    #[test]
    fn jump_clears_three_tiles_and_the_air_jump_clears_four() {
        let gravity = DUNGEON_GRAVITY.abs();
        let apex = DUNGEON_JUMP_SPEED * DUNGEON_JUMP_SPEED / (2.0 * gravity);
        let air_speed = DUNGEON_JUMP_SPEED * DUNGEON_AIR_JUMP_MULT;
        let with_air = apex + air_speed * air_speed / (2.0 * gravity);
        assert!((apex - 97.5).abs() < 0.05, "apex {apex}");
        assert!(3.0 * TILE <= apex, "three tiles must clear on one jump");
        assert!(4.0 * TILE > apex, "four tiles need the air jump");
        assert!(
            4.0 * TILE <= with_air,
            "four tiles must clear with the air jump"
        );
    }

    #[test]
    fn interact_distance_is_a_tile_and_a_half() {
        assert_eq!(INTERACT_DISTANCE, 48.0);
        assert_eq!(INTERACT_DISTANCE, TILE * 1.5);
    }

    #[test]
    fn blade_tip_reach_is_the_hit_frame_past_the_body() {
        let tip = hunter_blade_tip_reach();
        assert_eq!(tip, HUNTER_CELL_PX.x - HUNTER_BODY_PX.x * 0.5);
        assert!(tip > HUNTER_BODY_PX.x * 0.5);
    }

    #[test]
    fn solid_fill_uses_custom_size_not_the_source_pixel() {
        let bar = logical_screen_px(1.0, 1.0, CAMERA_ORTHO_SCALE, Some(16.0));
        assert_eq!(bar, 16.0);
    }
}
