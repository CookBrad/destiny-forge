use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SavedCropKind {
    Turnip,
    Potato,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SavedPlotStage {
    Tilled,
    Growing {
        crop: SavedCropKind,
        days: u8,
        watered: bool,
    },
    Ready {
        crop: SavedCropKind,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedCropPlot {
    pub tile_x: u32,
    pub tile_y: u32,
    pub stage: SavedPlotStage,
}
