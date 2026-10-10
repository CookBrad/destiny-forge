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

pub const LADDER_TILE: (u32, u32) = (3, 1);
pub const ARRIVAL_TILE: (u32, u32) = (3, 3);

pub struct OreNodePlacement {
    pub id: u16,
    pub kind: OreKind,
    pub tile_x: u32,
    pub tile_y: u32,
}

pub const ORE_NODE_PLACEMENTS: [OreNodePlacement; 13] = [
    OreNodePlacement {
        id: 0,
        kind: OreKind::Copper,
        tile_x: 7,
        tile_y: 4,
    },
    OreNodePlacement {
        id: 1,
        kind: OreKind::Copper,
        tile_x: 10,
        tile_y: 7,
    },
    OreNodePlacement {
        id: 2,
        kind: OreKind::Copper,
        tile_x: 6,
        tile_y: 11,
    },
    OreNodePlacement {
        id: 3,
        kind: OreKind::Copper,
        tile_x: 12,
        tile_y: 14,
    },
    OreNodePlacement {
        id: 4,
        kind: OreKind::Copper,
        tile_x: 15,
        tile_y: 5,
    },
    OreNodePlacement {
        id: 5,
        kind: OreKind::Copper,
        tile_x: 18,
        tile_y: 10,
    },
    OreNodePlacement {
        id: 6,
        kind: OreKind::Copper,
        tile_x: 9,
        tile_y: 16,
    },
    OreNodePlacement {
        id: 7,
        kind: OreKind::Copper,
        tile_x: 21,
        tile_y: 15,
    },
    OreNodePlacement {
        id: 8,
        kind: OreKind::Iron,
        tile_x: 22,
        tile_y: 6,
    },
    OreNodePlacement {
        id: 9,
        kind: OreKind::Iron,
        tile_x: 25,
        tile_y: 9,
    },
    OreNodePlacement {
        id: 10,
        kind: OreKind::Iron,
        tile_x: 24,
        tile_y: 14,
    },
    OreNodePlacement {
        id: 11,
        kind: OreKind::Iron,
        tile_x: 27,
        tile_y: 4,
    },
    OreNodePlacement {
        id: 12,
        kind: OreKind::Iron,
        tile_x: 26,
        tile_y: 17,
    },
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
pub struct MineLadder;

#[derive(Component)]
pub struct OreNode {
    pub id: u16,
    pub kind: OreKind,
    pub hits_taken: u32,
}

pub fn spawn_mine(commands: &mut Commands, art: &OverworldArt, depleted_node_ids: &[u16]) {
    let floor_color = Color::srgb(0.3, 0.27, 0.26);
    let wall_color = Color::srgb(0.2, 0.18, 0.2);
    for ty in 0..MAP_TILES_H {
        for tx in 0..MAP_TILES_W {
            let is_border_tile =
                tx == 0 || ty == 0 || tx + 1 == MAP_TILES_W || ty + 1 == MAP_TILES_H;
            let (image, tint, z) = if is_border_tile {
                (art.wall.clone(), wall_color, 1.0)
            } else {
                (art.path.clone(), floor_color, 0.0)
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
        world_transform(tile_center(LADDER_TILE.0, LADDER_TILE.1), 1.8),
        MineLadder,
        MineEntity,
    ));

    for placement in &ORE_NODE_PLACEMENTS {
        if depleted_node_ids.contains(&placement.id) {
            continue;
        }
        commands.spawn((
            Sprite {
                image: art.wall.clone(),
                color: ore_tint(placement.kind),
                ..default()
            },
            world_transform(tile_center(placement.tile_x, placement.tile_y), 2.0),
            OreNode {
                id: placement.id,
                kind: placement.kind,
                hits_taken: 0,
            },
            MineEntity,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ore_node_ids_are_unique_and_nodes_sit_inside_the_cavern() {
        let mut ids: Vec<u16> = ORE_NODE_PLACEMENTS
            .iter()
            .map(|placement| placement.id)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ORE_NODE_PLACEMENTS.len());
        for placement in &ORE_NODE_PLACEMENTS {
            let tile = (placement.tile_x, placement.tile_y);
            assert!(
                tile.0 > 0 && tile.1 > 0 && tile.0 + 1 < MAP_TILES_W && tile.1 + 1 < MAP_TILES_H
            );
            assert_ne!(tile, LADDER_TILE);
            assert_ne!(tile, ARRIVAL_TILE);
        }
    }

    #[test]
    fn mine_has_both_ore_kinds() {
        assert!(ORE_NODE_PLACEMENTS
            .iter()
            .any(|placement| placement.kind == OreKind::Copper));
        assert!(ORE_NODE_PLACEMENTS
            .iter()
            .any(|placement| placement.kind == OreKind::Iron));
    }
}
