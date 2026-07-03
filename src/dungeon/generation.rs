use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::graphics::{DUNGEON_FLOOR_Y, TILE};

use super::enemy::EnemyKind;
use super::level::{BatSpawn, BossSpawn, EnemySpawn, GeneratedFloor, PlatformSpec};

const BACKDROP_ROWS: u32 = 6;
const PLAYER_START_X: f32 = 1.5 * TILE;
const ENTRANCE_TILES: u32 = 8;
const BOSS_ARENA_TILES: u32 = 12;
const LADDER_PAD_TILES: u32 = 3;
const MIN_WIDTH_TILES: u32 = 180;

pub fn random_seed() -> u64 {
    rand::random()
}

pub fn generate_floor(seed: u64) -> GeneratedFloor {
    let mut rng = StdRng::seed_from_u64(seed);
    let segment_count = rng.gen_range(8..=12);

    let mut width_tiles = ENTRANCE_TILES;
    for _ in 0..segment_count {
        width_tiles += rng.gen_range(18..=26);
    }
    width_tiles += BOSS_ARENA_TILES + LADDER_PAD_TILES;
    width_tiles = width_tiles.max(MIN_WIDTH_TILES);

    let boss_arena_start = width_tiles - BOSS_ARENA_TILES - LADDER_PAD_TILES;
    let (platforms, enemies, bats) = generate_segments(&mut rng, ENTRANCE_TILES, boss_arena_start);
    let boss = boss_spawn(boss_arena_start);
    let ladder_tile = width_tiles - 3;

    GeneratedFloor {
        width_tiles,
        backdrop_rows: BACKDROP_ROWS,
        ground: PlatformSpec {
            left: 0.0,
            width_tiles,
            top_y: DUNGEON_FLOOR_Y,
        },
        platforms,
        enemies,
        bats,
        boss,
        player_start_x: PLAYER_START_X,
        ladder_tile,
    }
}

fn generate_segments(
    rng: &mut StdRng,
    start_tile: u32,
    end_tile: u32,
) -> (Vec<PlatformSpec>, Vec<EnemySpawn>, Vec<BatSpawn>) {
    let mut platforms = Vec::new();
    let mut enemies = Vec::new();
    let mut bats = Vec::new();
    let mut cursor = start_tile;
    let dungeon_span = (end_tile - start_tile).max(1) as f32;

    while cursor + 18 < end_tile {
        let segment_width = rng.gen_range(18..=28).min(end_tile - cursor);
        let segment_end = cursor + segment_width;
        let progress = (cursor.saturating_sub(start_tile) as f32 + segment_width as f32 * 0.5)
            / dungeon_span;

        let enemy_count = rng.gen_range(2..=4);
        for _ in 0..enemy_count {
            if segment_end <= cursor + 4 {
                break;
            }
            let tile = rng.gen_range((cursor + 2)..(segment_end - 2));
            enemies.push(EnemySpawn {
                kind: pick_ground_enemy(rng, progress),
                x: (tile as f32 + 0.5) * TILE,
                top_y: DUNGEON_FLOOR_Y,
            });
        }

        if rng.gen_bool(0.75) && segment_end > cursor + 6 {
            let plat_width = rng.gen_range(3..=5);
            let max_left = segment_end.saturating_sub(plat_width + 1);
            if max_left > cursor + 1 {
                let plat_left = rng.gen_range((cursor + 1)..=max_left);
                let height_tiles = rng.gen_range(4..=7);
                let top_y = DUNGEON_FLOOR_Y + height_tiles as f32 * TILE;

                platforms.push(PlatformSpec {
                    left: plat_left as f32 * TILE,
                    width_tiles: plat_width,
                    top_y,
                });

                if rng.gen_bool(0.55) {
                    bats.push(BatSpawn {
                        x: (plat_left as f32 + plat_width as f32 * 0.5) * TILE,
                        top_y,
                    });
                }
            }
        }

        cursor = segment_end;
    }

    (platforms, enemies, bats)
}

fn pick_ground_enemy(rng: &mut StdRng, progress: f32) -> EnemyKind {
    let roll: f32 = rng.gen();
    if progress < 0.25 {
        if roll < 0.65 {
            EnemyKind::Slime
        } else if roll < 0.9 {
            EnemyKind::Goblin
        } else {
            EnemyKind::Skeleton
        }
    } else if progress < 0.6 {
        if roll < 0.28 {
            EnemyKind::Slime
        } else if roll < 0.52 {
            EnemyKind::Goblin
        } else if roll < 0.78 {
            EnemyKind::Skeleton
        } else {
            EnemyKind::Zombie
        }
    } else if roll < 0.12 {
        EnemyKind::Slime
    } else if roll < 0.38 {
        EnemyKind::Goblin
    } else if roll < 0.68 {
        EnemyKind::Skeleton
    } else {
        EnemyKind::Zombie
    }
}

fn boss_spawn(arena_start: u32) -> BossSpawn {
    let center_tile = arena_start + BOSS_ARENA_TILES / 2;
    BossSpawn {
        x: (center_tile as f32 + 0.5) * TILE,
        top_y: DUNGEON_FLOOR_Y,
        patrol_min_x: (arena_start + 1) as f32 * TILE,
        patrol_max_x: (arena_start + BOSS_ARENA_TILES - 1) as f32 * TILE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_floor_always_has_boss_and_ladder_at_end() {
        for seed in [1, 42, 999, 12_345, 98_765] {
            let floor = generate_floor(seed);
            assert!(floor.ladder_tile < floor.width_tiles);
            assert!(floor.boss.patrol_max_x < floor.ladder_tile as f32 * TILE);
            assert!(floor.width_tiles >= MIN_WIDTH_TILES);
        }
    }
}