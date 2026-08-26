//! Sparse crop-plot save DTOs. Core owns these; farming maps live types to/from them.
//!
//! No `Soil` variant: virgin / harvested-back-to-soil tiles are omitted from
//! `PlayerProfile.crop_plots`. Restore missing `(tile_x, tile_y)` as Soil.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SavedCropKind {
    Turnip,
    Potato,
}

/// Same RON shape as live `PlotStage` minus `Soil`.
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedCropPlot {
    pub tile_x: u32,
    pub tile_y: u32,
    pub stage: SavedPlotStage,
}
