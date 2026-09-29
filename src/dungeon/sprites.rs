use bevy::prelude::*;

use super::hunter_pose::HunterSheet;

pub const ENV_ROOT: &str = "dungeon/environment";
pub const VFX_ROOT: &str = "dungeon/vfx";
pub const ENEMY_ROOT: &str = "dungeon/enemies";
pub const PROJECTILE_ROOT: &str = "dungeon/projectiles";

pub const PLAYER_COMBAT_ROOT: &str = "player/combat";

/// Narrow axis of the sword swing AABB, in pixels.
/// The blade is painted on the hunter. This is not a sprite file.
pub const SWORD_SPRITE_WIDTH: f32 = 12.0;

/// Native pixel size of each hunter frame (width × height).
/// Uniform cell is the hit frame width so the attack strip can hold chamber + thrust.
/// Hurtbox is the body; reach is the blade.
pub const PLAYER_SPRITE_WIDTH: f32 = crate::graphics::HUNTER_CELL_PX.x;
pub const PLAYER_SPRITE_HEIGHT: f32 = crate::graphics::HUNTER_CELL_PX.y;

#[derive(Resource)]
pub struct DungeonArt {
    pub player_idle: Handle<Image>,
    pub player_run: Handle<Image>,
    pub player_attack: Handle<Image>,
    pub player_jump: Handle<Image>,
    pub player_fall: Handle<Image>,
    pub player_death: Handle<Image>,
    pub floor_ground: Handle<Image>,
    pub floor_platform: Handle<Image>,
    pub floor_ladder: Handle<Image>,
    pub wall: Handle<Image>,
    pub floor_pit: Handle<Image>,
    pub pit_stake: Handle<Image>,
    pub pit_lip: Handle<Image>,
    pub ground_slam: Handle<Image>,
    pub slime: Handle<Image>,
    pub bat: Handle<Image>,
    pub goblin: Handle<Image>,
    pub skeleton: Handle<Image>,
    pub zombie: Handle<Image>,
    pub arrow: Handle<Image>,
    /// 1×1 white pixel for solid fills (king slime body until its art exists). Not resampled art.
    pub fill: Handle<Image>,
}

impl DungeonArt {
    pub fn load(asset_server: &AssetServer, images: &mut Assets<Image>) -> Self {
        Self {
            player_idle: asset_server.load(format!("{PLAYER_COMBAT_ROOT}/knight_idle_side.png")),
            player_run: asset_server.load(format!("{PLAYER_COMBAT_ROOT}/knight_run_side.png")),
            player_attack: asset_server
                .load(format!("{PLAYER_COMBAT_ROOT}/knight_attack_side.png")),
            player_jump: asset_server.load(format!("{PLAYER_COMBAT_ROOT}/knight_jump_side.png")),
            player_fall: asset_server.load(format!("{PLAYER_COMBAT_ROOT}/knight_fall_side.png")),
            player_death: asset_server
                .load(format!("{PLAYER_COMBAT_ROOT}/knight_death_side.png")),
            floor_ground: asset_server.load(format!("{ENV_ROOT}/floor_ground.png")),
            floor_platform: asset_server.load(format!("{ENV_ROOT}/floor_platform.png")),
            floor_ladder: asset_server.load(format!("{ENV_ROOT}/floor_ladder.png")),
            wall: asset_server.load(format!("{ENV_ROOT}/wall.png")),
            floor_pit: asset_server.load(format!("{ENV_ROOT}/floor_pit.png")),
            pit_stake: asset_server.load(format!("{ENV_ROOT}/pit_stake.png")),
            pit_lip: asset_server.load(format!("{ENV_ROOT}/pit_lip.png")),
            ground_slam: asset_server.load(format!("{VFX_ROOT}/ground_slam.png")),
            slime: asset_server.load(format!("{ENEMY_ROOT}/slime.png")),
            bat: asset_server.load(format!("{ENEMY_ROOT}/bat.png")),
            goblin: asset_server.load(format!("{ENEMY_ROOT}/goblin.png")),
            skeleton: asset_server.load(format!("{ENEMY_ROOT}/skeleton.png")),
            zombie: asset_server.load(format!("{ENEMY_ROOT}/zombie.png")),
            arrow: asset_server.load(format!("{PROJECTILE_ROOT}/arrow.png")),
            fill: crate::graphics::solid_white_pixel(images),
        }
    }

    pub fn hunter_image(&self, sheet: HunterSheet) -> Handle<Image> {
        match sheet {
            HunterSheet::Idle => self.player_idle.clone(),
            HunterSheet::Run => self.player_run.clone(),
            HunterSheet::Attack => self.player_attack.clone(),
            HunterSheet::Jump => self.player_jump.clone(),
            HunterSheet::Fall => self.player_fall.clone(),
            HunterSheet::Death => self.player_death.clone(),
        }
    }
}

/// Collision and feet use the GDD body, not the 343-wide cell.
pub fn player_half_extents() -> Vec2 {
    crate::graphics::HUNTER_BODY_PX * 0.5
}

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
    fn player_half_extents_are_the_body_not_the_cell() {
        let half = player_half_extents();
        assert_eq!(half, crate::graphics::HUNTER_BODY_PX * 0.5);
        assert_ne!(
            half,
            Vec2::new(PLAYER_SPRITE_WIDTH, PLAYER_SPRITE_HEIGHT) * 0.5
        );
    }
}
