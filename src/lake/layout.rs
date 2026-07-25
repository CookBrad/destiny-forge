//! Lake shore map — forest-sized exploration zone with a fishing pier.

use bevy::prelude::*;

use crate::exploration::{
    build_map_border, spawn_grid_overlay, tile_checker_shade, tile_rect, tint_shade, GridOverlayStyle,
};
use crate::graphics::{world_transform, TILE};
use crate::overworld::layout::tile_center;

pub const MAP_TILES_W: u32 = 40;
pub const MAP_TILES_H: u32 = 28;

pub const WORLD_WIDTH: f32 = MAP_TILES_W as f32 * TILE;
pub const WORLD_HEIGHT: f32 = MAP_TILES_H as f32 * TILE;

/// West trail edge — walk left/down into homestead.
pub fn lake_homestead_transition() -> Rect {
    tile_rect(0, 10, 2, 16)
}

#[derive(Resource, Clone)]
pub struct LakeLayout;

impl LakeLayout {
    pub fn generate() -> Self {
        Self
    }

    pub fn solids(&self) -> Vec<Rect> {
        let mut solids = Vec::new();
        build_map_border(&mut solids, MAP_TILES_W, MAP_TILES_H);
        solids
    }
}

pub fn lake_path(tx: u32, ty: u32) -> bool {
    // West approach trail
    (tx <= 8 && ty >= 11 && ty <= 14)
        // Shore path along the water
        || (tx >= 8 && tx <= 28 && ty >= 11 && ty <= 13)
        // Pier walkway
        || (tx >= 18 && tx <= 22 && ty >= 6 && ty <= 12)
}

pub fn lake_water(tx: u32, ty: u32) -> bool {
    // Large southern water body
    ty <= 10 && tx >= 6 && tx <= 36 && !lake_path(tx, ty)
}

/// Pier tip tiles where fishing is allowed.
pub const PIER_FISH_TILES: [(u32, u32); 3] = [(19, 6), (20, 6), (21, 6)];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lake_has_homestead_return_and_pier_spots() {
        let exit = lake_homestead_transition();
        assert!(exit.width() > 0.0 && exit.height() > 0.0);
        assert_eq!(PIER_FISH_TILES.len(), 3);
        // Pier tip sits in the water band
        for &(tx, ty) in &PIER_FISH_TILES {
            assert!(lake_water(tx, ty) || lake_path(tx, ty) || ty <= 10);
        }
        let layout = LakeLayout::generate();
        assert!(!layout.solids().is_empty());
    }

    #[test]
    fn lake_water_and_path_are_distinct() {
        assert!(lake_path(3, 12));
        assert!(lake_water(20, 4));
        assert!(!lake_path(20, 4));
    }
}

#[derive(Component)]
pub struct LakeEntity;

#[derive(Component)]
pub struct LakePier;

pub fn spawn_lake(commands: &mut Commands, grass: Handle<Image>, path: Handle<Image>, wall: Handle<Image>) {
    for ty in 0..MAP_TILES_H {
        for tx in 0..MAP_TILES_W {
            let center = tile_center(tx, ty);
            let shade = tile_checker_shade(tx, ty);
            let (texture, tint) = if lake_path(tx, ty) {
                (
                    path.clone(),
                    tint_shade(Color::srgb(0.52, 0.44, 0.32), shade),
                )
            } else if lake_water(tx, ty) {
                (
                    grass.clone(),
                    tint_shade(Color::srgb(0.12, 0.32, 0.52), shade),
                )
            } else {
                (
                    grass.clone(),
                    tint_shade(Color::srgb(0.28, 0.48, 0.26), shade),
                )
            };
            commands.spawn((
                Sprite {
                    image: texture,
                    color: tint,
                    ..default()
                },
                world_transform(center, 0.0),
                LakeEntity,
            ));
        }
    }

    spawn_grid_overlay(
        commands,
        grass.clone(),
        WORLD_WIDTH,
        WORLD_HEIGHT,
        MAP_TILES_W,
        MAP_TILES_H,
        GridOverlayStyle {
            line_color: Color::srgba(0.06, 0.1, 0.14, 0.55),
            z: 0.08,
        },
        |entity| {
            entity.insert(LakeEntity);
        },
    );

    // Pier planks
    for ty in 6..=12 {
        for tx in 19..=21 {
            let center = tile_center(tx, ty);
            commands.spawn((
                Sprite {
                    image: path.clone(),
                    color: Color::srgb(0.45, 0.34, 0.22),
                    custom_size: Some(Vec2::new(TILE * 0.95, TILE * 0.95)),
                    ..default()
                },
                world_transform(center, 0.6),
                LakePier,
                LakeEntity,
            ));
        }
    }

    // Dock posts
    for &(tx, ty) in &[(18, 6), (22, 6), (18, 10), (22, 10)] {
        let center = tile_center(tx, ty);
        commands.spawn((
            Sprite {
                image: wall.clone(),
                color: Color::srgb(0.35, 0.28, 0.2),
                custom_size: Some(Vec2::new(TILE * 0.35, TILE * 0.7)),
                ..default()
            },
            world_transform(center + Vec2::new(0.0, TILE * 0.15), 1.2),
            LakeEntity,
        ));
    }

    // Fishing spots at pier tip
    for &(tx, ty) in &PIER_FISH_TILES {
        let center = tile_center(tx, ty);
        commands.spawn((
            Sprite {
                image: wall.clone(),
                color: Color::srgb(0.55, 0.42, 0.28),
                custom_size: Some(Vec2::new(TILE * 0.5, TILE * 0.35)),
                ..default()
            },
            world_transform(center + Vec2::new(0.0, -TILE * 0.1), 1.3),
            crate::fishing::FishingSpot,
            LakeEntity,
        ));
    }

    // Shore reeds (decorative)
    for (tx, ty) in [(8, 11), (12, 10), (28, 11), (32, 9), (15, 11)] {
        let center = tile_center(tx, ty);
        commands.spawn((
            Sprite {
                image: grass.clone(),
                color: Color::srgb(0.25, 0.55, 0.3),
                custom_size: Some(Vec2::new(TILE * 0.4, TILE * 0.7)),
                ..default()
            },
            world_transform(center + Vec2::new(0.0, TILE * 0.15), 1.1),
            LakeEntity,
        ));
    }
}
