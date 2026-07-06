use bevy::prelude::*;

use crate::graphics::{scaled_transform, PIXEL_SCALE, TILE};

use super::sprites::OverworldArt;

pub const MAP_TILES_W: u32 = 52;
pub const MAP_TILES_H: u32 = 40;

pub const WORLD_WIDTH: f32 = MAP_TILES_W as f32 * TILE;
pub const WORLD_HEIGHT: f32 = MAP_TILES_H as f32 * TILE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomesteadZone {
    Yard,
    House,
    Forge,
    Crops,
    Animals,
    DungeonGate,
}

#[derive(Resource, Clone)]
pub struct OverworldLayout {
    pub solids: Vec<Rect>,
    pub zones: Vec<ZoneRect>,
}

#[derive(Clone)]
pub struct ZoneRect {
    pub zone: HomesteadZone,
    pub bounds: Rect,
    pub label: &'static str,
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
            zone: HomesteadZone::DungeonGate,
            bounds: tile_rect(22, 1, 29, 5),
            label: "Dungeon Entrance",
        });

        build_house(&mut solids);
        build_forge(&mut solids);
        build_crop_field(&mut solids);
        build_animal_pen(&mut solids);
        build_map_border(&mut solids);

        Self { solids, zones }
    }

    pub fn zone_at(&self, position: Vec2) -> Option<&ZoneRect> {
        self.zones
            .iter()
            .rev()
            .find(|zone| zone.bounds.contains(position))
    }
}

fn tile_rect(x0: u32, y0: u32, x1: u32, y1: u32) -> Rect {
    Rect {
        min: Vec2::new(x0 as f32 * TILE, y0 as f32 * TILE),
        max: Vec2::new(x1 as f32 * TILE, y1 as f32 * TILE),
    }
}

fn build_map_border(solids: &mut Vec<Rect>) {
    let thickness = TILE;
    solids.push(tile_rect(0, 0, MAP_TILES_W, 1));
    solids.push(tile_rect(0, MAP_TILES_H - 1, MAP_TILES_W, MAP_TILES_H));
    solids.push(tile_rect(0, 0, 1, MAP_TILES_H));
    solids.push(tile_rect(MAP_TILES_W - 1, 0, MAP_TILES_W, MAP_TILES_H));
    let _ = thickness;
}

fn build_house(solids: &mut Vec<Rect>) {
    add_perimeter_with_door(solids, 4, 30, 13, 38, 7, 9);
}

fn build_forge(solids: &mut Vec<Rect>) {
    add_perimeter_with_door(solids, 37, 30, 46, 38, 40, 42);
}

fn build_crop_field(solids: &mut Vec<Rect>) {
    // South fence only — crop rows are visual, not collision.
    add_fence_line(solids, 3, 21, 18, 8, 12);
}

fn build_animal_pen(solids: &mut Vec<Rect>) {
    // South fence only — interior pen is open so the player can tend livestock.
    add_fence_line(solids, 32, 49, 18, 38, 42);
}

fn add_perimeter_with_door(
    solids: &mut Vec<Rect>,
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
    door_x0: u32,
    door_x1: u32,
) {
    for ty in y0..y1 {
        for tx in x0..x1 {
            let edge = tx == x0 || tx + 1 == x1 || ty + 1 == y1;
            let door = ty == y0 && tx >= door_x0 && tx < door_x1;
            if edge && !door {
                solids.push(tile_rect(tx, ty, tx + 1, ty + 1));
            }
        }
    }
}

fn add_fence_line(
    solids: &mut Vec<Rect>,
    x0: u32,
    x1: u32,
    y: u32,
    gap_x0: u32,
    gap_x1: u32,
) {
    for x in x0..x1 {
        if x >= gap_x0 && x < gap_x1 {
            continue;
        }
        solids.push(tile_rect(x, y, x + 1, y + 1));
    }
}

pub fn spawn_homestead(
    commands: &mut Commands,
    art: &OverworldArt,
    layout: &OverworldLayout,
) {
    for ty in 0..MAP_TILES_H {
        for tx in 0..MAP_TILES_W {
            let center = tile_center(tx, ty);
            let (texture, tint) = ground_tile(tx, ty);
            let shade = tile_checker_shade(tx, ty);
            commands.spawn((
                Sprite {
                    image: texture(art),
                    color: tint_shade(tint, shade),
                    custom_size: Some(Vec2::splat(TILE)),
                    ..default()
                },
                scaled_transform(center, 0.0),
                OverworldTile { tx, ty },
                OverworldEntity,
            ));
        }
    }

    spawn_grid_overlay(commands, art);

    spawn_building(
        commands,
        art,
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

    for zone in &layout.zones {
        if matches!(
            zone.zone,
            HomesteadZone::House | HomesteadZone::Forge | HomesteadZone::Crops | HomesteadZone::Animals
        ) {
            spawn_zone_marker(commands, art, &zone.bounds, zone.label);
        }
    }
}

fn tile_checker_shade(tx: u32, ty: u32) -> f32 {
    if (tx + ty) % 2 == 0 {
        1.0
    } else {
        0.9
    }
}

fn tint_shade(color: Color, shade: f32) -> Color {
    let c = color.to_srgba();
    Color::srgba(c.red * shade, c.green * shade, c.blue * shade, c.alpha)
}

fn spawn_grid_overlay(commands: &mut Commands, art: &OverworldArt) {
    let line = Color::srgba(0.08, 0.1, 0.06, 0.72);
    let z = 0.08;

    for tx in 0..=MAP_TILES_W {
        let x = tx as f32 * TILE;
        commands.spawn((
            Sprite {
                image: art.grid_line.clone(),
                color: line,
                custom_size: Some(Vec2::new(1.0, WORLD_HEIGHT)),
                ..default()
            },
            Transform {
                translation: Vec3::new(x, WORLD_HEIGHT * 0.5, z),
                scale: Vec3::splat(PIXEL_SCALE),
                ..default()
            },
            OverworldGrid,
            OverworldEntity,
        ));
    }

    for ty in 0..=MAP_TILES_H {
        let y = ty as f32 * TILE;
        commands.spawn((
            Sprite {
                image: art.grid_line.clone(),
                color: line,
                custom_size: Some(Vec2::new(WORLD_WIDTH, 1.0)),
                ..default()
            },
            Transform {
                translation: Vec3::new(WORLD_WIDTH * 0.5, y, z),
                scale: Vec3::splat(PIXEL_SCALE),
                ..default()
            },
            OverworldGrid,
            OverworldEntity,
        ));
    }
}

fn ground_tile(tx: u32, ty: u32) -> (fn(&OverworldArt) -> Handle<Image>, Color) {
    let on_path = (22..=29).contains(&tx) && ty <= 24
        || (tx >= 14 && tx <= 37 && (19..=24).contains(&ty))
        || (ty >= 25 && ty <= 28 && ((4..=13).contains(&tx) || (37..=46).contains(&tx)));

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
                    scaled_transform(center, 1.0),
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
                    scaled_transform(center, 0.5),
                    ForgeEntity,
                    OverworldEntity,
                ));
            }
        }
    }

    let back_y = footprint.max.y - TILE * 1.2;
    let center_x = (footprint.min.x + footprint.max.x) * 0.5;

    commands.spawn((
        Sprite {
            image: art.forge_furnace.clone(),
            color: Color::WHITE,
            custom_size: Some(Vec2::new(20.0, 26.0)),
            ..default()
        },
        Transform {
            translation: Vec2::new(center_x, back_y).extend(2.2),
            scale: Vec3::splat(PIXEL_SCALE),
            ..default()
        },
        ForgeEntity,
        OverworldEntity,
    ));

    commands.spawn((
        Sprite {
            image: art.forge_chimney.clone(),
            color: Color::WHITE,
            custom_size: Some(Vec2::new(14.0, 22.0)),
            ..default()
        },
        Transform {
            translation: Vec2::new(center_x + TILE * 2.2, back_y + TILE * 0.3).extend(2.3),
            scale: Vec3::splat(PIXEL_SCALE),
            ..default()
        },
        ForgeEntity,
        OverworldEntity,
    ));

    commands.spawn((
        Sprite {
            image: art.forge_workbench.clone(),
            color: Color::WHITE,
            custom_size: Some(Vec2::new(18.0, 28.0)),
            ..default()
        },
        Transform {
            translation: Vec2::new(footprint.min.x + TILE * 2.5, back_y - TILE * 1.5).extend(2.1),
            scale: Vec3::splat(PIXEL_SCALE),
            ..default()
        },
        ForgeEntity,
        OverworldEntity,
    ));

    commands.spawn((
        Sprite {
            image: art.forge_anvil.clone(),
            color: Color::WHITE,
            custom_size: Some(Vec2::new(14.0, 12.0)),
            ..default()
        },
        Transform {
            translation: Vec2::new(footprint.max.x - TILE * 2.0, back_y - TILE * 1.2).extend(2.1),
            scale: Vec3::splat(PIXEL_SCALE),
            ..default()
        },
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
        Transform {
            translation: roof_center.extend(2.6),
            scale: Vec3::splat(PIXEL_SCALE),
            ..default()
        },
        ForgeEntity,
        OverworldEntity,
    ));
}

fn spawn_building(
    commands: &mut Commands,
    art: &OverworldArt,
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
                    scaled_transform(center, 1.0),
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
        Transform {
            translation: roof_center.extend(roof_z),
            scale: Vec3::splat(PIXEL_SCALE),
            ..default()
        },
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
                    custom_size: Some(Vec2::new(TILE * 0.8, TILE * 0.55)),
                    ..default()
                },
                scaled_transform(center, 1.2),
                OverworldEntity,
            ));
        }
    }

    for tx in min_tx..max_tx {
        let center = tile_center(tx, min_ty - 1);
        commands.spawn((
            Sprite {
                image: art.fence.clone(),
                color: Color::srgb(0.55, 0.4, 0.24),
                ..default()
            },
            scaled_transform(center, 1.5),
            OverworldEntity,
        ));
    }
}

fn spawn_animal_pen(commands: &mut Commands, art: &OverworldArt, pen: Rect) {
    let min_tx = (pen.min.x / TILE).floor() as u32;
    let max_tx = (pen.max.x / TILE).ceil() as u32;
    let min_ty = (pen.min.y / TILE).floor() as u32;

    for tx in min_tx..max_tx {
        let center = tile_center(tx, min_ty - 1);
        commands.spawn((
            Sprite {
                image: art.fence.clone(),
                color: Color::srgb(0.62, 0.48, 0.3),
                ..default()
            },
            scaled_transform(center, 1.5),
            OverworldEntity,
        ));
    }

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
            tile_center(*tx, *ty),
            2.0,
            index,
            super::animals::AnimalWander::new(super::animals::WANDER_SPEED + index as f32 * 2.0),
        );
    }
}

fn spawn_dungeon_gate(commands: &mut Commands, art: &OverworldArt, gate: Rect) {
    let center = Vec2::new(
        (gate.min.x + gate.max.x) * 0.5,
        (gate.min.y + gate.max.y) * 0.5,
    );
    commands.spawn((
        Sprite {
            image: art.wall.clone(),
            color: Color::srgb(0.22, 0.18, 0.28),
            custom_size: Some(Vec2::new(TILE * 4.5, TILE * 2.5)),
            ..default()
        },
        scaled_transform(center, 1.8),
        DungeonEntrance,
        OverworldEntity,
    ));
}

fn spawn_zone_marker(commands: &mut Commands, art: &OverworldArt, bounds: &Rect, _label: &str) {
    let sign_x = bounds.min.x + TILE * 1.5;
    let sign_y = bounds.max.y - TILE * 0.5;
    commands.spawn((
        Sprite {
            image: art.fence.clone(),
            color: Color::srgb(0.72, 0.66, 0.42),
            custom_size: Some(Vec2::new(TILE * 0.6, TILE * 1.4)),
            ..default()
        },
        scaled_transform(Vec2::new(sign_x, sign_y), 3.0),
        OverworldEntity,
    ));
}

fn tile_center(tx: u32, ty: u32) -> Vec2 {
    Vec2::new(tx as f32 * TILE + TILE * 0.5, ty as f32 * TILE + TILE * 0.5)
}

#[derive(Component)]
pub struct OverworldEntity;

#[derive(Component)]
pub struct OverworldTile {
    pub tx: u32,
    pub ty: u32,
}

#[derive(Component)]
pub struct DungeonEntrance;

#[derive(Component)]
pub struct ForgeEntity;

#[derive(Component)]
pub struct OverworldGrid;