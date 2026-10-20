use bevy::prelude::*;

use crate::graphics::{world_transform, TILE};

#[derive(Clone)]
pub struct ZoneRect<Z> {
    pub zone: Z,
    pub bounds: Rect,
}

pub fn zone_at<Z>(zones: &[ZoneRect<Z>], position: Vec2) -> Option<&ZoneRect<Z>> {
    zones
        .iter()
        .rev()
        .find(|zone| zone.bounds.contains(position))
}

pub fn tile_rect(x0: u32, y0: u32, x1: u32, y1: u32) -> Rect {
    Rect {
        min: Vec2::new(x0 as f32 * TILE, y0 as f32 * TILE),
        max: Vec2::new(x1 as f32 * TILE, y1 as f32 * TILE),
    }
}

pub fn build_map_border(solids: &mut Vec<Rect>, map_tiles_w: u32, map_tiles_h: u32) {
    solids.push(tile_rect(0, 0, map_tiles_w, 1));
    solids.push(tile_rect(0, map_tiles_h - 1, map_tiles_w, map_tiles_h));
    solids.push(tile_rect(0, 0, 1, map_tiles_h));
    solids.push(tile_rect(map_tiles_w - 1, 0, map_tiles_w, map_tiles_h));
}

pub fn tile_checker_shade(tx: u32, ty: u32) -> f32 {
    if (tx + ty).is_multiple_of(2) {
        1.0
    } else {
        0.9
    }
}

pub fn tint_shade(color: Color, shade: f32) -> Color {
    let c = color.to_srgba();
    Color::srgba(c.red * shade, c.green * shade, c.blue * shade, c.alpha)
}

pub struct GridOverlayStyle {
    pub line_color: Color,
    pub z: f32,
}

pub struct GridOverlayExtent {
    pub world_width: f32,
    pub world_height: f32,
    pub map_tiles_w: u32,
    pub map_tiles_h: u32,
}

pub fn spawn_grid_overlay(
    commands: &mut Commands,
    grid_line: Handle<Image>,
    extent: GridOverlayExtent,
    style: GridOverlayStyle,
    mut decorate: impl FnMut(&mut EntityCommands),
) {
    let GridOverlayExtent {
        world_width,
        world_height,
        map_tiles_w,
        map_tiles_h,
    } = extent;
    for tx in 0..=map_tiles_w {
        let x = tx as f32 * TILE;
        let mut entity = commands.spawn((
            Sprite {
                image: grid_line.clone(),
                color: style.line_color,
                // Stroke length is a world-unit fill. The image is a 1×1 pixel, not tile art.
                custom_size: Some(Vec2::new(1.0, world_height)),
                ..default()
            },
            world_transform(Vec2::new(x, world_height * 0.5), style.z),
        ));
        decorate(&mut entity);
    }

    for ty in 0..=map_tiles_h {
        let y = ty as f32 * TILE;
        let mut entity = commands.spawn((
            Sprite {
                image: grid_line.clone(),
                color: style.line_color,
                // Stroke length is a world-unit fill. The image is a 1×1 pixel, not tile art.
                custom_size: Some(Vec2::new(world_width, 1.0)),
                ..default()
            },
            world_transform(Vec2::new(world_width * 0.5, y), style.z),
        ));
        decorate(&mut entity);
    }
}
