use bevy::prelude::*;

pub const ENV_ROOT: &str = "dungeon/environment";
pub const OVERWORLD_ROOT: &str = "overworld";
pub const PLAYER_NON_COMBAT_ROOT: &str = "player/non-combat";
pub const ANIMAL_SHEET: &str = "overworld/animals/quadraped.png";

pub const PLAYER_SPRITE_WIDTH: f32 = 16.0;
pub const PLAYER_SPRITE_HEIGHT: f32 = 28.0;
pub const PLAYER_ANIM_FRAMES: usize = 4;

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
    pub forge_furnace: Handle<Image>,
    pub forge_workbench: Handle<Image>,
    pub forge_anvil: Handle<Image>,
    pub player: HomesteadPlayerFrames,
    pub animal: Handle<Image>,
}

impl OverworldArt {
    pub fn load(asset_server: &AssetServer) -> Self {
        Self {
            grass: asset_server.load(format!("{ENV_ROOT}/floor_ground.png")),
            path: asset_server.load(format!("{ENV_ROOT}/floor_platform.png")),
            wall: asset_server.load(format!("{ENV_ROOT}/wall.png")),
            soil: asset_server.load(format!("{ENV_ROOT}/floor_ground.png")),
            roof: asset_server.load(format!("{ENV_ROOT}/floor_platform.png")),
            grid_line: asset_server.load(format!("{ENV_ROOT}/floor_ground.png")),
            forge_furnace: asset_server.load(format!("{OVERWORLD_ROOT}/forge_furnace.png")),
            forge_workbench: asset_server.load(format!("{OVERWORLD_ROOT}/forge_workbench.png")),
            forge_anvil: asset_server.load(format!("{OVERWORLD_ROOT}/forge_anvil.png")),
            player: HomesteadPlayerFrames::load(asset_server),
            animal: asset_server.load(ANIMAL_SHEET),
        }
    }
}

pub fn animal_frame_rect(index: usize) -> Rect {
    animal_anim_rect(index / 4, index % 4)
}

pub fn animal_anim_rect(sheet_row: usize, anim_frame: usize) -> Rect {
    let x = (anim_frame % 4) as f32 * 16.0;
    let y = sheet_row as f32 * 16.0;
    Rect {
        min: Vec2::new(x, y),
        max: Vec2::new(x + 16.0, y + 16.0),
    }
}