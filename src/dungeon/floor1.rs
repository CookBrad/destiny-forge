use crate::graphics::{DUNGEON_FLOOR_Y, TILE};

use super::enemy::EnemyKind;
use super::level::{BatSpawn, BossSpawn, EnemySpawn, GeneratedFloor, PlatformSpec};

const WIDTH_TILES: u32 = 72;
const LADDER_TILE: u32 = WIDTH_TILES - 3;

/// Hand-authored Floor 1 for the Phase 1 vertical slice: 2 Slimes, 1 Bat, ladder exit.
pub fn floor_one() -> GeneratedFloor {
    GeneratedFloor {
        width_tiles: WIDTH_TILES,
        backdrop_rows: 6,
        ground_segments: vec![PlatformSpec {
            left: 0.0,
            width_tiles: WIDTH_TILES,
            top_y: DUNGEON_FLOOR_Y,
        }],
        pitfalls: Vec::new(),
        platforms: Vec::new(),
        enemies: vec![
            EnemySpawn {
                kind: EnemyKind::Slime,
                x: 14.0 * TILE,
                top_y: DUNGEON_FLOOR_Y,
            },
            EnemySpawn {
                kind: EnemyKind::Slime,
                x: 30.0 * TILE,
                top_y: DUNGEON_FLOOR_Y,
            },
        ],
        bats: vec![BatSpawn {
            x: 46.0 * TILE,
            top_y: DUNGEON_FLOOR_Y - 5.0 * TILE,
        }],
        boss: BossSpawn {
            x: 0.0,
            top_y: DUNGEON_FLOOR_Y,
            patrol_min_x: 0.0,
            patrol_max_x: 0.0,
        },
        has_boss: false,
        player_start_x: 2.5 * TILE,
        ladder_tile: LADDER_TILE,
    }
}