use bevy::prelude::*;

pub const ENV_ROOT: &str = "dungeon/environment";
pub const OVERWORLD_ROOT: &str = "overworld";
pub const PLAYER_NON_COMBAT_ROOT: &str = "player/non-combat";

/// Per-species farm animal walk strips (4 frames × 64px).
pub const ANIMAL_CELL: u32 = 64;
pub const ANIMAL_WALK_FRAMES: u32 = 4;
pub const ANIMAL_SHEET_COLS: u32 = ANIMAL_WALK_FRAMES;
pub const ANIMAL_SHEET_ROWS: u32 = 1;

/// Match the homestead player footprint; shared camera zoom applies uniformly.
pub const ANIMAL_DISPLAY_SIZE: Vec2 = Vec2::new(PLAYER_SPRITE_WIDTH, PLAYER_SPRITE_HEIGHT);

pub const PLAYER_SPRITE_WIDTH: f32 = 64.0;
pub const PLAYER_SPRITE_HEIGHT: f32 = 112.0;
pub const PLAYER_ANIM_FRAMES: usize = 4;

pub const FORGE_FURNACE_HEIGHT: f32 = 160.0;
pub const FORGE_WORKBENCH_HEIGHT: f32 = 320.0;
pub const FORGE_ANVIL_HEIGHT: f32 = 128.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimalKind {
    Cow,
    Goat,
    Sheep,
}

impl AnimalKind {
    pub const ALL: [Self; 3] = [Self::Cow, Self::Goat, Self::Sheep];

    pub fn sheet_path(self) -> &'static str {
        match self {
            Self::Cow => "overworld/animals/cow.png",
            Self::Goat => "overworld/animals/goat.png",
            Self::Sheep => "overworld/animals/sheep.png",
        }
    }

    pub fn from_index(index: usize) -> Self {
        Self::ALL[index % Self::ALL.len()]
    }
}

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
    pub cow: Handle<Image>,
    pub goat: Handle<Image>,
    pub sheep: Handle<Image>,
    /// Shared 4-frame horizontal layout for each animal strip.
    pub animal_layout: Handle<TextureAtlasLayout>,
}

impl OverworldArt {
    pub fn load(asset_server: &AssetServer, layouts: &mut Assets<TextureAtlasLayout>) -> Self {
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
            cow: asset_server.load(AnimalKind::Cow.sheet_path()),
            goat: asset_server.load(AnimalKind::Goat.sheet_path()),
            sheep: asset_server.load(AnimalKind::Sheep.sheet_path()),
            animal_layout,
        }
    }

    pub fn animal_image(&self, kind: AnimalKind) -> Handle<Image> {
        match kind {
            AnimalKind::Cow => self.cow.clone(),
            AnimalKind::Goat => self.goat.clone(),
            AnimalKind::Sheep => self.sheep.clone(),
        }
    }
}

/// Frame index within a single-species 4-frame walk strip.
pub fn animal_frame_index(frame: usize) -> usize {
    frame % ANIMAL_WALK_FRAMES as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animal_species_have_distinct_sheets() {
        let paths: Vec<_> = AnimalKind::ALL.iter().map(|k| k.sheet_path()).collect();
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&"overworld/animals/cow.png"));
        assert!(paths.contains(&"overworld/animals/goat.png"));
        assert!(paths.contains(&"overworld/animals/sheep.png"));
        assert_ne!(AnimalKind::Cow.sheet_path(), AnimalKind::Goat.sheet_path());
        assert_ne!(AnimalKind::Goat.sheet_path(), AnimalKind::Sheep.sheet_path());
    }

    #[test]
    fn animal_sheet_contract_four_walk_frames() {
        assert_eq!(ANIMAL_CELL, 64);
        assert_eq!(ANIMAL_WALK_FRAMES, 4);
        assert_eq!(ANIMAL_SHEET_COLS, 4);
        assert_eq!(ANIMAL_SHEET_ROWS, 1);
        assert_eq!(ANIMAL_CELL * ANIMAL_SHEET_COLS, 256);
        assert_eq!(ANIMAL_CELL * ANIMAL_SHEET_ROWS, 64);
    }

    #[test]
    fn animal_frame_index_wraps() {
        assert_eq!(animal_frame_index(0), 0);
        assert_eq!(animal_frame_index(3), 3);
        assert_eq!(animal_frame_index(4), 0);
        assert_eq!(animal_frame_index(5), 1);
    }

    #[test]
    fn animal_kind_cycles_from_index() {
        assert_eq!(AnimalKind::from_index(0), AnimalKind::Cow);
        assert_eq!(AnimalKind::from_index(1), AnimalKind::Goat);
        assert_eq!(AnimalKind::from_index(2), AnimalKind::Sheep);
        assert_eq!(AnimalKind::from_index(3), AnimalKind::Cow);
    }

    #[test]
    fn homestead_player_matches_combat_height_ratio() {
        assert!((PLAYER_SPRITE_WIDTH - 64.0).abs() < f32::EPSILON);
        assert!((PLAYER_SPRITE_HEIGHT - 112.0).abs() < f32::EPSILON);
    }
}
