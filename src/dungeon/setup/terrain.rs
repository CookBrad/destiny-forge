use bevy::prelude::*;

use crate::graphics::{center_on_surface, world_transform, DUNGEON_FLOOR_Y, TILE};

use super::super::level::{GeneratedFloor, PitfallSpec, PlatformSpec};
use super::super::sprites::DungeonArt;
use super::{DungeonEntity, DungeonExit, Pitfall, PlatformCollider};

const PIT_VOID_ROWS: u32 = 10;
const PIT_STAKE_HEIGHT: f32 = 40.0;
const PIT_LIP_HEIGHT: f32 = 16.0;
const LADDER_HEIGHT: f32 = 64.0;
/// Pixels of the lip hidden behind the opaque floor tile.
const PIT_LIP_TUCK: f32 = 4.0;

pub fn spawn_backdrop(commands: &mut Commands, art: &DungeonArt, floor: &GeneratedFloor) {
    let wall = art.wall.clone();
    for row in 0..floor.backdrop_rows {
        for column in 0..floor.width_tiles {
            // Floor tiles are centered on the tile. Index * TILE would sit the wall
            // half a tile off that grid, so the pixels would not share edges.
            let position = Vec2::new(
                column as f32 * TILE + TILE * 0.5,
                row as f32 * TILE + TILE * 0.5,
            );
            commands.spawn((
                Sprite {
                    image: wall.clone(),
                    ..default()
                },
                world_transform(position, 0.0),
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
                commands.spawn((
                    Sprite {
                        image: art.floor_pit.clone(),
                        ..default()
                    },
                    world_transform(Vec2::new(x, y), 0.35),
                    Pitfall,
                    DungeonEntity,
                ));
            }
        }
    }
}

fn spawn_pit_warning_stake(commands: &mut Commands, art: &DungeonArt, x: f32) {
    let y = center_on_surface(DUNGEON_FLOOR_Y, PIT_STAKE_HEIGHT);
    commands.spawn((
        Sprite {
            image: art.pit_stake.clone(),
            ..default()
        },
        world_transform(Vec2::new(x, y), 0.65),
        Pitfall,
        DungeonEntity,
    ));
}

fn spawn_pit_crumble_lip(commands: &mut Commands, art: &DungeonArt, x: f32) {
    let y = DUNGEON_FLOOR_Y - TILE - PIT_LIP_HEIGHT * 0.5 + PIT_LIP_TUCK;
    commands.spawn((
        Sprite {
            image: art.pit_lip.clone(),
            ..default()
        },
        world_transform(Vec2::new(x, y), 0.5),
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
            world_transform(Vec2::new(x, y), 1.0),
            collider,
            DungeonEntity,
        ));
    }
}

pub fn spawn_ladder_exit(commands: &mut Commands, art: &DungeonArt, ladder_tile: u32) {
    let x = ladder_tile as f32 * TILE + TILE * 0.5;
    let y = center_on_surface(DUNGEON_FLOOR_Y, LADDER_HEIGHT);

    commands.spawn((
        Sprite {
            image: art.floor_ladder.clone(),
            ..default()
        },
        world_transform(Vec2::new(x, y), 1.0),
        PlatformCollider {
            min_x: x - TILE * 0.5,
            max_x: x + TILE * 0.5,
            top_y: DUNGEON_FLOOR_Y,
        },
        DungeonExit,
        DungeonEntity,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pit_and_ladder_sprites_use_their_authored_heights() {
        assert_eq!(PIT_STAKE_HEIGHT, 40.0);
        assert_eq!(PIT_LIP_HEIGHT, 16.0);
        assert_eq!(LADDER_HEIGHT, 64.0);
        assert_eq!(LADDER_HEIGHT, TILE * 2.0);
        assert_eq!(PIT_VOID_ROWS, 10);
    }
}
