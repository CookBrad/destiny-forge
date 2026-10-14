use serde::{Deserialize, Serialize};

use super::super::day_cycle::TOOL_ENERGY_MAX;
use super::crop_save::SavedCropPlot;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HomesteadSave {
    pub tool_energy: f32,
    pub crop_plots: Vec<SavedCropPlot>,
}

impl Default for HomesteadSave {
    fn default() -> Self {
        Self {
            tool_energy: TOOL_ENERGY_MAX,
            crop_plots: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MineSave {
    pub depleted_ore_node_ids: Vec<u16>,
}
