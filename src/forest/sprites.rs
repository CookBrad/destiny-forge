use bevy::prelude::*;

pub const TREE_SHEET: &str = "forest/trees.png";
pub const TREE_CELL_W: f32 = 128.0;
pub const TREE_CELL_H: f32 = 192.0;
pub const TREE_VARIANTS: usize = 8;
pub const TREE_SHEET_COLS: usize = 4;
pub const TREE_SHEET_ROWS: usize = 2;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_frame_rects_match_sheet_grid() {
        let r0 = tree_frame_rect(0);
        assert!((r0.min.x - 0.0).abs() < f32::EPSILON);
        assert!((r0.min.y - 0.0).abs() < f32::EPSILON);
        assert!((r0.max.x - TREE_CELL_W).abs() < f32::EPSILON);
        assert!((r0.max.y - TREE_CELL_H).abs() < f32::EPSILON);

        let r3 = tree_frame_rect(3);
        assert!((r3.min.x - TREE_CELL_W * 3.0).abs() < f32::EPSILON);

        let r4 = tree_frame_rect(4);
        assert!((r4.min.x - 0.0).abs() < f32::EPSILON);
        assert!((r4.min.y - TREE_CELL_H).abs() < f32::EPSILON);

        let r7 = tree_frame_rect(7);
        assert!((r7.max.x - TREE_CELL_W * 4.0).abs() < f32::EPSILON);
        assert!((r7.max.y - TREE_CELL_H * 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tree_sheet_size_contract() {
        assert!((TREE_CELL_W * TREE_SHEET_COLS as f32 - 512.0).abs() < f32::EPSILON);
        assert!((TREE_CELL_H * TREE_SHEET_ROWS as f32 - 384.0).abs() < f32::EPSILON);
        assert_eq!(TREE_VARIANTS, TREE_SHEET_COLS * TREE_SHEET_ROWS);
    }
}