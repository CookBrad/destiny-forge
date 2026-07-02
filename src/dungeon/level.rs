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
    /// Long enough to scroll on a 1280px-wide window with extra runway.
    pub const WIDTH_TILES: u32 = 120;
    pub const BACKDROP_ROWS: u32 = 6;

    pub const fn width_pixels() -> f32 {
        Self::WIDTH_TILES as f32 * TILE
    }

    pub const GROUND: PlatformSpec = PlatformSpec {
        left: 0.0,
        width_tiles: Self::WIDTH_TILES,
        top_y: DUNGEON_FLOOR_Y,
    };

    pub const PLATFORMS: &'static [PlatformSpec] = &[
        PlatformSpec {
            left: 22.0 * TILE,
            width_tiles: 4,
            top_y: DUNGEON_FLOOR_Y + 4.0 * TILE,
        },
        PlatformSpec {
            left: 37.0 * TILE,
            width_tiles: 3,
            top_y: DUNGEON_FLOOR_Y + 5.0 * TILE,
        },
        PlatformSpec {
            left: 47.0 * TILE,
            width_tiles: 4,
            top_y: DUNGEON_FLOOR_Y + 4.0 * TILE,
        },
        PlatformSpec {
            left: 60.0 * TILE,
            width_tiles: 3,
            top_y: DUNGEON_FLOOR_Y + 7.0 * TILE,
        },
        PlatformSpec {
            left: 73.0 * TILE,
            width_tiles: 5,
            top_y: DUNGEON_FLOOR_Y + 5.0 * TILE,
        },
        PlatformSpec {
            left: 88.0 * TILE,
            width_tiles: 3,
            top_y: DUNGEON_FLOOR_Y + 4.0 * TILE,
        },
    ];

    pub const SLIMES: &'static [SlimeSpawn] = &[
        SlimeSpawn {
            x: 8.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
        SlimeSpawn {
            x: 18.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
        SlimeSpawn {
            x: 33.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
        SlimeSpawn {
            x: 45.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
        SlimeSpawn {
            x: 57.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
        SlimeSpawn {
            x: 68.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
        SlimeSpawn {
            x: 80.0 * TILE,
            top_y: DUNGEON_FLOOR_Y,
        },
    ];

    pub const BATS: &'static [BatSpawn] = &[
        BatSpawn {
            x: 26.0 * TILE,
            top_y: DUNGEON_FLOOR_Y + 4.0 * TILE,
        },
        BatSpawn {
            x: 51.0 * TILE,
            top_y: DUNGEON_FLOOR_Y + 4.0 * TILE,
        },
        BatSpawn {
            x: 64.0 * TILE,
            top_y: DUNGEON_FLOOR_Y + 7.0 * TILE,
        },
        BatSpawn {
            x: 79.0 * TILE,
            top_y: DUNGEON_FLOOR_Y + 5.0 * TILE,
        },
    ];

    pub const BOSS: BossSpawn = BossSpawn {
        x: 104.5 * TILE,
        top_y: DUNGEON_FLOOR_Y,
        patrol_min_x: 100.0 * TILE,
        patrol_max_x: 108.0 * TILE,
    };

    pub const PLAYER_START_X: f32 = 1.5 * TILE;
    pub const LADDER_TILE: u32 = 117;
}