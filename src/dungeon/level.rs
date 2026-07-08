use bevy::prelude::*;

use super::enemy::EnemyKind;
use crate::graphics::{DUNGEON_FLOOR_Y, ENEMY_DISPLAY_SIZE, TILE};

fn enemy_half_width() -> f32 {
    ENEMY_DISPLAY_SIZE.x * 0.5
}

#[derive(Clone, Copy, Debug)]
pub struct PlatformSpec {
    pub left: f32,
    pub width_tiles: u32,
    pub top_y: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PitfallSpec {
    pub left: f32,
    pub width_tiles: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct EnemySpawn {
    pub kind: EnemyKind,
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

#[derive(Clone, Debug)]
pub struct GeneratedFloor {
    pub width_tiles: u32,
    pub backdrop_rows: u32,
    pub ground_segments: Vec<PlatformSpec>,
    pub pitfalls: Vec<PitfallSpec>,
    pub platforms: Vec<PlatformSpec>,
    pub enemies: Vec<EnemySpawn>,
    pub bats: Vec<BatSpawn>,
    pub boss: BossSpawn,
    pub has_boss: bool,
    pub player_start_x: f32,
    pub ladder_tile: u32,
}

impl GeneratedFloor {
    pub fn width_pixels(&self) -> f32 {
        self.width_tiles as f32 * TILE
    }
}

#[derive(Resource, Clone, Debug)]
pub struct DungeonLayout {
    pub seed: u64,
    pub floor: GeneratedFloor,
}

pub fn find_ground_segment_at<'a>(x: f32, segments: &'a [PlatformSpec]) -> Option<&'a PlatformSpec> {
    segments.iter().find(|segment| {
        segment.top_y == DUNGEON_FLOOR_Y
            && x >= segment.left
            && x < segment.left + segment.width_tiles as f32 * TILE
    })
}

pub fn segment_walk_bounds(segment: &PlatformSpec) -> (f32, f32) {
    let half = enemy_half_width();
    (
        segment.left + half,
        segment.left + segment.width_tiles as f32 * TILE - half,
    )
}

/// Walkable span on the floor segment under `x`.
pub fn ground_walk_bounds_at(x: f32, segments: &[PlatformSpec]) -> Option<(f32, f32)> {
    find_ground_segment_at(x, segments).map(segment_walk_bounds)
}

pub fn ground_patrol_range(x: f32, segments: &[PlatformSpec]) -> Option<(f32, f32)> {
    ground_walk_bounds_at(x, segments)
}

pub fn is_on_ground_floor(x: f32, segments: &[PlatformSpec]) -> bool {
    find_ground_segment_at(x, segments).is_some()
}

/// Keep horizontal movement on the current floor segment; returns `(new_x, hit_edge)`.
pub fn constrain_ground_walk(current_x: f32, delta_x: f32, segments: &[PlatformSpec]) -> (f32, bool) {
    let proposed = current_x + delta_x;

    let Some((walk_min, walk_max)) = ground_walk_bounds_at(current_x, segments) else {
        return (current_x, false);
    };

    if proposed >= walk_min && proposed <= walk_max {
        return (proposed, false);
    }

    if delta_x > 0.0 {
        (walk_max, true)
    } else if delta_x < 0.0 {
        (walk_min, true)
    } else {
        (current_x, false)
    }
}

pub fn is_over_pit_gap(x: f32, pitfalls: &[PitfallSpec]) -> bool {
    pitfalls.iter().any(|pit| {
        let pit_left = pit.left;
        let pit_right = pit.left + pit.width_tiles as f32 * TILE;
        x > pit_left && x < pit_right
    })
}

pub fn pit_bounds(pit: &PitfallSpec) -> (f32, f32) {
    (pit.left, pit.left + pit.width_tiles as f32 * TILE)
}

/// Pit immediately beyond the walk edge the goblin is standing on, if any.
pub fn adjacent_pit_from_edge(
    x: f32,
    direction: f32,
    segments: &[PlatformSpec],
    pitfalls: &[PitfallSpec],
) -> Option<PitfallSpec> {
    let direction = direction.signum();
    if direction == 0.0 {
        return None;
    }

    let (walk_min, walk_max) = ground_walk_bounds_at(x, segments)?;
    let at_edge = if direction > 0.0 {
        (walk_max - x).abs() <= 2.0
    } else {
        (x - walk_min).abs() <= 2.0
    };
    if !at_edge {
        return None;
    }

    pitfalls.iter().copied().find(|pit| {
        let (pit_left, pit_right) = pit_bounds(pit);
        if direction > 0.0 {
            (pit_left - walk_max).abs() <= TILE * 0.5
                && is_on_ground_floor(pit_right + enemy_half_width(), segments)
        } else {
            (pit_right - walk_min).abs() <= TILE * 0.5
                && is_on_ground_floor(pit_left - enemy_half_width(), segments)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_segments() -> Vec<PlatformSpec> {
        vec![
            PlatformSpec {
                left: 0.0,
                width_tiles: 8,
                top_y: DUNGEON_FLOOR_Y,
            },
            PlatformSpec {
                left: 12.0 * TILE,
                width_tiles: 10,
                top_y: DUNGEON_FLOOR_Y,
            },
        ]
    }

    fn test_pit() -> PitfallSpec {
        PitfallSpec {
            left: 8.0 * TILE,
            width_tiles: 4,
        }
    }

    #[test]
    fn ground_walk_uses_full_segment_span() {
        let segments = test_segments();
        let (min_x, max_x) = ground_walk_bounds_at(4.0 * TILE, &segments).unwrap();
        assert_eq!(min_x, enemy_half_width());
        assert_eq!(max_x, 8.0 * TILE - enemy_half_width());
    }

    #[test]
    fn ground_walk_stops_at_segment_edge_not_mid_platform() {
        let segments = test_segments();
        let start = 7.0 * TILE;
        let (stopped, hit_edge) = constrain_ground_walk(start, TILE * 2.0, &segments);
        assert!(hit_edge);
        assert!(stopped < 12.0 * TILE);
        assert!(is_on_ground_floor(stopped, &segments));
    }

    #[test]
    fn adjacent_pit_detected_from_walk_edge() {
        let segments = test_segments();
        let pit = test_pit();
        let edge_x = 8.0 * TILE - enemy_half_width();
        assert!(adjacent_pit_from_edge(edge_x, 1.0, &segments, &[pit]).is_some());
        assert!(adjacent_pit_from_edge(4.0 * TILE, 1.0, &segments, &[pit]).is_none());
    }

    #[test]
    fn pit_gap_is_between_floor_segments() {
        let pit = test_pit();
        let (pit_left, pit_right) = pit_bounds(&pit);
        assert!(is_over_pit_gap((pit_left + pit_right) * 0.5, &[pit]));
        assert!(!is_over_pit_gap(pit_left, &[pit]));
    }
}