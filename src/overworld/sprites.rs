use bevy::prelude::*;

pub const ENV_ROOT: &str = "dungeon/environment";
pub const ANIMAL_SHEET: &str = "source/dawnlike/Characters/Quadraped0.png";
const HOMESTEAD_FRAME_ROOT: &str = "source/0x72_DungeonTilesetII_v1.7/frames";

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
                    "{HOMESTEAD_FRAME_ROOT}/dwarf_m_idle_anim_f{frame}.png"
                ))
            }),
            walk: std::array::from_fn(|frame| {
                asset_server.load(format!(
                    "{HOMESTEAD_FRAME_ROOT}/dwarf_m_run_anim_f{frame}.png"
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
    pub fence: Handle<Image>,
    pub roof: Handle<Image>,
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
            fence: asset_server.load(format!("{ENV_ROOT}/floor_ladder.png")),
            roof: asset_server.load(format!("{ENV_ROOT}/floor_platform.png")),
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