use bevy::prelude::*;

pub const ENV_ROOT: &str = "dungeon/environment";
pub const ANIMAL_SHEET: &str = "source/dawnlike/Characters/Quadraped0.png";

pub const PLAYER_SPRITE_WIDTH: f32 = 16.0;
pub const PLAYER_SPRITE_HEIGHT: f32 = 28.0;
pub const PLAYER_IDLE_FRAMES: usize = 4;

pub fn player_frame_rect(frame: usize) -> Rect {
    let x = frame as f32 * PLAYER_SPRITE_WIDTH;
    Rect {
        min: Vec2::new(x, 0.0),
        max: Vec2::new(x + PLAYER_SPRITE_WIDTH, PLAYER_SPRITE_HEIGHT),
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
    pub player_idle: Handle<Image>,
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
            player_idle: asset_server.load("player/knight_idle_side.png"),
            animal: asset_server.load(ANIMAL_SHEET),
        }
    }
}

pub fn animal_frame_rect(index: usize) -> Rect {
    let x = (index % 4) as f32 * 16.0;
    let y = (index / 4) as f32 * 16.0;
    Rect {
        min: Vec2::new(x, y),
        max: Vec2::new(x + 16.0, y + 16.0),
    }
}