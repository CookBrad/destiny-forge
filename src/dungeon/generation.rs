use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::graphics::{DUNGEON_FLOOR_Y, TILE};

use super::enemy::EnemyKind;
use super::level::{BatSpawn, BossSpawn, EnemySpawn, GeneratedFloor, PitfallSpec, PlatformSpec};

const BACKDROP_ROWS: u32 = 6;
const PLAYER_START_X: f32 = 1.5 * TILE;
const ENTRANCE_TILES: u32 = 8;
const BOSS_ARENA_TILES: u32 = 12;
const LADDER_PAD_TILES: u32 = 3;
const MIN_WIDTH_TILES: u32 = 180;
const MIN_PIT_TILES: u32 = 4;
const MAX_PIT_TILES: u32 = 8;
const MIN_PLATFORM_HEIGHT_TILES: u32 = 5;
const MAX_PLATFORM_HEIGHT_TILES: u32 = 10;
const MIN_BRIDGE_HEIGHT_TILES: u32 = 4;
const MAX_BRIDGE_HEIGHT_TILES: u32 = 7;
const MIN_PLATFORM_WIDTH_TILES: u32 = 6;
const MAX_PLATFORM_WIDTH_TILES: u32 = 12;
const MIN_ENEMY_SPACING_TILES: u32 = 8;
const MAX_ENEMY_SPAWN_ATTEMPTS: u32 = 16;

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
    let (mut ground_segments, pitfalls) =
        generate_ground_segments(&mut rng, ENTRANCE_TILES, boss_arena_start);
    ground_segments.push(PlatformSpec {
        left: boss_arena_start as f32 * TILE,
        width_tiles: width_tiles - boss_arena_start,
        top_y: DUNGEON_FLOOR_Y,
    });

    let (mut platforms, enemies, bats) =
        generate_segments(&mut rng, ENTRANCE_TILES, boss_arena_start, &ground_segments);

    for pit in &pitfalls {
        if rng.gen_bool(0.55) {
            platforms.push(bridge_over_pit(&mut rng, pit));
        }
    }

    let boss = boss_spawn(boss_arena_start);
    let ladder_tile = width_tiles - 3;

    GeneratedFloor {
        width_tiles,
        backdrop_rows: BACKDROP_ROWS,
        ground_segments,
        pitfalls,
        platforms,
        enemies,
        bats,
        boss,
        player_start_x: PLAYER_START_X,
        ladder_tile,
    }
}

fn generate_ground_segments(
    rng: &mut StdRng,
    start_tile: u32,
    end_tile: u32,
) -> (Vec<PlatformSpec>, Vec<PitfallSpec>) {
    let mut segments = Vec::new();
    let mut pitfalls = Vec::new();

    segments.push(PlatformSpec {
        left: 0.0,
        width_tiles: start_tile,
        top_y: DUNGEON_FLOOR_Y,
    });

    let mut cursor = start_tile;

    while cursor + MIN_PIT_TILES + 8 < end_tile {
        let max_run = (end_tile - cursor).saturating_sub(MIN_PIT_TILES + 3);
        if max_run < 6 {
            break;
        }

        let run_tiles = rng.gen_range(8..18).min(max_run);
        segments.push(PlatformSpec {
            left: cursor as f32 * TILE,
            width_tiles: run_tiles,
            top_y: DUNGEON_FLOOR_Y,
        });
        cursor += run_tiles;

        if cursor + MIN_PIT_TILES + 6 >= end_tile {
            break;
        }

        if rng.gen_bool(0.44) {
            let max_pit = (end_tile - cursor)
                .saturating_sub(3)
                .min(MAX_PIT_TILES);
            if max_pit >= MIN_PIT_TILES {
                let pit_width = rng.gen_range(MIN_PIT_TILES..=max_pit);
                pitfalls.push(PitfallSpec {
                    left: cursor as f32 * TILE,
                    width_tiles: pit_width,
                });
                cursor += pit_width;
            }
        }
    }

    if cursor < end_tile {
        segments.push(PlatformSpec {
            left: cursor as f32 * TILE,
            width_tiles: end_tile - cursor,
            top_y: DUNGEON_FLOOR_Y,
        });
    }

    (segments, pitfalls)
}

fn generate_segments(
    rng: &mut StdRng,
    start_tile: u32,
    end_tile: u32,
    ground_segments: &[PlatformSpec],
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

        let enemy_count = rng.gen_range(1..=3);
        for _ in 0..enemy_count {
            if segment_end <= cursor + 4 {
                break;
            }
            if let Some(spawn) = try_spawn_enemy(
                rng,
                cursor + 2,
                segment_end - 2,
                progress,
                ground_segments,
                &enemies,
            ) {
                enemies.push(spawn);
            }
        }

        if rng.gen_bool(0.88) && segment_end > cursor + 6 {
            let platform_count = if rng.gen_bool(0.22) { 2 } else { 1 };
            for step in 0..platform_count {
                let plat_width = rng.gen_range(MIN_PLATFORM_WIDTH_TILES..=MAX_PLATFORM_WIDTH_TILES);
                let max_left = segment_end.saturating_sub(plat_width + 1);
                if max_left <= cursor + 1 {
                    continue;
                }
                let plat_left = rng.gen_range((cursor + 1)..=max_left);
                let base_height = rng.gen_range(MIN_PLATFORM_HEIGHT_TILES..=MAX_PLATFORM_HEIGHT_TILES);
                let height_tiles = base_height + step * rng.gen_range(2..=4);
                let top_y = DUNGEON_FLOOR_Y + height_tiles as f32 * TILE;

                platforms.push(PlatformSpec {
                    left: plat_left as f32 * TILE,
                    width_tiles: plat_width,
                    top_y,
                });

                if rng.gen_bool(0.6) {
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

fn try_spawn_enemy(
    rng: &mut StdRng,
    min_tile: u32,
    max_tile: u32,
    progress: f32,
    ground_segments: &[PlatformSpec],
    existing: &[EnemySpawn],
) -> Option<EnemySpawn> {
    if max_tile <= min_tile {
        return None;
    }

    for _ in 0..MAX_ENEMY_SPAWN_ATTEMPTS {
        let tile = rng.gen_range(min_tile..max_tile);
        let x = (tile as f32 + 0.5) * TILE;
        if !is_on_floor(x, ground_segments) || enemies_too_close(existing, x) {
            continue;
        }
        return Some(EnemySpawn {
            kind: pick_ground_enemy(rng, progress),
            x,
            top_y: DUNGEON_FLOOR_Y,
        });
    }

    None
}

fn enemies_too_close(existing: &[EnemySpawn], x: f32) -> bool {
    let min_dist = MIN_ENEMY_SPACING_TILES as f32 * TILE;
    existing
        .iter()
        .any(|enemy| (enemy.x - x).abs() < min_dist)
}

fn bridge_over_pit(rng: &mut StdRng, pit: &PitfallSpec) -> PlatformSpec {
    let width = (pit.width_tiles + 2).clamp(MIN_PLATFORM_WIDTH_TILES, MAX_PLATFORM_WIDTH_TILES);
    let inset = ((pit.width_tiles.saturating_sub(width)) as f32 * 0.5 * TILE).max(0.0);
    PlatformSpec {
        left: pit.left + inset,
        width_tiles: width,
        top_y: DUNGEON_FLOOR_Y
            + rng.gen_range(MIN_BRIDGE_HEIGHT_TILES..=MAX_BRIDGE_HEIGHT_TILES) as f32 * TILE,
    }
}

fn is_on_floor(x: f32, segments: &[PlatformSpec]) -> bool {
    segments.iter().any(|segment| {
        segment.top_y == DUNGEON_FLOOR_Y
            && x >= segment.left
            && x <= segment.left + segment.width_tiles as f32 * TILE
    })
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
            assert!(!floor.ground_segments.is_empty());
        }
    }

    #[test]
    fn pitfalls_are_at_least_min_width() {
        for seed in [1, 42, 999, 12_345, 98_765] {
            let floor = generate_floor(seed);
            for pit in &floor.pitfalls {
                assert!(pit.width_tiles >= MIN_PIT_TILES);
                assert!(pit.width_tiles <= MAX_PIT_TILES);
            }
        }
    }

    #[test]
    fn floating_platforms_are_wider() {
        for seed in [1, 42, 999, 12_345, 98_765] {
            let floor = generate_floor(seed);
            for platform in &floor.platforms {
                assert!(platform.width_tiles >= MIN_PLATFORM_WIDTH_TILES);
                assert!(platform.width_tiles <= MAX_PLATFORM_WIDTH_TILES);
            }
        }
    }

    #[test]
    fn enemies_spawn_with_minimum_spacing() {
        let min_dist = MIN_ENEMY_SPACING_TILES as f32 * TILE;
        for seed in [1, 42, 999, 12_345, 98_765] {
            let floor = generate_floor(seed);
            for (i, left) in floor.enemies.iter().enumerate() {
                for right in floor.enemies.iter().skip(i + 1) {
                    assert!(
                        (left.x - right.x).abs() >= min_dist,
                        "seed {seed}: enemies at {} and {} are too close",
                        left.x,
                        right.x
                    );
                }
            }
        }
    }
}