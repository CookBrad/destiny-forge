//! Mine map: one small cavern layer. Tinted stock tiles only (no new art).

use bevy::prelude::*;

use crate::exploration::{
    build_map_border, spawn_grid_overlay, tile_checker_shade, tint_shade, GridOverlayStyle,
};
use crate::graphics::{world_transform, TILE};
use crate::overworld::layout::tile_center;
use crate::overworld::sprites::OverworldArt;

use super::ore::OreKind;

pub const MAP_TILES_W: u32 = 30;
pub const MAP_TILES_H: u32 = 20;

pub const WORLD_WIDTH: f32 = MAP_TILES_W as f32 * TILE;
pub const WORLD_HEIGHT: f32 = MAP_TILES_H as f32 * TILE;

/// Ladder back up to the homestead (bottom-left).
pub const EXIT_TILE: (u32, u32) = (3, 1);
/// Where the player appears on entering.
pub const ENTRY_TILE: (u32, u32) = (3, 3);

/// Stable node ids — persisted in `PlayerProfile::depleted_ore_nodes`. Append only.
pub const ORE_NODES: [(u16, OreKind, u32, u32); 13] = [
    (0, OreKind::Copper, 7, 4),
    (1, OreKind::Copper, 10, 7),
    (2, OreKind::Copper, 6, 11),
    (3, OreKind::Copper, 12, 14),
    (4, OreKind::Copper, 15, 5),
    (5, OreKind::Copper, 18, 10),
    (6, OreKind::Copper, 9, 16),
    (7, OreKind::Copper, 21, 15),
    (8, OreKind::Iron, 22, 6),
    (9, OreKind::Iron, 25, 9),
    (10, OreKind::Iron, 24, 14),
    (11, OreKind::Iron, 27, 4),
    (12, OreKind::Iron, 26, 17),
];

pub fn mine_solids() -> Vec<Rect> {
    let mut solids = Vec::new();
    build_map_border(&mut solids, MAP_TILES_W, MAP_TILES_H);
    solids
}

pub fn ore_tint(kind: OreKind) -> Color {
    match kind {
        OreKind::Copper => Color::srgb(0.82, 0.48, 0.24),
        OreKind::Iron => Color::srgb(0.62, 0.6, 0.66),
    }
}

#[derive(Component)]
pub struct MineEntity;

#[derive(Component)]
pub struct MineExit;

#[derive(Component)]
pub struct OreNode {
    pub id: u16,
    pub kind: OreKind,
    pub hits: u32,
}

pub fn spawn_mine(commands: &mut Commands, art: &OverworldArt, depleted: &[u16]) {
    let floor = Color::srgb(0.3, 0.27, 0.26);
    let wall = Color::srgb(0.2, 0.18, 0.2);
    for ty in 0..MAP_TILES_H {
        for tx in 0..MAP_TILES_W {
            let edge = tx == 0 || ty == 0 || tx + 1 == MAP_TILES_W || ty + 1 == MAP_TILES_H;
            let (image, tint, z) = if edge {
                (art.wall.clone(), wall, 1.0)
            } else {
                (art.path.clone(), floor, 0.0)
            };
            commands.spawn((
                Sprite {
                    image,
                    color: tint_shade(tint, tile_checker_shade(tx, ty)),
                    ..default()
                },
                world_transform(tile_center(tx, ty), z),
                MineEntity,
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
            line_color: Color::srgba(0.04, 0.03, 0.04, 0.72),
            z: 0.08,
        },
        |entity| {
            entity.insert(MineEntity);
        },
    );

    commands.spawn((
        Sprite {
            image: art.path.clone(),
            color: Color::srgb(0.66, 0.56, 0.36),
            ..default()
        },
        world_transform(tile_center(EXIT_TILE.0, EXIT_TILE.1), 1.8),
        MineExit,
        MineEntity,
    ));

    for (id, kind, tx, ty) in ORE_NODES {
        if depleted.contains(&id) {
            continue;
        }
        commands.spawn((
            Sprite {
                image: art.wall.clone(),
                color: ore_tint(kind),
                ..default()
            },
            world_transform(tile_center(tx, ty), 2.0),
            OreNode { id, kind, hits: 0 },
            MineEntity,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_ids_are_unique_and_inside_the_cavern() {
        let mut ids: Vec<u16> = ORE_NODES.iter().map(|n| n.0).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ORE_NODES.len());
        for (_, _, tx, ty) in ORE_NODES {
            assert!(tx > 0 && ty > 0 && tx + 1 < MAP_TILES_W && ty + 1 < MAP_TILES_H);
            assert_ne!((tx, ty), EXIT_TILE);
            assert_ne!((tx, ty), ENTRY_TILE);
        }
    }

    #[test]
    fn mine_has_both_ore_kinds() {
        assert!(ORE_NODES.iter().any(|n| n.1 == OreKind::Copper));
        assert!(ORE_NODES.iter().any(|n| n.1 == OreKind::Iron));
    }
}
