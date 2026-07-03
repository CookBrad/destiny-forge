use bevy::prelude::*;

use super::enemy::EnemyKind;
use crate::graphics::TILE;

#[derive(Clone, Copy, Debug)]
pub struct PlatformSpec {
    pub left: f32,
    pub width_tiles: u32,
    pub top_y: f32,
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
    pub ground: PlatformSpec,
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