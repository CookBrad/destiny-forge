use bevy::prelude::*;

use crate::overworld::sprites::ENV_ROOT;

pub const TREE_SHEET: &str = "forest/trees.png";
pub const TREE_CELL_W: f32 = 32.0;
pub const TREE_CELL_H: f32 = 48.0;
pub const TREE_VARIANTS: usize = 8;

#[derive(Resource)]
pub struct ForestArt {
    pub grass: Handle<Image>,
    pub path: Handle<Image>,
    /// 1×1 white pixel. Grid strokes are solid fills sized in world units, not a stretched tile.
    pub grid_line: Handle<Image>,
    pub trees: Handle<Image>,
}

impl ForestArt {
    pub fn load(asset_server: &AssetServer, images: &mut Assets<Image>) -> Self {
        Self {
            grass: asset_server.load(format!("{ENV_ROOT}/floor_ground.png")),
            path: asset_server.load(format!("{ENV_ROOT}/floor_platform.png")),
            grid_line: crate::graphics::solid_white_pixel(images),
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
        max: Vec2::new(
            (col + 1) as f32 * TREE_CELL_W,
            (row + 1) as f32 * TREE_CELL_H,
        ),
    }
}

pub const TREE_TINT: Color = Color::srgb(0.52, 0.92, 0.42);
