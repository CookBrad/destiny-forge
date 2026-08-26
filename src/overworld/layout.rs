use bevy::prelude::*;

use crate::exploration::{
    build_map_border, spawn_grid_overlay, tile_checker_shade, tile_rect, tint_shade, zone_at,
    GridOverlayStyle, ZoneRect,
};
use crate::graphics::{center_on_surface, world_transform, TILE};

use super::sprites::{FORGE_ANVIL_HEIGHT, FORGE_FURNACE_HEIGHT, FORGE_WORKBENCH_HEIGHT};

use super::sprites::OverworldArt;

pub const MAP_TILES_W: u32 = 52;
pub const MAP_TILES_H: u32 = 40;

pub const WORLD_WIDTH: f32 = MAP_TILES_W as f32 * TILE;
pub const WORLD_HEIGHT: f32 = MAP_TILES_H as f32 * TILE;

/// Northern edge of the west forest trail — walk up into this to enter the forest.
pub fn homestead_forest_transition() -> Rect {
    tile_rect(2, 38, 5, 40)
}

pub fn homestead_forest_trail(tx: u32, ty: u32) -> bool {
    (tx >= 2 && tx <= 4 && ty >= 22 && ty <= 39)
        || (tx >= 4 && tx <= 6 && ty >= 25 && ty <= 28)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomesteadZone {
    House,
    Forge,
    Crops,
    Animals,
    ForestTrail,
    DungeonGate,
}

#[derive(Resource, Clone)]
pub struct OverworldLayout {
    pub solids: Vec<Rect>,
    pub zones: Vec<ZoneRect<HomesteadZone>>,
}

impl Default for OverworldLayout {
    fn default() -> Self {
        Self::homestead()
    }
}

impl OverworldLayout {
    pub fn homestead() -> Self {
        let mut solids = Vec::new();
        let mut zones = Vec::new();

        zones.push(ZoneRect {
            zone: HomesteadZone::House,
            bounds: tile_rect(2, 26, 15, 38),
        });
        zones.push(ZoneRect {
            zone: HomesteadZone::Forge,
            bounds: tile_rect(35, 26, 48, 38),
        });
        zones.push(ZoneRect {
            zone: HomesteadZone::Crops,
            bounds: tile_rect(2, 5, 21, 19),
        });
        zones.push(ZoneRect {
            zone: HomesteadZone::Animals,
            bounds: tile_rect(31, 5, 49, 19),
        });
        zones.push(ZoneRect {
            zone: HomesteadZone::ForestTrail,
            bounds: tile_rect(1, 22, 8, 40),
        });
        zones.push(ZoneRect {
            zone: HomesteadZone::DungeonGate,
            bounds: tile_rect(22, 1, 29, 5),
        });

        build_map_border(&mut solids, MAP_TILES_W, MAP_TILES_H);

        Self { solids, zones }
    }

    pub fn zone_at(&self, position: Vec2) -> Option<&ZoneRect<HomesteadZone>> {
        zone_at(&self.zones, position)
    }
}

pub fn spawn_homestead(commands: &mut Commands, art: &OverworldArt) {
    for ty in 0..MAP_TILES_H {
        for tx in 0..MAP_TILES_W {
            let center = tile_center(tx, ty);
            let (texture, tint) = ground_tile(tx, ty);
            let shade = tile_checker_shade(tx, ty);
            commands.spawn((
                Sprite {
                    image: texture(art),
                    color: tint_shade(tint, shade),
                    ..default()
                },
                world_transform(center, 0.0),
                OverworldTile,
                OverworldEntity,
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
            line_color: Color::srgba(0.08, 0.1, 0.06, 0.72),
            z: 0.08,
        },
        |entity| {
            entity.insert((OverworldGrid, OverworldEntity));
        },
    );

    spawn_house(commands, art, tile_rect(4, 30, 13, 38));
    spawn_forge(commands, art, tile_rect(37, 30, 46, 38));

    // Crop plots are spawned by FarmingPlugin / setup_overworld (persist + till/plant).
    spawn_animal_pen(commands, art, tile_rect(33, 7, 48, 17));
    spawn_dungeon_gate(commands, art, tile_rect(23, 2, 28, 4));
}

fn ground_tile(tx: u32, ty: u32) -> (fn(&OverworldArt) -> Handle<Image>, Color) {
    let on_path = (22..=29).contains(&tx) && ty <= 24
        || (tx >= 14 && tx <= 37 && (19..=24).contains(&ty))
        || (ty >= 25 && ty <= 28 && ((4..=13).contains(&tx) || (37..=46).contains(&tx)))
        || homestead_forest_trail(tx, ty);

    if on_path {
        (|art| art.path.clone(), Color::srgb(0.58, 0.48, 0.34))
    } else if tx >= 4 && tx <= 20 && ty >= 7 && ty <= 17 {
        (|art| art.soil.clone(), Color::srgb(0.36, 0.24, 0.14))
    } else {
        (|art| art.grass.clone(), Color::srgb(0.34, 0.52, 0.28))
    }
}
