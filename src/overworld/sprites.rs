use bevy::prelude::*;

pub const ENV_ROOT: &str = "dungeon/environment";
pub const OVERWORLD_ROOT: &str = "overworld";
pub const PLAYER_NON_COMBAT_ROOT: &str = "player/non-combat";
pub const ANIMAL_SHEET: &str = "overworld/animals/quadraped.png";
pub const ANIMAL_CELL: u32 = 16;
pub const ANIMAL_SHEET_COLS: u32 = 8;
pub const ANIMAL_SHEET_ROWS: u32 = 12;

/// Match the homestead player footprint so camera zoom applies uniformly.
pub const ANIMAL_DISPLAY_SIZE: Vec2 = Vec2::new(PLAYER_SPRITE_WIDTH, PLAYER_SPRITE_HEIGHT);

pub const PLAYER_SPRITE_WIDTH: f32 = 16.0;
pub const PLAYER_SPRITE_HEIGHT: f32 = 28.0;
pub const PLAYER_ANIM_FRAMES: usize = 4;

pub const FORGE_FURNACE_HEIGHT: f32 = 74.0;
pub const FORGE_WORKBENCH_HEIGHT: f32 = 160.0;
pub const FORGE_ANVIL_HEIGHT: f32 = 80.0;

#[derive(Clone)]
pub struct HomesteadPlayerFrames {
    pub idle: [Handle<Image>; PLAYER_ANIM_FRAMES],
    pub walk: [Handle<Image>; PLAYER_ANIM_FRAMES],
}

impl HomesteadPlayerFrames {
    pub fn load(asset_server: &AssetServer) -> Self {
        Self {
            idle: std::array::from_fn(|frame| {
                asset_server.load(format!(
                    "{PLAYER_NON_COMBAT_ROOT}/dwarf_m_idle_anim_f{frame}.png"
                ))
            }),
            walk: std::array::from_fn(|frame| {
                asset_server.load(format!(
                    "{PLAYER_NON_COMBAT_ROOT}/dwarf_m_run_anim_f{frame}.png"
                ))
            }),
        }
    }

    pub fn frame_handle(&self, moving: bool, frame: usize) -> Handle<Image> {
        let index = frame % PLAYER_ANIM_FRAMES;
        if moving {
            self.walk[index].clone()
        } else {
            self.idle[index].clone()
        }
    }
}

#[derive(Resource)]
pub struct OverworldArt {
    pub grass: Handle<Image>,
    pub path: Handle<Image>,
    pub wall: Handle<Image>,
    pub soil: Handle<Image>,
    pub roof: Handle<Image>,
    pub grid_line: Handle<Image>,
    /// Cluster of planted seeds for farm plot day-0 visuals.
    pub seed: Handle<Image>,
    pub forge_furnace: Handle<Image>,
    pub forge_workbench: Handle<Image>,
    pub forge_anvil: Handle<Image>,
    pub player: HomesteadPlayerFrames,
    pub animal: Handle<Image>,
    pub animal_layout: Handle<TextureAtlasLayout>,
}

impl OverworldArt {
    pub fn load(asset_server: &AssetServer, layouts: &mut Assets<TextureAtlasLayout>) -> Self {
        let animal = asset_server.load(ANIMAL_SHEET);
        let animal_layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(ANIMAL_CELL, ANIMAL_CELL),
            ANIMAL_SHEET_COLS,
            ANIMAL_SHEET_ROWS,
            None,
            None,
        ));

        Self {
            grass: asset_server.load(format!("{ENV_ROOT}/floor_ground.png")),
            path: asset_server.load(format!("{ENV_ROOT}/floor_platform.png")),
            wall: asset_server.load(format!("{ENV_ROOT}/wall.png")),
            soil: asset_server.load(format!("{ENV_ROOT}/floor_ground.png")),
            roof: asset_server.load(format!("{ENV_ROOT}/floor_platform.png")),
            grid_line: asset_server.load(format!("{ENV_ROOT}/floor_ground.png")),
            seed: asset_server.load(format!("{OVERWORLD_ROOT}/seed.png")),
            forge_furnace: asset_server.load(format!("{OVERWORLD_ROOT}/forge_furnace.png")),
            forge_workbench: asset_server.load(format!("{OVERWORLD_ROOT}/forge_workbench.png")),
            forge_anvil: asset_server.load(format!("{OVERWORLD_ROOT}/forge_anvil.png")),
            player: HomesteadPlayerFrames::load(asset_server),
            animal,
            animal_layout,
        }
    }
}

pub fn animal_atlas_index(creature: usize, frame: usize) -> usize {
    let row = creature;
    let col = frame % 4;
    row * ANIMAL_SHEET_COLS as usize + col
}