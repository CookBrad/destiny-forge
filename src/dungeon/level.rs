use bevy::prelude::*;

use super::enemy::EnemyKind;
use crate::graphics::{DUNGEON_FLOOR_Y, TILE};

const GROUND_EDGE_INSET: f32 = TILE * 0.35;

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

pub fn ground_patrol_range(x: f32, radius: f32, segments: &[PlatformSpec]) -> (f32, f32) {
    if let Some((seg_min, seg_max)) = ground_segment_bounds_at(x, segments) {
        let min_x = (x - radius).max(seg_min);
        let max_x = (x + radius).min(seg_max);
        if min_x < max_x {
            return (min_x, max_x);
        }
    }

    (x - radius, x + radius)
}

pub fn clamp_x_to_ground_segment(x: f32, segments: &[PlatformSpec]) -> Option<f32> {
    ground_segment_bounds_at(x, segments).map(|(min_x, max_x)| x.clamp(min_x, max_x))
}