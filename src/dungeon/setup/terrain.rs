use bevy::prelude::*;

use crate::graphics::{
    center_on_surface, sole_anchor, world_transform, DUNGEON_CEILING_Y, DUNGEON_FLOOR_Y,
    HUNTER_BODY_PX, TILE,
};

use super::super::level::{GeneratedFloor, PitfallSpec, PlatformSpec};
use super::super::sprites::DungeonArt;
use super::{DungeonEntity, DungeonExit, Pitfall, PlatformCollider};

/// Repeat textures are larger than a tile so bricks and slabs read at hunter
/// scale. Each tile samples its own 32x32 cell, still drawn 1x.
const WALL_PATTERN_PX: UVec2 = UVec2::new(256, 256);
/// Floor slab row, platform girder and ceiling beam: two 64 px bays.
const STRIP_PATTERN_PX: UVec2 = UVec2::new(128, 32);
/// Foundation rock and pit shaft.
const BLOCK_PATTERN_PX: UVec2 = UVec2::new(128, 128);

/// Rows under the floor slab and down the pit shaft. Both stop at y = -256,
/// below the 720p view (-32) and a 1080p view (-212).
const GROUND_FILL_ROWS: u32 = 9;
const PIT_VOID_ROWS: u32 = 10;
/// Rock over the ceiling beam, up to y = 896, past the top of a 1080p view (868).
const CEILING_ROCK_ROWS: u32 = 9;

/// The wall darkens toward the ceiling so the floor band where the fight is stays lit.
const WALL_SHADE_FIRST_ROW: u32 = 2;
const WALL_SHADE_PER_ROW: f32 = 0.025;
const WALL_SHADE_FLOOR: f32 = 0.55;

const PIT_STAKE_HEIGHT: f32 = 112.0;
const PIT_LIP_SIZE: Vec2 = Vec2::new(24.0, 96.0);
const LADDER_SIZE: Vec2 = Vec2::new(64.0, 256.0);

const Z_WALL: f32 = 0.0;
const Z_PIT: f32 = 0.35;
const Z_STAKE: f32 = 0.65;
const Z_GROUND: f32 = 1.0;
const Z_PIT_LIP: f32 = 1.1;

/// The 32x32 cell of a repeat texture for world tile (`column`, `row`).
/// Rows count up from the bottom, image rows count down, so the pattern stays upright.
fn pattern_cell(column: u32, row: u32, pattern: UVec2) -> Rect {
    let tile = TILE as u32;
    let columns = pattern.x / tile;
    let rows = pattern.y / tile;
    let x = (column % columns) * tile;
    let y = (rows - 1 - row % rows) * tile;
    Rect::new(x as f32, y as f32, (x + tile) as f32, (y + tile) as f32)
}

fn pattern_sprite(image: &Handle<Image>, column: u32, row: u32, pattern: UVec2) -> Sprite {
    Sprite {
        image: image.clone(),
        rect: Some(pattern_cell(column, row, pattern)),
        ..default()
    }
}

/// Center of the tile whose bottom-left corner is (`column`, `bottom_y`).
fn tile_center(column: u32, bottom_y: f32) -> Vec2 {
    Vec2::new(column as f32 * TILE + TILE * 0.5, bottom_y + TILE * 0.5)
}

fn first_column(left: f32) -> u32 {
    (left / TILE).round() as u32
}

fn wall_shade(row: u32) -> f32 {
    let steps = row.saturating_sub(WALL_SHADE_FIRST_ROW) as f32;
    (1.0 - steps * WALL_SHADE_PER_ROW).max(WALL_SHADE_FLOOR)
}

pub fn spawn_backdrop(commands: &mut Commands, art: &DungeonArt, floor: &GeneratedFloor) {
    for row in 0..floor.backdrop_rows {
        let shade = wall_shade(row);
        for column in 0..floor.width_tiles {
            let mut sprite = pattern_sprite(&art.wall, column, row, WALL_PATTERN_PX);
            sprite.color = Color::srgb(shade, shade, shade);
            commands.spawn((
                sprite,
                world_transform(tile_center(column, row as f32 * TILE), Z_WALL),
                DungeonEntity,
            ));
        }
    }
}

/// Iron beam at the top of the wall, then rock above it.
pub fn spawn_ceiling(commands: &mut Commands, art: &DungeonArt, floor: &GeneratedFloor) {
    for column in 0..floor.width_tiles {
        commands.spawn((
            pattern_sprite(&art.ceiling_beam, column, 0, STRIP_PATTERN_PX),
            world_transform(tile_center(column, DUNGEON_CEILING_Y), Z_GROUND),
            DungeonEntity,
        ));
        for row in 1..=CEILING_ROCK_ROWS {
            let bottom = DUNGEON_CEILING_Y + row as f32 * TILE;
            commands.spawn((
                pattern_sprite(&art.ground_fill, column, row, BLOCK_PATTERN_PX),
                world_transform(tile_center(column, bottom), Z_GROUND),
                DungeonEntity,
            ));
        }
    }
}

pub fn spawn_ground(commands: &mut Commands, art: &DungeonArt, spec: PlatformSpec) {
    spawn_walkable_strip(commands, &art.floor_ground, spec);
    let start = first_column(spec.left);
    for column in start..start + spec.width_tiles {
        for depth in 1..=GROUND_FILL_ROWS {
            let bottom = spec.top_y - (depth + 1) as f32 * TILE;
            commands.spawn((
                pattern_sprite(
                    &art.ground_fill,
                    column,
                    GROUND_FILL_ROWS - depth,
                    BLOCK_PATTERN_PX,
                ),
                world_transform(tile_center(column, bottom), Z_GROUND),
                DungeonEntity,
            ));
        }
    }
}

pub fn spawn_platform(commands: &mut Commands, art: &DungeonArt, spec: PlatformSpec) {
    spawn_walkable_strip(commands, &art.floor_platform, spec);
}

/// One row of tiles whose top edge is the walkable surface.
fn spawn_walkable_strip(commands: &mut Commands, image: &Handle<Image>, spec: PlatformSpec) {
    let collider = PlatformCollider {
        min_x: spec.left,
        max_x: spec.left + spec.width_tiles as f32 * TILE,
        top_y: spec.top_y,
    };
    let start = first_column(spec.left);
    for column in start..start + spec.width_tiles {
        commands.spawn((
            pattern_sprite(image, column, 0, STRIP_PATTERN_PX),
            world_transform(tile_center(column, spec.top_y - TILE), Z_GROUND),
            collider,
            DungeonEntity,
        ));
    }
}

pub fn spawn_pitfalls(commands: &mut Commands, art: &DungeonArt, pitfalls: &[PitfallSpec]) {
    for pit in pitfalls {
        let pit_right = pit.left + pit.width_tiles as f32 * TILE;

        spawn_pit_warning_stake(commands, art, pit.left - TILE * 0.5);
        spawn_pit_warning_stake(commands, art, pit_right + TILE * 0.5);
        spawn_pit_crumble_lip(commands, art, pit.left + PIT_LIP_SIZE.x * 0.5, false);
        spawn_pit_crumble_lip(commands, art, pit_right - PIT_LIP_SIZE.x * 0.5, true);
        spawn_pit_shaft(commands, art, pit);
    }
}

fn spawn_pit_shaft(commands: &mut Commands, art: &DungeonArt, pit: &PitfallSpec) {
    let start = first_column(pit.left);
    for column in start..start + pit.width_tiles {
        for depth in 0..PIT_VOID_ROWS {
            let bottom = DUNGEON_FLOOR_Y - (depth + 1) as f32 * TILE;
            commands.spawn((
                pattern_sprite(
                    &art.floor_pit,
                    column,
                    PIT_VOID_ROWS - 1 - depth,
                    BLOCK_PATTERN_PX,
                ),
                world_transform(tile_center(column, bottom), Z_PIT),
                Pitfall,
                DungeonEntity,
            ));
        }
    }
}

fn spawn_pit_warning_stake(commands: &mut Commands, art: &DungeonArt, x: f32) {
    let y = center_on_surface(DUNGEON_FLOOR_Y, PIT_STAKE_HEIGHT);
    commands.spawn((
        Sprite {
            image: art.pit_stake.clone(),
            ..default()
        },
        world_transform(Vec2::new(x, y), Z_STAKE),
        Pitfall,
        DungeonEntity,
    ));
}

/// Broken floor edge hanging into the pit. The art's straight side is the
/// pit edge, so the far edge mirrors it.
fn spawn_pit_crumble_lip(commands: &mut Commands, art: &DungeonArt, x: f32, far_edge: bool) {
    let y = DUNGEON_FLOOR_Y - PIT_LIP_SIZE.y * 0.5;
    commands.spawn((
        Sprite {
            image: art.pit_lip.clone(),
            flip_x: far_edge,
            ..default()
        },
        world_transform(Vec2::new(x, y), Z_PIT_LIP),
        Pitfall,
        DungeonEntity,
    ));
}

/// Exit origin height: level with the hunter's body center, so the
/// interact distance measures across, not up the ladder.
fn ladder_exit_origin_y() -> f32 {
    center_on_surface(DUNGEON_FLOOR_Y, HUNTER_BODY_PX.y)
}

pub fn spawn_ladder_exit(commands: &mut Commands, art: &DungeonArt, ladder_tile: u32) {
    // 64 px wide: spans `ladder_tile` and the next tile.
    let x = (ladder_tile + 1) as f32 * TILE;
    let origin_above_floor = ladder_exit_origin_y() - DUNGEON_FLOOR_Y;

    commands.spawn((
        Sprite {
            image: art.floor_ladder.clone(),
            anchor: sole_anchor(LADDER_SIZE.y, origin_above_floor),
            ..default()
        },
        world_transform(Vec2::new(x, ladder_exit_origin_y()), Z_GROUND),
        PlatformCollider {
            min_x: x - LADDER_SIZE.x * 0.5,
            max_x: x + LADDER_SIZE.x * 0.5,
            top_y: DUNGEON_FLOOR_Y,
        },
        DungeonExit,
        DungeonEntity,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_sprites_are_drawn_at_hunter_scale() {
        let hunter = HUNTER_BODY_PX.y;
        assert!(
            LADDER_SIZE.y >= 1.2 * hunter,
            "the exit is taller than the hunter"
        );
        assert_eq!(LADDER_SIZE, Vec2::new(64.0, 256.0));
        assert_eq!(PIT_STAKE_HEIGHT, 112.0);
        assert_eq!(PIT_LIP_SIZE, Vec2::new(24.0, 96.0));
        assert_eq!(WALL_PATTERN_PX, UVec2::new(256, 256));
        assert_eq!(STRIP_PATTERN_PX, UVec2::new(128, 32));
        assert_eq!(BLOCK_PATTERN_PX, UVec2::new(128, 128));
        assert_eq!(TILE, 32.0);
    }

    #[test]
    fn pattern_cells_tile_upward_and_wrap() {
        assert_eq!(
            pattern_cell(0, 0, WALL_PATTERN_PX),
            Rect::new(0.0, 224.0, 32.0, 256.0)
        );
        assert_eq!(
            pattern_cell(1, 1, WALL_PATTERN_PX),
            Rect::new(32.0, 192.0, 64.0, 224.0)
        );
        assert_eq!(
            pattern_cell(8, 8, WALL_PATTERN_PX),
            pattern_cell(0, 0, WALL_PATTERN_PX)
        );
        assert_eq!(
            pattern_cell(5, 3, STRIP_PATTERN_PX),
            Rect::new(32.0, 0.0, 64.0, 32.0)
        );
    }

    #[test]
    fn ground_and_pit_reach_below_the_view_and_the_ceiling_above_it() {
        let ground_bottom = DUNGEON_FLOOR_Y - (GROUND_FILL_ROWS + 1) as f32 * TILE;
        let pit_bottom = DUNGEON_FLOOR_Y - PIT_VOID_ROWS as f32 * TILE;
        assert_eq!(ground_bottom, -256.0);
        assert_eq!(pit_bottom, ground_bottom);
        let ceiling_top = DUNGEON_CEILING_Y + (CEILING_ROCK_ROWS + 1) as f32 * TILE;
        assert_eq!(ceiling_top, 896.0);
    }

    #[test]
    fn wall_darkens_upward_and_never_goes_black() {
        assert_eq!(wall_shade(0), 1.0);
        assert_eq!(wall_shade(WALL_SHADE_FIRST_ROW), 1.0);
        assert!(wall_shade(10) < wall_shade(5));
        assert_eq!(wall_shade(40), WALL_SHADE_FLOOR);
    }

    #[test]
    fn ladder_exit_sits_level_with_the_hunter_and_its_feet_on_the_floor() {
        assert_eq!(
            ladder_exit_origin_y(),
            DUNGEON_FLOOR_Y + HUNTER_BODY_PX.y * 0.5
        );
        let anchor = sole_anchor(LADDER_SIZE.y, ladder_exit_origin_y() - DUNGEON_FLOOR_Y);
        let bottom = ladder_exit_origin_y() + LADDER_SIZE.y * (-anchor.as_vec().y - 0.5);
        assert!((bottom - DUNGEON_FLOOR_Y).abs() < 0.01);
    }
}
