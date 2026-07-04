use bevy::prelude::*;

use super::enemy::EnemyKind;
use crate::graphics::{DUNGEON_FLOOR_Y, TILE};

const GROUND_EDGE_INSET: f32 = TILE * 0.2;

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

/// Walkable span of the floor segment under `x`, if any.
pub fn ground_segment_bounds_at(x: f32, segments: &[PlatformSpec]) -> Option<(f32, f32)> {
    segments
        .iter()
        .find(|segment| {
            segment.top_y == DUNGEON_FLOOR_Y
                && x >= segment.left
                && x < segment.left + segment.width_tiles as f32 * TILE
        })
        .map(|segment| {
            (
                segment.left + GROUND_EDGE_INSET,
                segment.left + segment.width_tiles as f32 * TILE - GROUND_EDGE_INSET,
            )
        })
}

pub fn ground_patrol_range(x: f32, radius: f32, segments: &[PlatformSpec]) -> Option<(f32, f32)> {
    ground_segment_bounds_at(x, segments).map(|(seg_min, seg_max)| {
        let mut min_x = (x - radius).max(seg_min);
        let mut max_x = (x + radius).min(seg_max);
        let min_span = TILE * 2.0;
        if max_x - min_x < min_span {
            let center = ((min_x + max_x) * 0.5).clamp(seg_min, seg_max);
            min_x = (center - min_span * 0.5).max(seg_min);
            max_x = (min_x + min_span).min(seg_max);
            min_x = (max_x - min_span).max(seg_min);
        }
        (min_x, max_x)
    })
}

/// Keep ground movement on the current floor segment; returns `(new_x, hit_edge)`.
pub fn constrain_ground_movement(
    current_x: f32,
    delta_x: f32,
    segments: &[PlatformSpec],
) -> (f32, bool) {
    let proposed = current_x + delta_x;

    if let Some((cur_min, cur_max)) = ground_segment_bounds_at(current_x, segments) {
        if proposed >= cur_min && proposed <= cur_max {
            return (proposed, false);
        }
        if delta_x > 0.0 {
            return (cur_max, true);
        }
        if delta_x < 0.0 {
            return (cur_min, true);
        }
        return (current_x, false);
    }

    if let Some(snapped) = snap_x_to_nearest_ground(current_x, segments) {
        return (snapped, true);
    }

    (current_x, false)
}

pub fn is_on_ground_floor(x: f32, segments: &[PlatformSpec]) -> bool {
    ground_segment_bounds_at(x, segments).is_some()
}

fn snap_x_to_nearest_ground(x: f32, segments: &[PlatformSpec]) -> Option<f32> {
    let mut best: Option<(f32, f32)> = None;

    for segment in segments {
        if segment.top_y != DUNGEON_FLOOR_Y {
            continue;
        }
        let seg_min = segment.left + GROUND_EDGE_INSET;
        let seg_max = segment.left + segment.width_tiles as f32 * TILE - GROUND_EDGE_INSET;
        for edge in [seg_min, seg_max] {
            let distance = (x - edge).abs();
            if best.is_none_or(|(_, best_dist)| distance < best_dist) {
                best = Some((edge, distance));
            }
        }
    }

    best.map(|(edge, _)| edge)
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

    #[test]
    fn ground_movement_stops_at_segment_edge() {
        let segments = test_segments();
        let start = 7.0 * TILE;
        let (stopped, hit_edge) = constrain_ground_movement(start, TILE * 2.0, &segments);
        assert!(hit_edge);
        assert!(stopped < 12.0 * TILE);
        assert!(is_on_ground_floor(stopped, &segments));
    }

    #[test]
    fn ground_movement_cannot_enter_pit_gap() {
        let segments = test_segments();
        let in_pit = 10.0 * TILE;
        assert!(!is_on_ground_floor(in_pit, &segments));
        let (snapped, hit_edge) = constrain_ground_movement(in_pit, 0.0, &segments);
        assert!(hit_edge);
        assert!(is_on_ground_floor(snapped, &segments));
    }
}