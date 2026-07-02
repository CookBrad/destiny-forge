use crate::graphics::{DUNGEON_FLOOR_Y, TILE};

#[derive(Clone, Copy, Debug)]
pub struct PlatformSpec {
    pub left: f32,
    pub width_tiles: u32,
    pub top_y: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct SlimeSpawn {
    pub x: f32,
    pub top_y: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct BatSpawn {
    pub x: f32,
    pub top_y: f32,
}

pub struct FloorOne;

impl FloorOne {
    pub const GROUND: PlatformSpec = PlatformSpec {
        left: 0.0,
        width_tiles: 22,
        top_y: DUNGEON_FLOOR_Y,
    };

    pub const PLATFORMS: &'static [PlatformSpec] = &[
        PlatformSpec {
            left: 8.0 * TILE,
            width_tiles: 4,
            top_y: DUNGEON_FLOOR_Y + 4.0 * TILE,
        },
        PlatformSpec {
            left: 15.0 * TILE,
            width_tiles: 3,
            top_y: DUNGEON_FLOOR_Y + 6.0 * TILE,
        },
    ];

    pub const SLIMES: &'static [SlimeSpawn] = &[
        SlimeSpawn {
            x: 5.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
        SlimeSpawn {
            x: 12.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
    ];

    pub const BATS: &'static [BatSpawn] = &[BatSpawn {
        x: 16.5 * TILE,
        top_y: DUNGEON_FLOOR_Y + 6.0 * TILE,
    }];

    pub const PLAYER_START_X: f32 = 2.5 * TILE;
    pub const LADDER_TILE: u32 = 20;
}