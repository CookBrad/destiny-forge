use crate::graphics::{DUNGEON_FLOOR_Y, TILE};

use super::enemy::EnemyKind;
use super::level::{BatSpawn, BossSpawn, EnemySpawn, GeneratedFloor, PlatformSpec};

const WIDTH_TILES: u32 = 72;
const LADDER_TILE: u32 = WIDTH_TILES - 3;

/// Hand-authored Floor 1 layout (tests / fallback). Live runs use `generation::generate_floor`.
pub fn floor_one() -> GeneratedFloor {
    const BOSS_TILE: f32 = 58.0;
    GeneratedFloor {
        width_tiles: WIDTH_TILES,
        backdrop_rows: 6,
        ground_segments: vec![PlatformSpec {
            left: 0.0,
            width_tiles: WIDTH_TILES,
            top_y: DUNGEON_FLOOR_Y,
        }],
        pitfalls: Vec::new(),
        platforms: vec![
            PlatformSpec {
                left: 18.0 * TILE,
                width_tiles: 6,
                top_y: DUNGEON_FLOOR_Y + 5.0 * TILE,
            },
            PlatformSpec {
                left: 36.0 * TILE,
                width_tiles: 8,
                top_y: DUNGEON_FLOOR_Y + 6.0 * TILE,
            },
        ],
        enemies: vec![
            EnemySpawn {
                kind: EnemyKind::Slime,
                x: 12.0 * TILE,
                top_y: DUNGEON_FLOOR_Y,
            },
            EnemySpawn {
                kind: EnemyKind::Slime,
                x: 22.0 * TILE,
                top_y: DUNGEON_FLOOR_Y,
            },
            EnemySpawn {
                kind: EnemyKind::Goblin,
                x: 32.0 * TILE,
                top_y: DUNGEON_FLOOR_Y,
            },
            EnemySpawn {
                kind: EnemyKind::Skeleton,
                x: 42.0 * TILE,
                top_y: DUNGEON_FLOOR_Y,
            },
            EnemySpawn {
                kind: EnemyKind::Zombie,
                x: 50.0 * TILE,
                top_y: DUNGEON_FLOOR_Y,
            },
        ],
        bats: vec![
            BatSpawn {
                x: 20.0 * TILE,
                top_y: DUNGEON_FLOOR_Y + 4.0 * TILE,
            },
            BatSpawn {
                x: 40.0 * TILE,
                top_y: DUNGEON_FLOOR_Y + 5.0 * TILE,
            },
        ],
        boss: BossSpawn {
            x: (BOSS_TILE + 0.5) * TILE,
            top_y: DUNGEON_FLOOR_Y,
            patrol_min_x: 54.0 * TILE,
            patrol_max_x: 66.0 * TILE,
        },
        has_boss: true,
        player_start_x: 2.5 * TILE,
        ladder_tile: LADDER_TILE,
    }
}