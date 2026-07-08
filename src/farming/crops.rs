//! Crop kinds and plot state — pure logic for tests.

use serde::{Deserialize, Serialize};

use crate::items::MaterialId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CropKind {
    Turnip,
    Potato,
}

impl CropKind {
    pub fn seed_material(self) -> MaterialId {
        match self {
            Self::Turnip => MaterialId::TurnipSeed,
            Self::Potato => MaterialId::PotatoSeed,
        }
    }

    pub fn harvest_material(self) -> MaterialId {
        match self {
            Self::Turnip => MaterialId::Turnip,
            Self::Potato => MaterialId::Potato,
        }
    }

    pub fn days_to_mature(self) -> u8 {
        match self {
            Self::Turnip => 2,
            Self::Potato => 3,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Turnip => "Turnip",
            Self::Potato => "Potato",
        }
    }

    pub fn from_seed(material: MaterialId) -> Option<Self> {
        match material {
            MaterialId::TurnipSeed => Some(Self::Turnip),
            MaterialId::PotatoSeed => Some(Self::Potato),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotStage {
    /// Untilled soil.
    Soil,
    /// Ready for seeds.
    Tilled,
    /// Growing crop (needs water each day to advance on sleep).
    Growing {
        crop: CropKind,
        days: u8,
        watered: bool,
    },
    /// Ready to harvest.
    Ready { crop: CropKind },
}

impl Default for PlotStage {
    fn default() -> Self {
        Self::Soil
    }
}

impl PlotStage {
    pub fn display_hint(self) -> &'static str {
        match self {
            Self::Soil => "untilled",
            Self::Tilled => "tilled — plant seeds",
            Self::Growing { watered: true, .. } => "growing (watered)",
            Self::Growing { watered: false, .. } => "growing — needs water",
            Self::Ready { .. } => "ready to harvest",
        }
    }
}

/// Result of applying a farm action to a plot (pure).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FarmActionResult {
    Tilled,
    Planted(CropKind),
    Watered,
    Harvested { crop: CropKind, amount: u32 },
    Failed(&'static str),
}

pub fn till_plot(stage: PlotStage) -> (PlotStage, FarmActionResult) {
    match stage {
        PlotStage::Soil => (PlotStage::Tilled, FarmActionResult::Tilled),
        _ => (stage, FarmActionResult::Failed("already worked soil")),
    }
}

pub fn plant_plot(stage: PlotStage, crop: CropKind) -> (PlotStage, FarmActionResult) {
    match stage {
        PlotStage::Tilled => (
            PlotStage::Growing {
                crop,
                days: 0,
                watered: false,
            },
            FarmActionResult::Planted(crop),
        ),
        _ => (stage, FarmActionResult::Failed("till before planting")),
    }
}

pub fn water_plot(stage: PlotStage) -> (PlotStage, FarmActionResult) {
    match stage {
        PlotStage::Growing {
            crop,
            days,
            watered: false,
        } => (
            PlotStage::Growing {
                crop,
                days,
                watered: true,
            },
            FarmActionResult::Watered,
        ),
        PlotStage::Growing {
            watered: true, ..
        } => (stage, FarmActionResult::Failed("already watered today")),
        _ => (stage, FarmActionResult::Failed("nothing to water")),
    }
}

pub fn harvest_plot(stage: PlotStage) -> (PlotStage, FarmActionResult) {
    match stage {
        PlotStage::Ready { crop } => (
            PlotStage::Soil,
            FarmActionResult::Harvested { crop, amount: 1 },
        ),
        _ => (stage, FarmActionResult::Failed("not ready to harvest")),
    }
}

/// Night growth tick: watered crops gain a day; mature becomes ready; water resets.
pub fn advance_plot_day(stage: PlotStage) -> PlotStage {
    match stage {
        PlotStage::Growing {
            crop,
            days,
            watered: true,
        } => {
            let next_days = days.saturating_add(1);
            if next_days >= crop.days_to_mature() {
                PlotStage::Ready { crop }
            } else {
                PlotStage::Growing {
                    crop,
                    days: next_days,
                    watered: false,
                }
            }
        }
        PlotStage::Growing {
            crop,
            days,
            watered: false,
        } => PlotStage::Growing {
            crop,
            days,
            watered: false,
        },
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_grow_cycle_turnip() {
        let (s, r) = till_plot(PlotStage::Soil);
        assert_eq!(r, FarmActionResult::Tilled);
        let (s, r) = plant_plot(s, CropKind::Turnip);
        assert!(matches!(r, FarmActionResult::Planted(CropKind::Turnip)));
        let (s, r) = water_plot(s);
        assert_eq!(r, FarmActionResult::Watered);
        let s = advance_plot_day(s);
        assert!(matches!(
            s,
            PlotStage::Growing {
                days: 1,
                watered: false,
                ..
            }
        ));
        let (s, _) = water_plot(s);
        let s = advance_plot_day(s);
        assert_eq!(s, PlotStage::Ready { crop: CropKind::Turnip });
        let (s, r) = harvest_plot(s);
        assert_eq!(s, PlotStage::Soil);
        assert_eq!(
            r,
            FarmActionResult::Harvested {
                crop: CropKind::Turnip,
                amount: 1
            }
        );
    }

    #[test]
    fn unwatered_crop_does_not_grow() {
        let (s, _) = till_plot(PlotStage::Soil);
        let (s, _) = plant_plot(s, CropKind::Potato);
        let s = advance_plot_day(s);
        assert!(matches!(
            s,
            PlotStage::Growing {
                days: 0,
                watered: false,
                ..
            }
        ));
    }
}
