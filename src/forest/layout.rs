use bevy::prelude::*;

use crate::exploration::{
    build_map_border, spawn_grid_overlay, tile_checker_shade, tile_rect, tint_shade,
    GridOverlayStyle,
};
use crate::graphics::{center_on_surface, world_transform, TILE};

use crate::overworld::layout::tile_center;

use super::sprites::{tree_frame_rect, ForestArt, TREE_CELL_H, TREE_TINT};

pub const MAP_TILES_W: u32 = 44;
pub const MAP_TILES_H: u32 = 32;

pub const WORLD_WIDTH: f32 = MAP_TILES_W as f32 * TILE;
pub const WORLD_HEIGHT: f32 = MAP_TILES_H as f32 * TILE;

/// Southern edge of the west trail — walk down into this to return to the homestead.
pub fn forest_homestead_transition() -> Rect {
    tile_rect(2, 0, 5, 2)
}

#[derive(Resource, Clone)]
pub struct ForestLayout;

impl ForestLayout {
    pub fn generate() -> Self {
        Self
    }

    pub fn solids(&self) -> Vec<Rect> {
        let mut solids = Vec::new();
        build_map_border(&mut solids, MAP_TILES_W, MAP_TILES_H);
        solids
    }

    pub fn tree_variant(&self, tx: u32, ty: u32) -> Option<usize> {
        if forest_path(tx, ty) || tx == 0 || ty == 0 || tx + 1 >= MAP_TILES_W || ty + 1 >= MAP_TILES_H
        {
            return None;
        }
        let hash = tx.wrapping_mul(37).wrapping_add(ty.wrapping_mul(91));
        if hash % 5 >= 2 {
            return None;
        }
        Some(hash as usize % super::sprites::TREE_VARIANTS)
    }
}

pub fn forest_path(tx: u32, ty: u32) -> bool {
    (tx >= 2 && tx <= 4 && ty <= 30)
        || (tx >= 2 && tx <= 6 && ty <= 4)
        || (tx >= 20 && tx <= 24 && ty <= 28)
}

pub fn spawn_forest(commands: &mut Commands, art: &ForestArt, layout: &ForestLayout) {
    for ty in 0..MAP_TILES_H {
        for tx in 0..MAP_TILES_W {
            let center = tile_center(tx, ty);
            let shade = tile_checker_shade(tx, ty);
            let (texture, tint) = if forest_path(tx, ty) {
                (
                    art.path.clone(),
                    tint_shade(Color::srgb(0.48, 0.4, 0.28), shade),
                )
            } else {
                (
                    art.grass.clone(),
                    tint_shade(Color::srgb(0.2, 0.38, 0.18), shade),
                )
            };
            commands.spawn((
                Sprite {
                    image: texture,
                    color: tint,
                    ..default()
                },
                world_transform(center, 0.0),
                ForestEntity,
            ));
        }
    }

    spawn_grid_overlay(
        commands,
        art.grid_line.clone(),
        WORLD_WIDTH,
        WORLD_HEIGHT,
        MAP_TILES_W,
        MAP_TILES_H,
        GridOverlayStyle {
            line_color: Color::srgba(0.05, 0.08, 0.04, 0.72),
            z: 0.08,
        },
        |entity| {
            entity.insert(ForestEntity);
        },
    );

    for ty in 1..MAP_TILES_H - 1 {
        for tx in 1..MAP_TILES_W - 1 {
            let Some(index) = layout.tree_variant(tx, ty) else {
                continue;
            };
            let center_x = tile_center(tx, ty).x;
            let ground_y = ty as f32 * TILE;
            commands.spawn((
                Sprite {
                    image: art.trees.clone(),
                    rect: Some(tree_frame_rect(index)),
                    color: TREE_TINT,
                    ..default()
                },
                world_transform(
                    Vec2::new(center_x, center_on_surface(ground_y, TREE_CELL_H)),
                    1.4,
                ),
                ForestTree,
                ForestEntity,
            ));
        }
    }

    let return_center = tile_center(3, 2);
    commands.spawn((
        Sprite {
            image: art.path.clone(),
            color: Color::srgb(0.62, 0.52, 0.34),
            ..default()
        },
        world_transform(return_center, 1.8),
        HomesteadReturnMarker,
        ForestEntity,
    ));
}

#[derive(Component)]
pub struct ForestEntity;

#[derive(Component)]
pub struct ForestTree;

#[derive(Component)]
pub struct HomesteadReturnMarker;