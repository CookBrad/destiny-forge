//! Homestead crop plots persisted on `PlayerProfile`.
//!
//! Farming maps live `CropKind` / `PlotStage` onto core save DTOs.
//! Soil has no DTO: skip on capture; missing tiles restore as Soil.

use crate::core::{SavedCropKind, SavedCropPlot, SavedPlotStage};

use super::crops::{CropKind, PlotStage};
use super::plots::CropPlot;

pub fn capture_plots<'a>(plots: impl Iterator<Item = &'a CropPlot>) -> Vec<SavedCropPlot> {
    let mut saved: Vec<SavedCropPlot> = plots.filter_map(saved_from_plot).collect();
    saved.sort_by_key(|plot| (plot.tile_y, plot.tile_x));
    saved
}

/// Restore by tile coordinates only. Missing tiles are Soil. Do not use index.
pub fn restored_stage(saved: &[SavedCropPlot], tile_x: u32, tile_y: u32) -> PlotStage {
    saved
        .iter()
        .find(|plot| plot.tile_x == tile_x && plot.tile_y == tile_y)
        .map(|plot| live_stage(plot.stage))
        .unwrap_or(PlotStage::Soil)
}

fn saved_from_plot(plot: &CropPlot) -> Option<SavedCropPlot> {
    saved_stage(plot.stage).map(|stage| SavedCropPlot {
        tile_x: plot.tile_x,
        tile_y: plot.tile_y,
        stage,
    })
}

fn saved_stage(stage: PlotStage) -> Option<SavedPlotStage> {
    match stage {
        PlotStage::Soil => None,
        PlotStage::Tilled => Some(SavedPlotStage::Tilled),
        PlotStage::Growing {
            crop,
            days,
            watered,
        } => Some(SavedPlotStage::Growing {
            crop: saved_kind(crop),
            days,
            watered,
        }),
        PlotStage::Ready { crop } => Some(SavedPlotStage::Ready {
            crop: saved_kind(crop),
        }),
    }
}

fn live_stage(stage: SavedPlotStage) -> PlotStage {
    match stage {
        SavedPlotStage::Tilled => PlotStage::Tilled,
        SavedPlotStage::Growing {
            crop,
            days,
            watered,
        } => PlotStage::Growing {
            crop: live_kind(crop),
            days,
            watered,
        },
        SavedPlotStage::Ready { crop } => PlotStage::Ready {
            crop: live_kind(crop),
        },
    }
}

fn saved_kind(kind: CropKind) -> SavedCropKind {
    match kind {
        CropKind::Turnip => SavedCropKind::Turnip,
        CropKind::Potato => SavedCropKind::Potato,
    }
}

fn live_kind(kind: SavedCropKind) -> CropKind {
    match kind {
        SavedCropKind::Turnip => CropKind::Turnip,
        SavedCropKind::Potato => CropKind::Potato,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::PlayerProfile;

    fn growing_turnip() -> CropPlot {
        CropPlot {
            tile_x: 4,
            tile_y: 7,
            stage: PlotStage::Growing {
                crop: CropKind::Turnip,
                days: 1,
                watered: true,
            },
        }
    }

    fn tilled() -> CropPlot {
        CropPlot {
            tile_x: 5,
            tile_y: 7,
            stage: PlotStage::Tilled,
        }
    }

    fn ready_potato() -> CropPlot {
        CropPlot {
            tile_x: 6,
            tile_y: 7,
            stage: PlotStage::Ready {
                crop: CropKind::Potato,
            },
        }
    }

    fn soil_at(tile_x: u32, tile_y: u32) -> CropPlot {
        CropPlot {
            tile_x,
            tile_y,
            stage: PlotStage::Soil,
        }
    }

    #[test]
    fn save_teardown_reload_restores_plots() {
        let live = vec![growing_turnip(), tilled(), ready_potato()];
        let saved = capture_plots(live.iter());
        assert_eq!(saved.len(), 3);

        let mut profile = PlayerProfile::default();
        profile.crop_plots = saved.clone();
        assert_eq!(profile.version, PlayerProfile::default().version);

        let encoded = ron::ser::to_string(&profile).expect("serialize profile");
        assert!(
            encoded.contains("crop_plots"),
            "profile RON must name crop_plots: {encoded}"
        );
        assert!(
            !encoded.contains("crop_id"),
            "must not flatten crop_id beside stage: {encoded}"
        );

        let loaded: PlayerProfile = ron::from_str(&encoded).expect("deserialize profile");
        assert_eq!(loaded.crop_plots, saved);

        assert_eq!(
            restored_stage(&loaded.crop_plots, 4, 7),
            PlotStage::Growing {
                crop: CropKind::Turnip,
                days: 1,
                watered: true,
            }
        );
        assert_eq!(restored_stage(&loaded.crop_plots, 5, 7), PlotStage::Tilled);
        assert_eq!(
            restored_stage(&loaded.crop_plots, 6, 7),
            PlotStage::Ready {
                crop: CropKind::Potato,
            }
        );
        assert_eq!(restored_stage(&[], 4, 7), PlotStage::Soil);
        assert_eq!(
            restored_stage(&loaded.crop_plots, 99, 99),
            PlotStage::Soil,
            "unknown tiles drop to Soil"
        );
    }

    #[test]
    fn soil_is_not_persisted() {
        let live = vec![soil_at(4, 7), tilled(), soil_at(8, 8)];
        let saved = capture_plots(live.iter());
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].tile_x, 5);
        assert_eq!(saved[0].tile_y, 7);
        assert_eq!(saved[0].stage, SavedPlotStage::Tilled);
        assert_eq!(restored_stage(&saved, 4, 7), PlotStage::Soil);
        assert_eq!(restored_stage(&saved, 8, 8), PlotStage::Soil);
    }

    #[test]
    fn harvest_back_to_soil_drops_entry() {
        let before = vec![
            CropPlot {
                tile_x: 4,
                tile_y: 7,
                stage: PlotStage::Ready {
                    crop: CropKind::Turnip,
                },
            },
            tilled(),
        ];
        let saved = capture_plots(before.iter());
        assert_eq!(saved.len(), 2);

        let after = vec![soil_at(4, 7), tilled()];
        let saved = capture_plots(after.iter());
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].tile_x, 5);
        assert_eq!(saved[0].tile_y, 7);
        assert_eq!(saved[0].stage, SavedPlotStage::Tilled);
        assert_eq!(restored_stage(&saved, 4, 7), PlotStage::Soil);
    }

    #[test]
    fn empty_vec_is_virgin_field() {
        assert!(capture_plots(std::iter::empty()).is_empty());
        assert_eq!(restored_stage(&[], 0, 0), PlotStage::Soil);
        assert_eq!(PlayerProfile::default().crop_plots, Vec::new());
    }
}
