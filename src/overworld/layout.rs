use bevy::prelude::*;

use crate::exploration::{
    build_map_border, spawn_grid_overlay, tile_checker_shade, tile_rect, tint_shade, zone_at,
    GridOverlayExtent, GridOverlayStyle, ZoneRect,
};
use crate::graphics::{center_on_surface, world_transform, TILE};

use super::sprites::{FORGE_ANVIL_HEIGHT, FORGE_FURNACE_HEIGHT, FORGE_WORKBENCH_HEIGHT};

use super::sprites::OverworldArt;

pub const MAP_TILES_W: u32 = 52;
pub const MAP_TILES_H: u32 = 40;

pub const WORLD_WIDTH: f32 = MAP_TILES_W as f32 * TILE;
pub const WORLD_HEIGHT: f32 = MAP_TILES_H as f32 * TILE;

pub fn mine_entrance_rect() -> Rect {
    tile_rect(45, 21, 49, 24)
}

pub fn mine_return_tile() -> Vec2 {
    tile_center(46, 20)
}

pub fn fishing_dock_rect() -> Rect {
    tile_rect(1, 12, 5, 16)
}

pub fn northern_forest_trail_entry_rect() -> Rect {
    tile_rect(2, 38, 5, 40)
}

pub fn homestead_forest_trail(tx: u32, ty: u32) -> bool {
    ((2..=4).contains(&tx) && (22..=39).contains(&ty))
        || ((4..=6).contains(&tx) && (25..=28).contains(&ty))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomesteadZone {
    House,
    Forge,
    Crops,
    Animals,
    ForestTrail,
    DungeonGate,
    MineEntrance,
    FishingDock,
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
        let zones = vec![
            ZoneRect {
                zone: HomesteadZone::House,
                bounds: tile_rect(2, 26, 15, 38),
            },
            ZoneRect {
                zone: HomesteadZone::Forge,
                bounds: tile_rect(35, 26, 48, 38),
            },
            ZoneRect {
                zone: HomesteadZone::Crops,
                bounds: tile_rect(2, 5, 21, 19),
            },
            ZoneRect {
                zone: HomesteadZone::Animals,
                bounds: tile_rect(31, 5, 49, 19),
            },
            ZoneRect {
                zone: HomesteadZone::ForestTrail,
                bounds: tile_rect(1, 22, 8, 40),
            },
            ZoneRect {
                zone: HomesteadZone::DungeonGate,
                bounds: tile_rect(22, 1, 29, 5),
            },
            ZoneRect {
                zone: HomesteadZone::MineEntrance,
                bounds: mine_entrance_rect(),
            },
            ZoneRect {
                zone: HomesteadZone::FishingDock,
                bounds: fishing_dock_rect(),
            },
        ];

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
        GridOverlayExtent {
            world_width: WORLD_WIDTH,
            world_height: WORLD_HEIGHT,
            map_tiles_w: MAP_TILES_W,
            map_tiles_h: MAP_TILES_H,
        },
        GridOverlayStyle {
            line_color: Color::srgba(0.08, 0.1, 0.06, 0.72),
            z: 0.08,
        },
        |entity| {
            entity.insert((OverworldGrid, OverworldEntity));
        },
    );

    spawn_player_house(commands, art, tile_rect(4, 30, 13, 38));
    spawn_forge(commands, art, tile_rect(37, 30, 46, 38));

    spawn_animal_pen(commands, art, tile_rect(33, 7, 48, 17));
    spawn_dungeon_gate(commands, art, tile_rect(23, 2, 28, 4));
    spawn_mine_entrance(commands, art, mine_entrance_rect());
    spawn_fishing_dock(commands, art, fishing_dock_rect());
}

fn ground_tile(tx: u32, ty: u32) -> (fn(&OverworldArt) -> Handle<Image>, Color) {
    let on_path = (22..=29).contains(&tx) && ty <= 24
        || ((14..=37).contains(&tx) && (19..=24).contains(&ty))
        || ((38..=48).contains(&tx) && (20..=23).contains(&ty))
        || ((25..=28).contains(&ty) && ((4..=13).contains(&tx) || (37..=46).contains(&tx)))
        || homestead_forest_trail(tx, ty);

    if on_path {
        (|art| art.path.clone(), Color::srgb(0.58, 0.48, 0.34))
    } else if (4..=20).contains(&tx) && (7..=17).contains(&ty) {
        (|art| art.soil.clone(), Color::srgb(0.36, 0.24, 0.14))
    } else if (33..=48).contains(&tx) && (7..=17).contains(&ty) {
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

    spawn_native_roof_row(
        commands,
        NativeRoofRow {
            image: art.roof.clone(),
            color: Color::srgb(0.34, 0.3, 0.28),
            min_tx,
            max_tx,
            tile_y: max_ty.saturating_sub(1),
            z: 2.6,
        },
        |entity| {
            entity.insert((ForgeEntity, OverworldEntity));
        },
    );
}

fn spawn_player_house(commands: &mut Commands, art: &OverworldArt, footprint: Rect) {
    let min_tx = (footprint.min.x / TILE).floor() as u32;
    let max_tx = (footprint.max.x / TILE).ceil() as u32;
    let min_ty = (footprint.min.y / TILE).floor() as u32;
    let max_ty = (footprint.max.y / TILE).ceil() as u32;

    let wall_tint = Color::srgb(0.62, 0.5, 0.38);
    let floor_tint = Color::srgb(0.42, 0.32, 0.22);
    let door_tx_start = min_tx + (max_tx - min_tx) / 2 - 1;
    let door_tx_end = door_tx_start + 2;

    for ty in min_ty..max_ty {
        for tx in min_tx..max_tx {
            let center = tile_center(tx, ty);
            let on_south = ty == min_ty;
            let on_edge = tx == min_tx || tx + 1 == max_tx || on_south || ty + 1 == max_ty;
            let door_gap = on_south && tx >= door_tx_start && tx < door_tx_end;

            if on_edge && !door_gap {
                commands.spawn((
                    Sprite {
                        image: art.wall.clone(),
                        color: wall_tint,
                        ..default()
                    },
                    world_transform(center, 1.0),
                    HouseEntity,
                    OverworldEntity,
                ));
            } else if !on_edge || door_gap {
                commands.spawn((
                    Sprite {
                        image: art.path.clone(),
                        color: floor_tint,
                        ..default()
                    },
                    world_transform(center, 0.4),
                    HouseEntity,
                    OverworldEntity,
                ));
            }
        }
    }

    spawn_sleep_bed_placeholder(commands, art, min_tx, max_tx, min_ty, max_ty);

    spawn_native_roof_row(
        commands,
        NativeRoofRow {
            image: art.roof.clone(),
            color: Color::srgb(0.45, 0.22, 0.16),
            min_tx,
            max_tx,
            tile_y: max_ty.saturating_sub(1),
            z: 2.0,
        },
        |entity| {
            entity.insert((HouseEntity, OverworldEntity));
        },
    );
}

struct NativeRoofRow {
    image: Handle<Image>,
    color: Color,
    min_tx: u32,
    max_tx: u32,
    tile_y: u32,
    z: f32,
}

fn spawn_native_roof_row(
    commands: &mut Commands,
    row: NativeRoofRow,
    mut mark: impl FnMut(&mut EntityCommands),
) {
    let NativeRoofRow {
        image,
        color,
        min_tx,
        max_tx,
        tile_y,
        z,
    } = row;
    for tx in min_tx..max_tx {
        let mut roof = commands.spawn((
            Sprite {
                image: image.clone(),
                color,
                ..default()
            },
            world_transform(tile_center(tx, tile_y), z),
        ));
        mark(&mut roof);
    }
}

fn spawn_animal_pen(commands: &mut Commands, art: &OverworldArt, _pen: Rect) {
    let animal_spots = [(36, 10), (40, 12), (44, 9), (38, 14), (42, 11)];
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

fn tile_range_from_rect(bounds: Rect) -> (u32, u32, u32, u32) {
    let first_tile_column = (bounds.min.x / TILE).floor() as u32;
    let end_tile_column = (bounds.max.x / TILE).ceil() as u32;
    let first_tile_row = (bounds.min.y / TILE).floor() as u32;
    let end_tile_row = (bounds.max.y / TILE).ceil() as u32;
    (
        first_tile_column,
        end_tile_column,
        first_tile_row,
        end_tile_row,
    )
}

fn spawn_mine_entrance(commands: &mut Commands, art: &OverworldArt, entrance: Rect) {
    let (first_tile_column, end_tile_column, first_tile_row, end_tile_row) =
        tile_range_from_rect(entrance);
    let rock_color = Color::srgb(0.36, 0.32, 0.3);
    let opening_color = Color::srgb(0.08, 0.07, 0.08);

    for tile_row in first_tile_row..end_tile_row {
        for tile_column in first_tile_column..end_tile_column {
            let is_opening_tile = tile_column > first_tile_column
                && tile_column + 1 < end_tile_column
                && tile_row == first_tile_row;
            commands.spawn((
                Sprite {
                    image: art.wall.clone(),
                    color: if is_opening_tile {
                        opening_color
                    } else {
                        rock_color
                    },
                    ..default()
                },
                world_transform(tile_center(tile_column, tile_row), 1.8),
                MineEntrance,
                OverworldEntity,
            ));
        }
    }
}

fn fishing_dock_water_color() -> Color {
    Color::srgb(0.22, 0.42, 0.58)
}

fn fishing_dock_plank_color() -> Color {
    Color::srgb(0.52, 0.38, 0.24)
}

fn fishing_dock_tile_is_plank(
    tile_column: u32,
    tile_row: u32,
    end_tile_column: u32,
    first_tile_row: u32,
    end_tile_row: u32,
) -> bool {
    tile_row == first_tile_row
        || (tile_column == end_tile_column - 1 && tile_row < end_tile_row - 1)
}

fn spawn_fishing_dock(commands: &mut Commands, art: &OverworldArt, dock: Rect) {
    let (first_tile_column, end_tile_column, first_tile_row, end_tile_row) =
        tile_range_from_rect(dock);
    let water = fishing_dock_water_color();
    let plank = fishing_dock_plank_color();

    for tile_row in first_tile_row..end_tile_row {
        for tile_column in first_tile_column..end_tile_column {
            let on_plank = fishing_dock_tile_is_plank(
                tile_column,
                tile_row,
                end_tile_column,
                first_tile_row,
                end_tile_row,
            );
            commands.spawn((
                Sprite {
                    image: if on_plank {
                        art.path.clone()
                    } else {
                        art.grass.clone()
                    },
                    color: if on_plank { plank } else { water },
                    ..default()
                },
                world_transform(tile_center(tile_column, tile_row), 0.6),
                FishingDock,
                OverworldEntity,
            ));
        }
    }
}

fn spawn_sleep_bed_placeholder(
    commands: &mut Commands,
    art: &OverworldArt,
    first_tile_column: u32,
    end_tile_column: u32,
    first_tile_row: u32,
    end_tile_row: u32,
) {
    let bed_column = first_tile_column + (end_tile_column - first_tile_column) / 2;
    let bed_row = end_tile_row.saturating_sub(2).max(first_tile_row + 1);
    let bed_center = tile_center(bed_column, bed_row);
    commands.spawn((
        Sprite {
            image: art.wall.clone(),
            color: Color::srgb(0.55, 0.28, 0.35),
            ..default()
        },
        world_transform(bed_center, 1.6),
        SleepBed,
        HouseEntity,
        OverworldEntity,
    ));
    commands.spawn((
        Sprite {
            image: art.path.clone(),
            color: Color::srgb(0.85, 0.82, 0.75),
            ..default()
        },
        world_transform(bed_center + Vec2::new(0.0, TILE * 0.22), 1.7),
        HouseEntity,
        OverworldEntity,
    ));
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
pub struct MineEntrance;

#[derive(Component)]
pub struct FishingDock;

#[derive(Component)]
pub struct ForgeEntity;

#[derive(Component)]
pub struct HouseEntity;

#[derive(Component)]
pub struct SleepBed;

#[derive(Component)]
pub struct OverworldGrid;

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::sprites::HomesteadPlayerFrames;

    #[derive(Component)]
    struct RoofTile;

    #[test]
    fn a_roof_row_places_one_tile_on_every_column_in_range() {
        let min_tx = 1;
        let max_tx = 4;
        let tile_y = 2;
        let roof_z = 2.0;
        let color = Color::srgb(0.45, 0.22, 0.16);
        let tiles = roof_tiles(min_tx, max_tx, tile_y, roof_z, color);
        let expected: Vec<Vec3> = (min_tx..max_tx)
            .map(|tx| world_transform(tile_center(tx, tile_y), roof_z).translation)
            .collect();
        assert_eq!(tiles.len(), expected.len());
        for (translation, tile_color) in &tiles {
            assert_eq!(*tile_color, color);
            assert!(expected.contains(translation));
        }
        assert_eq!(tiles.len(), (max_tx - min_tx) as usize);
    }

    fn roof_tiles(
        min_tx: u32,
        max_tx: u32,
        tile_y: u32,
        roof_z: f32,
        color: Color,
    ) -> Vec<(Vec3, Color)> {
        let mut app = App::new();
        app.add_systems(Update, move |mut commands: Commands| {
            spawn_native_roof_row(
                &mut commands,
                NativeRoofRow {
                    image: Handle::default(),
                    color,
                    min_tx,
                    max_tx,
                    tile_y,
                    z: roof_z,
                },
                |entity| {
                    entity.insert(RoofTile);
                },
            );
        });
        app.update();
        let mut tiles = Vec::new();
        let mut drawn = app.world_mut().query::<(&Sprite, &Transform, &RoofTile)>();
        for (sprite, transform, _) in drawn.iter(app.world()) {
            tiles.push((transform.translation, sprite.color));
        }
        tiles
    }

    #[test]
    fn an_empty_roof_range_places_no_tiles() {
        let tiles = roof_tiles(3, 3, 1, 2.0, Color::WHITE);
        assert_eq!(tiles.len(), 0);
    }

    #[test]
    fn a_reversed_roof_range_places_no_tiles() {
        let tiles = roof_tiles(5, 2, 1, 2.0, Color::WHITE);
        assert_eq!(tiles.len(), 0);
    }

    #[test]
    fn fishing_dock_center_is_inside_the_fishing_dock_zone() {
        let layout = OverworldLayout::homestead();
        let dock = fishing_dock_rect();
        let zone = layout
            .zone_at(dock.center())
            .expect("dock center is a zone");
        assert_eq!(zone.zone, HomesteadZone::FishingDock);
        assert_eq!(zone.bounds, dock);
    }

    #[test]
    fn spawn_fishing_dock_places_sixteen_tiles_with_six_planks() {
        let mut app = App::new();
        let dock = fishing_dock_rect();
        let (first_tile_column, end_tile_column, first_tile_row, end_tile_row) =
            tile_range_from_rect(dock);
        app.add_systems(Update, move |mut commands: Commands| {
            let blank = Handle::default();
            let art = OverworldArt {
                grass: blank.clone(),
                path: blank.clone(),
                wall: blank.clone(),
                soil: blank.clone(),
                roof: blank.clone(),
                grid_line: blank.clone(),
                forge_furnace: blank.clone(),
                forge_workbench: blank.clone(),
                forge_anvil: blank.clone(),
                player: HomesteadPlayerFrames {
                    idle: std::array::from_fn(|_| Handle::default()),
                    walk: std::array::from_fn(|_| Handle::default()),
                    eat: std::array::from_fn(|_| Handle::default()),
                },
                animal: blank.clone(),
                animal_layout: Handle::default(),
            };
            spawn_fishing_dock(&mut commands, &art, dock);
        });
        app.update();
        let mut plank_count = 0;
        let mut water_count = 0;
        let mut tiles = app.world_mut().query::<(&Sprite, &Transform, &FishingDock)>();
        let mut seen = Vec::new();
        for (sprite, transform, _) in tiles.iter(app.world()) {
            seen.push(transform.translation);
            if sprite.color == fishing_dock_plank_color() {
                plank_count += 1;
            } else if sprite.color == fishing_dock_water_color() {
                water_count += 1;
            } else {
                panic!("unexpected dock color {:?}", sprite.color);
            }
        }
        assert_eq!(seen.len(), 16);
        assert_eq!(plank_count, 6);
        assert_eq!(water_count, 10);
        for tile_row in first_tile_row..end_tile_row {
            for tile_column in first_tile_column..end_tile_column {
                let expected = world_transform(tile_center(tile_column, tile_row), 0.6).translation;
                assert!(seen.contains(&expected));
                let on_plank = fishing_dock_tile_is_plank(
                    tile_column,
                    tile_row,
                    end_tile_column,
                    first_tile_row,
                    end_tile_row,
                );
                let sprite = tiles
                    .iter(app.world())
                    .find(|(_, transform, _)| transform.translation == expected)
                    .map(|(sprite, _, _)| sprite.color)
                    .unwrap();
                if on_plank {
                    assert_eq!(sprite, fishing_dock_plank_color());
                } else {
                    assert_eq!(sprite, fishing_dock_water_color());
                }
            }
        }
    }
}
