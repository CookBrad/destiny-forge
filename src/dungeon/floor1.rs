use crate::graphics::{DUNGEON_FLOOR_Y, TILE};

use super::enemy::EnemyKind;
use super::generation::BACKDROP_ROWS;
use super::level::{BatSpawn, BossSpawn, EnemySpawn, GeneratedFloor, PlatformSpec};

/// Half the old 72-tile fallback. 36 * 32 px matches 72 * 16 px.
const WIDTH_TILES: u32 = 36;
const LADDER_TILE: u32 = WIDTH_TILES - 3;

/// Hand-authored Floor 1 layout (tests / fallback). Live runs use `generation::generate_floor`.
pub fn floor_one() -> GeneratedFloor {
    const BOSS_TILE: f32 = 29.0;
    GeneratedFloor {
        width_tiles: WIDTH_TILES,
        backdrop_rows: BACKDROP_ROWS,
        ground_segments: vec![PlatformSpec {
            left: 0.0,
            width_tiles: WIDTH_TILES,
            top_y: DUNGEON_FLOOR_Y,
        }],
        pitfalls: Vec::new(),
        platforms: vec![
            PlatformSpec {
                left: 9.0 * TILE,
                width_tiles: 3,
                top_y: DUNGEON_FLOOR_Y + 4.0 * TILE,
            },
            PlatformSpec {
                left: 18.0 * TILE,
                width_tiles: 4,
                top_y: DUNGEON_FLOOR_Y + 6.0 * TILE,
            },
        ],
        enemies: vec![
            EnemySpawn {
                kind: EnemyKind::Slime,
                x: 6.0 * TILE,
                top_y: DUNGEON_FLOOR_Y,
            },
            EnemySpawn {
                kind: EnemyKind::Slime,
                x: 11.0 * TILE,
                top_y: DUNGEON_FLOOR_Y,
            },
            EnemySpawn {
                kind: EnemyKind::Goblin,
                x: 16.0 * TILE,
                top_y: DUNGEON_FLOOR_Y,
            },
            EnemySpawn {
                kind: EnemyKind::Skeleton,
                x: 21.0 * TILE,
                top_y: DUNGEON_FLOOR_Y,
            },
            EnemySpawn {
                kind: EnemyKind::Zombie,
                x: 25.0 * TILE,
                top_y: DUNGEON_FLOOR_Y,
            },
        ],
        bats: vec![
            BatSpawn {
                x: 10.0 * TILE,
                top_y: DUNGEON_FLOOR_Y + 3.0 * TILE,
            },
            BatSpawn {
                x: 20.0 * TILE,
                top_y: DUNGEON_FLOOR_Y + 2.0 * TILE,
            },
        ],
        boss: BossSpawn {
            x: (BOSS_TILE + 0.5) * TILE,
            top_y: DUNGEON_FLOOR_Y,
            patrol_min_x: 27.0 * TILE,
            patrol_max_x: 32.0 * TILE,
        },
        has_boss: true,
        player_start_x: 2.5 * TILE,
        ladder_tile: LADDER_TILE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::generation::{MAX_PLATFORM_HEIGHT_TILES, MIN_PLATFORM_HEIGHT_TILES};

    #[test]
    fn fallback_width_is_half_and_the_ledges_are_jumpable() {
        let floor = floor_one();
        assert_eq!(floor.width_tiles, 36);
        assert_eq!(floor.width_tiles as f32 * TILE, 72.0 * 16.0);
        assert_eq!(floor.backdrop_rows, BACKDROP_ROWS);
        assert!(floor.boss.patrol_max_x < floor.ladder_tile as f32 * TILE);
        for platform in &floor.platforms {
            let rise = platform.top_y - DUNGEON_FLOOR_Y;
            assert!(rise >= MIN_PLATFORM_HEIGHT_TILES as f32 * TILE);
            assert!(rise <= MAX_PLATFORM_HEIGHT_TILES as f32 * TILE);
        }
    }
}
