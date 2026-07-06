use bevy::prelude::*;

pub const TREE_SHEET: &str = "forest/trees.png";
pub const TREE_CELL_W: f32 = 32.0;
pub const TREE_CELL_H: f32 = 48.0;
pub const TREE_VARIANTS: usize = 8;

#[derive(Resource)]
pub struct ForestArt {
    pub grass: Handle<Image>,
    pub path: Handle<Image>,
    pub grid_line: Handle<Image>,
    pub trees: Handle<Image>,
}

impl ForestArt {
    pub fn load(asset_server: &AssetServer) -> Self {
        Self {
            grass: asset_server.load("dungeon/environment/floor_ground.png"),
            path: asset_server.load("dungeon/environment/floor_platform.png"),
            grid_line: asset_server.load("dungeon/environment/floor_ground.png"),
            trees: asset_server.load(TREE_SHEET),
        }
    }
}

pub fn tree_frame_rect(index: usize) -> Rect {
    let index = index % TREE_VARIANTS;
    let col = index % 4;
    let row = index / 4;
    Rect {
        min: Vec2::new(col as f32 * TREE_CELL_W, row as f32 * TREE_CELL_H),
        max: Vec2::new((col + 1) as f32 * TREE_CELL_W, (row + 1) as f32 * TREE_CELL_H),
    }
}

pub const TREE_TINT: Color = Color::srgb(0.52, 0.92, 0.42);