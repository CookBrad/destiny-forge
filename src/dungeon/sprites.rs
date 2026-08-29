use bevy::prelude::*;

pub const ENV_ROOT: &str = "dungeon/environment";
pub const ENEMY_ROOT: &str = "dungeon/enemies";
pub const PROJECTILE_ROOT: &str = "dungeon/projectiles";

pub const PLAYER_COMBAT_ROOT: &str = "player/combat";
pub const WEAPON_ANIME_SWORD: &str = "player/weapons/weapon_anime_sword.png";

/// Native pixel size of `weapon_anime_sword.png` (width × height).
/// Blade is baked into the hunter sheet; this overlay is a transparent placeholder.
pub const SWORD_SPRITE_WIDTH: f32 = 12.0;
pub const SWORD_SPRITE_HEIGHT: f32 = 30.0;

/// Native pixel size of each hunter frame (width × height).
/// Uniform cell is the hit frame width so the attack strip can hold chamber + thrust.
/// Hurtbox is the body; reach is the blade. Camera/hitbox retune is a Systems follow-up.
pub const PLAYER_SPRITE_WIDTH: f32 = 343.0;
pub const PLAYER_SPRITE_HEIGHT: f32 = 160.0;

/// Floor 1 slime idle (Taste-signed hunt-density cut). Coil/hop are a later slice.
/// Hurtbox stays `ENEMY_DISPLAY_SIZE` until Combat retunes.
pub const SLIME_SPRITE_WIDTH: f32 = 83.0;
pub const SLIME_SPRITE_HEIGHT: f32 = 56.0;

pub const PLAYER_IDLE_FRAMES: usize = 4;
pub const PLAYER_RUN_FRAMES: usize = 4;
pub const PLAYER_ATTACK_FRAMES: usize = 4;

#[derive(Resource)]
pub struct DungeonArt {
    pub player_idle: Handle<Image>,
    pub player_run: Handle<Image>,
    pub player_attack: Handle<Image>,
    pub weapon_anime_sword: Handle<Image>,
    pub floor_ground: Handle<Image>,
    pub floor_platform: Handle<Image>,
    pub floor_ladder: Handle<Image>,
    pub wall: Handle<Image>,
    pub slime: Handle<Image>,
    pub slime_king: Handle<Image>,
    pub bat: Handle<Image>,
    pub goblin: Handle<Image>,
    pub skeleton: Handle<Image>,
    pub zombie: Handle<Image>,
    pub arrow: Handle<Image>,
}

impl DungeonArt {
    pub fn load(asset_server: &AssetServer) -> Self {
        Self {
            player_idle: asset_server.load(format!("{PLAYER_COMBAT_ROOT}/knight_idle_side.png")),
            player_run: asset_server.load(format!("{PLAYER_COMBAT_ROOT}/knight_run_side.png")),
            player_attack: asset_server.load(format!("{PLAYER_COMBAT_ROOT}/knight_attack_side.png")),
            weapon_anime_sword: asset_server.load(WEAPON_ANIME_SWORD),
            floor_ground: asset_server.load(format!("{ENV_ROOT}/floor_ground.png")),
            floor_platform: asset_server.load(format!("{ENV_ROOT}/floor_platform.png")),
            floor_ladder: asset_server.load(format!("{ENV_ROOT}/floor_ladder.png")),
            wall: asset_server.load(format!("{ENV_ROOT}/wall.png")),
            slime: asset_server.load(format!("{ENEMY_ROOT}/slime.png")),
            slime_king: asset_server.load(format!("{ENEMY_ROOT}/slime_king.png")),
            bat: asset_server.load(format!("{ENEMY_ROOT}/bat.png")),
            goblin: asset_server.load(format!("{ENEMY_ROOT}/goblin.png")),
            skeleton: asset_server.load(format!("{ENEMY_ROOT}/skeleton.png")),
            zombie: asset_server.load(format!("{ENEMY_ROOT}/zombie.png")),
            arrow: asset_server.load(format!("{PROJECTILE_ROOT}/arrow.png")),
        }
    }
}

pub fn player_sprite_size() -> Vec2 {
    Vec2::new(PLAYER_SPRITE_WIDTH, PLAYER_SPRITE_HEIGHT)
}

pub fn player_half_extents() -> Vec2 {
    player_sprite_size() * 0.5
}

pub fn player_frame_rect(frame: usize) -> Rect {
    let x = frame as f32 * PLAYER_SPRITE_WIDTH;
    Rect {
        min: Vec2::new(x, 0.0),
        max: Vec2::new(x + PLAYER_SPRITE_WIDTH, PLAYER_SPRITE_HEIGHT),
    }
}

pub fn slime_sprite_size() -> Vec2 {
    Vec2::new(SLIME_SPRITE_WIDTH, SLIME_SPRITE_HEIGHT)
}
