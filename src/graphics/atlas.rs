use bevy::prelude::*;

/// Stardew Valley uses 16×16 world tiles and 16×32 character frames.
pub const TILE_SIZE: u32 = 16;
pub const PLAYER_FRAME_WIDTH: u32 = 16;
pub const PLAYER_FRAME_HEIGHT: u32 = 32;
pub const PLAYER_WALK_FRAMES: usize = 4;
pub const PLAYER_SHEET_COLUMNS: u32 = 4;
pub const PLAYER_SHEET_ROWS: u32 = 4;

#[derive(Resource, Clone)]
pub struct GameSprites {
    pub hub_tiles: Handle<Image>,
    pub hub_tiles_layout: Handle<TextureAtlasLayout>,
    pub dungeon_sheet: Handle<Image>,
    pub dungeon_layout: Handle<TextureAtlasLayout>,
    pub player: Handle<Image>,
    pub player_layout: Handle<TextureAtlasLayout>,
    pub forge_building: Handle<Image>,
    pub mine_entrance: Handle<Image>,
    pub hub_background: Handle<Image>,
    pub dungeon_background: Handle<Image>,
}

#[derive(Clone, Copy, Debug)]
pub enum HubTile {
    GrassA = 0,
    GrassB = 1,
    GrassC = 2,
    GrassFlowers = 3,
    DirtA = 4,
    DirtB = 5,
    Bush = 6,
    Rock = 7,
}

impl HubTile {
    pub const fn atlas_index(self) -> usize {
        self as usize
    }

    pub fn grass_variant(world_x: i32, world_y: i32) -> Self {
        match (world_x + world_y).rem_euclid(4) {
            0 => Self::GrassA,
            1 => Self::GrassB,
            2 => Self::GrassC,
            _ => Self::GrassFlowers,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DungeonSprite {
    CaveFloorA = 0,
    CaveFloorB = 1,
    StonePlatform = 2,
    Slime = 3,
    Bat = 4,
    Corpse = 5,
    LadderExit = 6,
    Slash = 7,
    Torch = 8,
}

impl DungeonSprite {
    pub const fn atlas_index(self) -> usize {
        self as usize
    }

    pub fn cave_floor_variant(tile_x: i32) -> Self {
        if tile_x.rem_euclid(2) == 0 {
            Self::CaveFloorA
        } else {
            Self::CaveFloorB
        }
    }
}

/// Stardew farmer layout: 4 walk frames × 4 directions, each frame 16×32.
/// Row 0 down, row 1 right, row 2 up, row 3 left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PlayerSprite {
    Down0 = 0,
    Down1 = 1,
    Down2 = 2,
    Down3 = 3,
    Right0 = 4,
    Right1 = 5,
    Right2 = 6,
    Right3 = 7,
    Up0 = 8,
    Up1 = 9,
    Up2 = 10,
    Up3 = 11,
    Left0 = 12,
    Left1 = 13,
    Left2 = 14,
    Left3 = 15,
}

impl PlayerSprite {
    pub const fn atlas_index(self) -> usize {
        self as usize
    }

    pub const DOWN_WALK: [PlayerSprite; 4] = [
        Self::Down0,
        Self::Down1,
        Self::Down2,
        Self::Down3,
    ];

    pub const RIGHT_WALK: [PlayerSprite; 4] = [
        Self::Right0,
        Self::Right1,
        Self::Right2,
        Self::Right3,
    ];

    pub const UP_WALK: [PlayerSprite; 4] = [
        Self::Up0,
        Self::Up1,
        Self::Up2,
        Self::Up3,
    ];

    pub const LEFT_WALK: [PlayerSprite; 4] = [
        Self::Left0,
        Self::Left1,
        Self::Left2,
        Self::Left3,
    ];

    pub const DOWN_IDLE: [usize; 1] = [Self::Down0.atlas_index()];
    pub const RIGHT_IDLE: [usize; 1] = [Self::Right0.atlas_index()];
    pub const LEFT_IDLE: [usize; 1] = [Self::Left0.atlas_index()];

    pub const DOWN_WALK_INDICES: [usize; 4] = [
        Self::Down0.atlas_index(),
        Self::Down1.atlas_index(),
        Self::Down2.atlas_index(),
        Self::Down3.atlas_index(),
    ];

    pub const RIGHT_WALK_INDICES: [usize; 4] = [
        Self::Right0.atlas_index(),
        Self::Right1.atlas_index(),
        Self::Right2.atlas_index(),
        Self::Right3.atlas_index(),
    ];

    pub const UP_WALK_INDICES: [usize; 4] = [
        Self::Up0.atlas_index(),
        Self::Up1.atlas_index(),
        Self::Up2.atlas_index(),
        Self::Up3.atlas_index(),
    ];

    pub const LEFT_WALK_INDICES: [usize; 4] = [
        Self::Left0.atlas_index(),
        Self::Left1.atlas_index(),
        Self::Left2.atlas_index(),
        Self::Left3.atlas_index(),
    ];

    pub fn idle_for_facing(facing: HubFacing) -> Self {
        match facing {
            HubFacing::Down => Self::Down0,
            HubFacing::Right => Self::Right0,
            HubFacing::Up => Self::Up0,
            HubFacing::Left => Self::Left0,
        }
    }

    pub fn walk_indices_for_facing(facing: HubFacing) -> &'static [usize; 4] {
        match facing {
            HubFacing::Down => &Self::DOWN_WALK_INDICES,
            HubFacing::Right => &Self::RIGHT_WALK_INDICES,
            HubFacing::Up => &Self::UP_WALK_INDICES,
            HubFacing::Left => &Self::LEFT_WALK_INDICES,
        }
    }

    pub fn dungeon_idle(facing: crate::player::Facing) -> Self {
        match facing {
            crate::player::Facing::Right => Self::Right0,
            crate::player::Facing::Left => Self::Left0,
        }
    }

    pub fn dungeon_walk_indices(facing: crate::player::Facing) -> &'static [usize; 4] {
        match facing {
            crate::player::Facing::Right => &Self::RIGHT_WALK_INDICES,
            crate::player::Facing::Left => &Self::LEFT_WALK_INDICES,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HubFacing {
    #[default]
    Down,
    Up,
    Left,
    Right,
}