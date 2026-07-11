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

/// Expected pixel size of idle/run/attack combat strips (width × height).
pub fn player_strip_pixel_size() -> (u32, u32) {
    (
        (PLAYER_SPRITE_WIDTH * PLAYER_IDLE_FRAMES as f32) as u32,
        PLAYER_SPRITE_HEIGHT as u32,
    )
}

/// Expected pixel size of one combat/homestead frame cell.
pub fn player_cell_pixel_size() -> (u32, u32) {
    (PLAYER_SPRITE_WIDTH as u32, PLAYER_SPRITE_HEIGHT as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
        assert_eq!(PLAYER_IDLE_FRAMES, 4);
        assert_eq!(PLAYER_RUN_FRAMES, 4);
        assert_eq!(PLAYER_ATTACK_FRAMES, 4);
    }

    #[test]
    fn half_extents_match_sprite_size() {
        let half = player_half_extents();
        assert!((half.x - 32.0).abs() < f32::EPSILON);
        assert!((half.y - 56.0).abs() < f32::EPSILON);
    }

    #[test]
    fn strip_and_cell_pixel_helpers_match_constants() {
        assert_eq!(player_cell_pixel_size(), (64, 112));
        assert_eq!(player_strip_pixel_size(), (256, 112));
    }

    #[test]
    fn shipped_combat_strips_match_frame_rect_contract() {
        // Drive real on-disk assets the game loads (not reimplemented layout math).
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/player/combat");
        let (cell_w, cell_h) = player_cell_pixel_size();
        let (strip_w, strip_h) = player_strip_pixel_size();

        for name in [
            "knight_idle_side.png",
            "knight_run_side.png",
            "knight_attack_side.png",
        ] {
            let path = root.join(name);
            assert!(path.is_file(), "missing combat strip {name}");
            let bytes = std::fs::read(&path).expect("read strip");
            let (w, h) = png_dimensions(&bytes).expect("png header");
            assert_eq!((w, h), (strip_w, strip_h), "{name} strip size");

            // Every frame rect from the shipped helper must lie inside the strip.
            for frame in 0..PLAYER_IDLE_FRAMES {
                let rect = player_frame_rect(frame);
                assert!(rect.min.x >= 0.0);
                assert!(rect.min.y >= 0.0);
                assert!(rect.max.x <= w as f32 + f32::EPSILON);
                assert!(rect.max.y <= h as f32 + f32::EPSILON);
                assert!((rect.max.x - rect.min.x - cell_w as f32).abs() < f32::EPSILON);
                assert!((rect.max.y - rect.min.y - cell_h as f32).abs() < f32::EPSILON);
            }
        }
    }

    #[test]
    fn shipped_weapon_overlay_exists_separate_from_body() {
        let weapon = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join(WEAPON_ANIME_SWORD);
        assert!(
            weapon.is_file(),
            "weapon overlay must remain the only sword graphic at {}",
            weapon.display()
        );
        // Body attack strip is a different path — sword is not baked into body load path.
        assert_ne!(
            format!("{PLAYER_COMBAT_ROOT}/knight_attack_side.png"),
            WEAPON_ANIME_SWORD
        );
    }

    #[test]
    fn shipped_homestead_frames_match_cell_size() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/player/non-combat");
        let (cell_w, cell_h) = player_cell_pixel_size();
        for prefix in ["dwarf_m_idle_anim_f", "dwarf_m_run_anim_f"] {
            for i in 0..PLAYER_IDLE_FRAMES {
                let path = root.join(format!("{prefix}{i}.png"));
                assert!(path.is_file(), "missing {prefix}{i}");
                let bytes = std::fs::read(&path).expect("read frame");
                let (w, h) = png_dimensions(&bytes).expect("png header");
                assert_eq!((w, h), (cell_w, cell_h), "{prefix}{i}");
            }
        }
    }

    /// Minimal PNG IHDR reader — exercises real shipped files without extra deps.
    fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
        if bytes.len() < 24 || &bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
            return None;
        }
        // IHDR chunk: length(4) + "IHDR"(4) + width(4) + height(4)
        if &bytes[12..16] != b"IHDR" {
            return None;
        }
        let w = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
        let h = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
        Some((w, h))
    }
}
