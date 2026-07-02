use bevy::prelude::*;

pub const ENV_ROOT: &str = "dungeon/environment";
pub const ENEMY_ROOT: &str = "dungeon/enemies";

/// 0x72 knight_m — side-scroller armored hero (matches dungeon tileset).
pub const PLAYER_IDLE: &str = "player/knight_idle_side.png";
pub const PLAYER_RUN: &str = "player/knight_run_side.png";
pub const PLAYER_ATTACK: &str = "player/knight_attack_side.png";

pub const PLAYER_FRAME_WIDTH: f32 = 16.0;
pub const PLAYER_FRAME_HEIGHT: f32 = 28.0;

pub const PLAYER_IDLE_FRAMES: usize = 4;
pub const PLAYER_RUN_FRAMES: usize = 4;
pub const PLAYER_ATTACK_FRAMES: usize = 4;

#[derive(Resource)]
pub struct DungeonArt {
    pub player_idle: Handle<Image>,
    pub player_run: Handle<Image>,
    pub player_attack: Handle<Image>,
    pub floor_ground: Handle<Image>,
    pub floor_platform: Handle<Image>,
    pub floor_ladder: Handle<Image>,
    pub wall: Handle<Image>,
    pub slime: Handle<Image>,
    pub bat: Handle<Image>,
}

impl DungeonArt {
    pub fn load(asset_server: &AssetServer) -> Self {
        Self {
            player_idle: asset_server.load(PLAYER_IDLE),
            player_run: asset_server.load(PLAYER_RUN),
            player_attack: asset_server.load(PLAYER_ATTACK),
            floor_ground: asset_server.load(format!("{ENV_ROOT}/floor_ground.png")),
            floor_platform: asset_server.load(format!("{ENV_ROOT}/floor_platform.png")),
            floor_ladder: asset_server.load(format!("{ENV_ROOT}/floor_ladder.png")),
            wall: asset_server.load(format!("{ENV_ROOT}/wall.png")),
            slime: asset_server.load(format!("{ENEMY_ROOT}/slime.png")),
            bat: asset_server.load(format!("{ENEMY_ROOT}/bat.png")),
        }
    }
}

pub fn player_frame_rect(frame: usize) -> Rect {
    let x = frame as f32 * PLAYER_FRAME_WIDTH;
    Rect {
        min: Vec2::new(x, 0.0),
        max: Vec2::new(x + PLAYER_FRAME_WIDTH, PLAYER_FRAME_HEIGHT),
    }
}