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

#[cfg(test)]
mod tests {
    use super::super::crop_save::SavedPlotStage;
    use super::*;

    #[test]
    fn a_new_homestead_save_starts_with_a_full_tool_and_no_crops() {
        let homestead = HomesteadSave::default();
        assert_eq!(homestead.tool_energy, TOOL_ENERGY_MAX);
        assert!(homestead.crop_plots.is_empty());
    }
    #[test]
    fn a_homestead_save_keeps_one_tilled_plot() {
        let homestead: HomesteadSave =
            ron::from_str("(tool_energy:40.0,crop_plots:[(tile_x:4,tile_y:2,stage:Tilled)])")
                .unwrap();
        assert_eq!(homestead.tool_energy, 40.0);
        assert_eq!(homestead.crop_plots.len(), 1);
        assert_eq!(homestead.crop_plots[0].tile_x, 4);
        assert_eq!(homestead.crop_plots[0].stage, SavedPlotStage::Tilled);
    }

    #[test]
    fn a_homestead_save_rejects_a_missing_tool_energy() {
        let error = ron::from_str::<HomesteadSave>("(crop_plots:[])").unwrap_err();
        assert!(error.to_string().contains("tool_energy"));
    }

    #[test]
    fn a_new_mine_save_starts_with_no_depleted_nodes() {
        assert!(MineSave::default().depleted_ore_node_ids.is_empty());
    }

    #[test]
    fn a_mine_save_keeps_one_depleted_node() {
        let mine: MineSave = ron::from_str("(depleted_ore_node_ids:[4])").unwrap();
        assert_eq!(mine.depleted_ore_node_ids, vec![4]);
    }

    #[test]
    fn a_mine_save_rejects_a_missing_node_list() {
        let error = ron::from_str::<MineSave>("()").unwrap_err();
        assert!(error.to_string().contains("depleted_ore_node_ids"));
    }
}
