use bevy::prelude::*;

use crate::graphics::{scaled_transform, DUNGEON_FLOOR_Y, TILE};

use super::{DungeonEntity, DungeonExit, Pitfall, PlatformCollider};
use super::super::level::{GeneratedFloor, PitfallSpec, PlatformSpec};
use super::super::sprites::DungeonArt;

const PIT_VOID_COLOR: Color = Color::srgb(0.04, 0.02, 0.07);
const PIT_VOID_ROWS: u32 = 10;
const PIT_WARNING_STAKE_COLOR: Color = Color::srgb(0.95, 0.82, 0.18);

pub fn spawn_backdrop(commands: &mut Commands, art: &DungeonArt, floor: &GeneratedFloor) {
    let wall = art.wall.clone();
    for row in 0..floor.backdrop_rows {
        for column in 0..floor.width_tiles {
            let position = Vec2::new(column as f32 * TILE, row as f32 * TILE);
            commands.spawn((
                Sprite {
                    image: wall.clone(),
                    ..default()
                },
                scaled_transform(position, 0.0),
                DungeonEntity,
            ));
        }
    }
}

pub fn spawn_ground(commands: &mut Commands, art: &DungeonArt, spec: PlatformSpec) {
    spawn_platform_tiles(commands, art, spec, true);
}

pub fn spawn_pitfalls(commands: &mut Commands, art: &DungeonArt, pitfalls: &[PitfallSpec]) {
    for pit in pitfalls {
        let pit_right = pit.left + pit.width_tiles as f32 * TILE;

        spawn_pit_warning_stake(commands, art, pit.left - TILE * 0.5);
        spawn_pit_warning_stake(commands, art, pit_right + TILE * 0.5);
        spawn_pit_crumble_lip(commands, art, pit.left - TILE * 0.5);
        spawn_pit_crumble_lip(commands, art, pit_right + TILE * 0.5);

        for tile in 0..pit.width_tiles {
            let x = pit.left + tile as f32 * TILE + TILE * 0.5;
            for row in 1..PIT_VOID_ROWS {
                let y = DUNGEON_FLOOR_Y - TILE * (0.5 + row as f32);
                let stripe = row % 2 == 0;
                commands.spawn((
                    Sprite {
                        image: art.wall.clone(),
                        color: if stripe {
                            PIT_VOID_COLOR
                        } else {
                            Color::srgb(0.08, 0.03, 0.12)
                        },
                        ..default()
                    },
                    scaled_transform(Vec2::new(x, y), 0.35),
                    Pitfall,
                    DungeonEntity,
                ));
            }
        }
    }
}

fn spawn_pit_warning_stake(commands: &mut Commands, art: &DungeonArt, x: f32) {
    commands.spawn((
        Sprite {
            image: art.wall.clone(),
            color: PIT_WARNING_STAKE_COLOR,
            ..default()
        },
        Transform {
            translation: Vec3::new(x, DUNGEON_FLOOR_Y + TILE * 0.55, 0.65),
            scale: Vec3::new(0.28, 1.35, 1.0),
            ..default()
        },
        Pitfall,
        DungeonEntity,
    ));
}

fn spawn_pit_crumble_lip(commands: &mut Commands, art: &DungeonArt, x: f32) {
    commands.spawn((
        Sprite {
            image: art.floor_ground.clone(),
            color: Color::srgb(0.28, 0.22, 0.26),
            ..default()
        },
        Transform {
            translation: Vec3::new(x, DUNGEON_FLOOR_Y - TILE * 1.1, 0.5),
            scale: Vec3::new(0.75, 0.55, 1.0),
            ..default()
        },
        Pitfall,
        DungeonEntity,
    ));
}

pub fn spawn_platform(commands: &mut Commands, art: &DungeonArt, spec: PlatformSpec) {
    spawn_platform_tiles(commands, art, spec, false);
}

fn spawn_platform_tiles(
    commands: &mut Commands,
    art: &DungeonArt,
    spec: PlatformSpec,
    ground: bool,
) {
    let texture = if ground {
        art.floor_ground.clone()
    } else {
        art.floor_platform.clone()
    };

    let width = spec.width_tiles as f32 * TILE;
    let collider = PlatformCollider {
        min_x: spec.left,
        max_x: spec.left + width,
        top_y: spec.top_y,
    };

    for tile in 0..spec.width_tiles {
        let x = spec.left + tile as f32 * TILE + TILE * 0.5;
        let y = spec.top_y - TILE * 0.5;
        commands.spawn((
            Sprite {
                image: texture.clone(),
                ..default()
            },
            scaled_transform(Vec2::new(x, y), 1.0),
            collider,
            DungeonEntity,
        ));
    }
}

pub fn spawn_ladder_exit(commands: &mut Commands, art: &DungeonArt, ladder_tile: u32) {
    let x = ladder_tile as f32 * TILE + TILE * 0.5;
    let y = DUNGEON_FLOOR_Y - TILE * 0.5;

    commands.spawn((
        Sprite {
            image: art.floor_ladder.clone(),
            ..default()
        },
        scaled_transform(Vec2::new(x, y), 1.0),
        PlatformCollider {
            min_x: x - TILE * 0.5,
            max_x: x + TILE * 0.5,
            top_y: DUNGEON_FLOOR_Y,
        },
        DungeonExit,
        DungeonEntity,
    ));
}