use bevy::prelude::*;

pub const ENV_ROOT: &str = "dungeon/environment";
pub const ENEMY_ROOT: &str = "dungeon/enemies";
pub const PROJECTILE_ROOT: &str = "dungeon/projectiles";

pub const PLAYER_COMBAT_ROOT: &str = "player/combat";
pub const WEAPON_ANIME_SWORD: &str = "player/weapons/weapon_anime_sword.png";

/// Native pixel size of `weapon_anime_sword.png` (width × height).
pub const SWORD_SPRITE_WIDTH: f32 = 48.0;
pub const SWORD_SPRITE_HEIGHT: f32 = 120.0;

/// Native pixel size of each combat frame cell (width × height).
/// Combat strips are [`PLAYER_IDLE_FRAMES`] cells wide at this size.
pub const PLAYER_SPRITE_WIDTH: f32 = 64.0;
pub const PLAYER_SPRITE_HEIGHT: f32 = 112.0;

pub const PLAYER_IDLE_FRAMES: usize = 4;
pub const PLAYER_RUN_FRAMES: usize = 4;
pub const PLAYER_ATTACK_FRAMES: usize = 4;

/// Expected strip width for idle/run/attack sheets.
pub const PLAYER_STRIP_WIDTH: f32 = PLAYER_SPRITE_WIDTH * PLAYER_IDLE_FRAMES as f32;

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

/// UV/source rect for combat strip frame `frame` (0..N).
pub fn player_frame_rect(frame: usize) -> Rect {
    let x = frame as f32 * PLAYER_SPRITE_WIDTH;
    Rect {
        min: Vec2::new(x, 0.0),
        max: Vec2::new(x + PLAYER_SPRITE_WIDTH, PLAYER_SPRITE_HEIGHT),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_frame_rects_tile_across_strip() {
        let r0 = player_frame_rect(0);
        let r1 = player_frame_rect(1);
        let r3 = player_frame_rect(3);

        assert!((r0.min.x - 0.0).abs() < f32::EPSILON);
        assert!((r0.max.x - PLAYER_SPRITE_WIDTH).abs() < f32::EPSILON);
        assert!((r0.max.y - PLAYER_SPRITE_HEIGHT).abs() < f32::EPSILON);

        assert!((r1.min.x - PLAYER_SPRITE_WIDTH).abs() < f32::EPSILON);
        assert!((r1.max.x - PLAYER_SPRITE_WIDTH * 2.0).abs() < f32::EPSILON);

        assert!((r3.min.x - PLAYER_SPRITE_WIDTH * 3.0).abs() < f32::EPSILON);
        assert!((r3.max.x - PLAYER_STRIP_WIDTH).abs() < f32::EPSILON);

        // Frames do not overlap.
        assert!(r0.max.x <= r1.min.x + f32::EPSILON);
    }

    #[test]
    fn higher_res_frame_size_contract() {
        assert!((PLAYER_SPRITE_WIDTH - 64.0).abs() < f32::EPSILON);
        assert!((PLAYER_SPRITE_HEIGHT - 112.0).abs() < f32::EPSILON);
        assert!((SWORD_SPRITE_WIDTH - 48.0).abs() < f32::EPSILON);
        assert!((SWORD_SPRITE_HEIGHT - 120.0).abs() < f32::EPSILON);
        assert!((PLAYER_STRIP_WIDTH - 256.0).abs() < f32::EPSILON);
    }

    #[test]
    fn half_extents_match_sprite_size() {
        let half = player_half_extents();
        assert!((half.x - 32.0).abs() < f32::EPSILON);
        assert!((half.y - 56.0).abs() < f32::EPSILON);
    }
}
