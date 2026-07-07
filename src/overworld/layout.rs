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
            label: "Your House",
        });
        zones.push(ZoneRect {
            zone: HomesteadZone::Forge,
            bounds: tile_rect(35, 26, 48, 38),
            label: "The Forge",
        });
        zones.push(ZoneRect {
            zone: HomesteadZone::Crops,
            bounds: tile_rect(2, 5, 21, 19),
            label: "Crop Fields",
        });
        zones.push(ZoneRect {
            zone: HomesteadZone::Animals,
            bounds: tile_rect(31, 5, 49, 19),
            label: "Animal Pens",
        });
        zones.push(ZoneRect {
            zone: HomesteadZone::ForestTrail,
            bounds: tile_rect(1, 22, 8, 40),
            label: "Forest Trail",
        });
        zones.push(ZoneRect {
            zone: HomesteadZone::DungeonGate,
            bounds: tile_rect(22, 1, 29, 5),
            label: "Dungeon Entrance",
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

    spawn_building(
        commands,
        tile_rect(4, 30, 13, 38),
        art.wall.clone(),
        Color::srgb(0.62, 0.5, 0.38),
        art.roof.clone(),
        Color::srgb(0.45, 0.22, 0.16),
        2.0,
    );
    spawn_forge(commands, art, tile_rect(37, 30, 46, 38));

    spawn_tilled_field(commands, art, tile_rect(4, 7, 20, 17));
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
    } else if tx >= 33 && tx <= 48 && ty >= 7 && ty <= 17 {
        (|art| art.grass.clone(), Color::srgb(0.42, 0.58, 0.3))
    } else {
        (|art| art.grass.clone(), Color::srgb(0.34, 0.52, 0.28))
    }
}

fn spawn_forge(commands: &mut Commands, art: &OverworldArt, footprint: Rect) {
    let min_tx = (footprint.min.x / TILE).floor() as u32;
    let max_tx = (footprint.max.x / TILE).ceil() as u32;
    let min_ty = (footprint.min.y / TILE).floor() as u32;
    let max_ty = (footprint.max.y / TILE).ceil() as u32;

    let stone = Color::srgb(0.46, 0.4, 0.34);
    let floor = Color::srgb(0.32, 0.28, 0.24);

    for ty in min_ty..max_ty {
        for tx in min_tx..max_tx {
            let center = tile_center(tx, ty);
            let edge = tx == min_tx || tx + 1 == max_tx || ty == min_ty || ty + 1 == max_ty;
            if edge {
                commands.spawn((
                    Sprite {
                        image: art.wall.clone(),
                        color: stone,
                        ..default()
                    },
                    world_transform(center, 1.0),
                    ForgeEntity,
                    OverworldEntity,
                ));
            } else {
                commands.spawn((
                    Sprite {
                        image: art.path.clone(),
                        color: floor,
                        ..default()
                    },
                    world_transform(center, 0.5),
                    ForgeEntity,
                    OverworldEntity,
                ));
            }
        }
    }

    let center_x = (footprint.min.x + footprint.max.x) * 0.5;
    let floor_surface = footprint.min.y + TILE;
    let back_wall = footprint.max.y - TILE;

    commands.spawn((
        Sprite {
            image: art.forge_furnace.clone(),
            color: Color::WHITE,
            ..default()
        },
        world_transform(
            Vec2::new(center_x, back_wall - FORGE_FURNACE_HEIGHT * 0.5),
            2.2,
        ),
        ForgeEntity,
        OverworldEntity,
    ));

    commands.spawn((
        Sprite {
            image: art.forge_workbench.clone(),
            color: Color::WHITE,
            ..default()
        },
        world_transform(
            Vec2::new(
                footprint.min.x + TILE * 3.0,
                center_on_surface(floor_surface, FORGE_WORKBENCH_HEIGHT),
            ),
            2.1,
        ),
        ForgeEntity,
        OverworldEntity,
    ));

    commands.spawn((
        Sprite {
            image: art.forge_anvil.clone(),
            color: Color::WHITE,
            ..default()
        },
        world_transform(
            Vec2::new(
                footprint.max.x - TILE * 3.0,
                center_on_surface(floor_surface, FORGE_ANVIL_HEIGHT),
            ),
            2.1,
        ),
        ForgeEntity,
        OverworldEntity,
    ));

    let roof_center = Vec2::new(
        center_x,
        footprint.max.y - TILE * 0.35,
    );
    commands.spawn((
        Sprite {
            image: art.roof.clone(),
            color: Color::srgb(0.34, 0.3, 0.28),
            custom_size: Some(Vec2::new(
                (max_tx - min_tx) as f32 * TILE * 0.85,
                TILE * 0.7,
            )),
            ..default()
        },
        world_transform(roof_center, 2.6),
        ForgeEntity,
        OverworldEntity,
    ));
}

fn spawn_building(
    commands: &mut Commands,
    footprint: Rect,
    wall_tex: Handle<Image>,
    wall_tint: Color,
    roof_tex: Handle<Image>,
    roof_tint: Color,
    roof_z: f32,
) {
    let min_tx = (footprint.min.x / TILE).floor() as u32;
    let max_tx = (footprint.max.x / TILE).ceil() as u32;
    let min_ty = (footprint.min.y / TILE).floor() as u32;
    let max_ty = (footprint.max.y / TILE).ceil() as u32;

    for ty in min_ty..max_ty {
        for tx in min_tx..max_tx {
            let center = tile_center(tx, ty);
            let edge = tx == min_tx || tx + 1 == max_tx || ty == min_ty || ty + 1 == max_ty;
            if edge {
                commands.spawn((
                    Sprite {
                        image: wall_tex.clone(),
                        color: wall_tint,
                        ..default()
                    },
                    world_transform(center, 1.0),
                    OverworldEntity,
                ));
            }
        }
    }

    let roof_center = Vec2::new(
        (footprint.min.x + footprint.max.x) * 0.5,
        footprint.max.y - TILE * 0.5,
    );
    commands.spawn((
        Sprite {
            image: roof_tex,
            color: roof_tint,
            custom_size: Some(Vec2::new(
                (max_tx - min_tx) as f32 * TILE,
                TILE * 1.2,
            )),
            ..default()
        },
        world_transform(roof_center, roof_z),
        OverworldEntity,
    ));
}

fn spawn_tilled_field(commands: &mut Commands, art: &OverworldArt, field: Rect) {
    let min_tx = (field.min.x / TILE).floor() as u32;
    let max_tx = (field.max.x / TILE).ceil() as u32;
    let min_ty = (field.min.y / TILE).floor() as u32;
    let max_ty = (field.max.y / TILE).ceil() as u32;

    for ty in min_ty..max_ty {
        for tx in min_tx..max_tx {
            if (tx + ty) % 3 != 0 {
                continue;
            }
            let center = tile_center(tx, ty);
            commands.spawn((
                Sprite {
                    image: art.soil.clone(),
                    color: Color::srgb(0.28, 0.62, 0.22),
                    ..default()
                },
                world_transform(center, 1.2),
                OverworldEntity,
            ));
        }
    }

}

fn spawn_animal_pen(commands: &mut Commands, art: &OverworldArt, _pen: Rect) {
    let animal_spots = [
        (36, 10),
        (40, 12),
        (44, 9),
        (38, 14),
        (42, 11),
    ];
    for (index, (tx, ty)) in animal_spots.iter().enumerate() {
        super::animals::spawn_farm_animal(
            commands,
            art.animal.clone(),
            art.animal_layout.clone(),
            tile_center(*tx, *ty),
            2.0,
            index,
            super::animals::AnimalWander::new(super::animals::WANDER_SPEED + index as f32 * 2.0),
        );
    }
}

fn spawn_dungeon_gate(commands: &mut Commands, art: &OverworldArt, gate: Rect) {
    let min_tx = (gate.min.x / TILE).floor() as u32;
    let max_tx = (gate.max.x / TILE).ceil() as u32;
    let min_ty = (gate.min.y / TILE).floor() as u32;
    let max_ty = (gate.max.y / TILE).ceil() as u32;
    let tint = Color::srgb(0.22, 0.18, 0.28);

    for ty in min_ty..max_ty {
        for tx in min_tx..max_tx {
            commands.spawn((
                Sprite {
                    image: art.wall.clone(),
                    color: tint,
                    ..default()
                },
                world_transform(tile_center(tx, ty), 1.8),
                DungeonEntrance,
                OverworldEntity,
            ));
        }
    }
}

pub fn tile_center(tx: u32, ty: u32) -> Vec2 {
    Vec2::new(tx as f32 * TILE + TILE * 0.5, ty as f32 * TILE + TILE * 0.5)
}

#[derive(Component)]
pub struct OverworldEntity;

#[derive(Component)]
pub struct OverworldTile;

#[derive(Component)]
pub struct DungeonEntrance;

#[derive(Component)]
pub struct ForgeEntity;

#[derive(Component)]
pub struct OverworldGrid;