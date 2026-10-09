use bevy::prelude::*;

use crate::graphics::{
    center_on_surface, world_transform, DUNGEON_CEILING_Y, DUNGEON_FLOOR_Y, TILE,
};

use super::super::level::{is_on_ground_floor, GeneratedFloor};
use super::super::sprites::DungeonArt;
use super::DungeonEntity;

const BAY_TILES: u32 = 12;
const FIRST_PILLAR_TILE: u32 = 2;
const PILLAR_SIZE: Vec2 = Vec2::new(64.0, 512.0);
const TORCH_SIZE: Vec2 = Vec2::new(32.0, 64.0);
const CHAIN_SIZE: Vec2 = Vec2::new(16.0, 160.0);
const TORCH_BOTTOM_ABOVE_FLOOR: f32 = 208.0;
const CHAIN_OFFSET_TILES: u32 = 3;
const TORCH_OFFSET_TILES: u32 = 6;

const Z_PILLAR: f32 = 0.2;
const Z_WALL_PROP: f32 = 0.25;

pub fn spawn_props(commands: &mut Commands, art: &DungeonArt, floor: &GeneratedFloor) {
    for bay_start in bay_starts(floor.width_tiles) {
        if pillar_has_floor(bay_start, floor) {
            spawn_prop(commands, &art.pillar, pillar_position(bay_start), Z_PILLAR);
        }
        let chain_column = bay_start + CHAIN_OFFSET_TILES;
        let torch_column = bay_start + TORCH_OFFSET_TILES;
        if chain_column < floor.width_tiles {
            spawn_prop(
                commands,
                &art.chain,
                chain_position(chain_column),
                Z_WALL_PROP,
            );
        }
        if torch_column < floor.width_tiles {
            spawn_prop(
                commands,
                &art.torch,
                torch_position(torch_column),
                Z_WALL_PROP,
            );
        }
    }
}

fn bay_starts(width_tiles: u32) -> impl Iterator<Item = u32> {
    (FIRST_PILLAR_TILE..width_tiles).step_by(BAY_TILES as usize)
}

fn spawn_prop(commands: &mut Commands, image: &Handle<Image>, position: Vec2, z: f32) {
    commands.spawn((
        Sprite {
            image: image.clone(),
            ..default()
        },
        world_transform(position, z),
        DungeonEntity,
    ));
}

fn pillar_has_floor(column: u32, floor: &GeneratedFloor) -> bool {
    let pillar_tiles = (PILLAR_SIZE.x / TILE) as u32;
    if column + pillar_tiles > floor.width_tiles {
        return false;
    }
    let left = column as f32 * TILE + 1.0;
    let right = (column + pillar_tiles) as f32 * TILE - 1.0;
    is_on_ground_floor(left, &floor.ground_segments)
        && is_on_ground_floor(right, &floor.ground_segments)
}

fn pillar_position(column: u32) -> Vec2 {
    Vec2::new(
        column as f32 * TILE + PILLAR_SIZE.x * 0.5,
        center_on_surface(DUNGEON_FLOOR_Y, PILLAR_SIZE.y),
    )
}

fn chain_position(column: u32) -> Vec2 {
    Vec2::new(
        column as f32 * TILE + TILE * 0.5,
        DUNGEON_CEILING_Y - CHAIN_SIZE.y * 0.5,
    )
}

fn torch_position(column: u32) -> Vec2 {
    Vec2::new(
        column as f32 * TILE + TILE * 0.5,
        center_on_surface(DUNGEON_FLOOR_Y + TORCH_BOTTOM_ABOVE_FLOOR, TORCH_SIZE.y),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::HUNTER_BODY_PX;

    #[test]
    fn pillar_runs_floor_to_ceiling_beam() {
        let center = pillar_position(4).y;
        assert_eq!(center - PILLAR_SIZE.y * 0.5, DUNGEON_FLOOR_Y);
        assert_eq!(center + PILLAR_SIZE.y * 0.5, DUNGEON_CEILING_Y);
    }

    #[test]
    fn torches_and_chains_stay_over_the_hunters_head() {
        let head = DUNGEON_FLOOR_Y + HUNTER_BODY_PX.y;
        assert!(torch_position(6).y - TORCH_SIZE.y * 0.5 > head);
        let chain_bottom = chain_position(3).y - CHAIN_SIZE.y * 0.5;
        assert!(chain_bottom > head);
        assert_eq!(chain_position(3).y + CHAIN_SIZE.y * 0.5, DUNGEON_CEILING_Y);
    }

    #[test]
    fn pillars_skip_pits() {
        let mut floor = crate::dungeon::floor1::floor_one();
        assert!(pillar_has_floor(2, &floor));
        floor.ground_segments = vec![crate::dungeon::level::PlatformSpec {
            left: 0.0,
            width_tiles: 3,
            top_y: DUNGEON_FLOOR_Y,
        }];
        assert!(!pillar_has_floor(2, &floor));
        assert!(!pillar_has_floor(floor.width_tiles - 1, &floor));
    }
}
