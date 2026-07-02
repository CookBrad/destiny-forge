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

#[derive(Clone, Copy, Debug)]
pub struct BossSpawn {
    pub x: f32,
    pub top_y: f32,
    pub patrol_min_x: f32,
    pub patrol_max_x: f32,
}

/// Hand-authored Floor 1 — entrance → platforms → boss arena → ladder exit.
pub struct FloorOne;

impl FloorOne {
    pub const WIDTH_TILES: u32 = 72;
    pub const BACKDROP_ROWS: u32 = 6;

    pub const GROUND: PlatformSpec = PlatformSpec {
        left: 0.0,
        width_tiles: Self::WIDTH_TILES,
        top_y: DUNGEON_FLOOR_Y,
    };

    pub const PLATFORMS: &'static [PlatformSpec] = &[
        // Early hop — teaches jumping.
        PlatformSpec {
            left: 13.0 * TILE,
            width_tiles: 4,
            top_y: DUNGEON_FLOOR_Y + 4.0 * TILE,
        },
        // Mid-floor stepping stones.
        PlatformSpec {
            left: 22.0 * TILE,
            width_tiles: 3,
            top_y: DUNGEON_FLOOR_Y + 5.0 * TILE,
        },
        PlatformSpec {
            left: 28.0 * TILE,
            width_tiles: 4,
            top_y: DUNGEON_FLOOR_Y + 4.0 * TILE,
        },
        PlatformSpec {
            left: 36.0 * TILE,
            width_tiles: 3,
            top_y: DUNGEON_FLOOR_Y + 7.0 * TILE,
        },
        PlatformSpec {
            left: 44.0 * TILE,
            width_tiles: 5,
            top_y: DUNGEON_FLOOR_Y + 5.0 * TILE,
        },
        // Short climb before the arena.
        PlatformSpec {
            left: 53.0 * TILE,
            width_tiles: 3,
            top_y: DUNGEON_FLOOR_Y + 4.0 * TILE,
        },
    ];

    pub const SLIMES: &'static [SlimeSpawn] = &[
        SlimeSpawn {
            x: 5.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
        SlimeSpawn {
            x: 11.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
        SlimeSpawn {
            x: 20.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
        SlimeSpawn {
            x: 27.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
        SlimeSpawn {
            x: 34.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
        SlimeSpawn {
            x: 41.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
        SlimeSpawn {
            x: 48.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
    ];

    pub const BATS: &'static [BatSpawn] = &[
        BatSpawn {
            x: 15.5 * TILE,
            top_y: DUNGEON_FLOOR_Y + 4.0 * TILE,
        },
        BatSpawn {
            x: 30.5 * TILE,
            top_y: DUNGEON_FLOOR_Y + 4.0 * TILE,
        },
        BatSpawn {
            x: 38.5 * TILE,
            top_y: DUNGEON_FLOOR_Y + 7.0 * TILE,
        },
        BatSpawn {
            x: 47.5 * TILE,
            top_y: DUNGEON_FLOOR_Y + 5.0 * TILE,
        },
    ];

    pub const BOSS: BossSpawn = BossSpawn {
        x: 62.5 * TILE,
        top_y: DUNGEON_FLOOR_Y,
        patrol_min_x: 58.0 * TILE,
        patrol_max_x: 66.0 * TILE,
    };

    pub const PLAYER_START_X: f32 = 2.5 * TILE;
    pub const LADDER_TILE: u32 = 69;
}