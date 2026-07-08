//! Homestead tools and energy costs.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::items::MaterialId;

use super::crops::CropKind;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HomesteadTool {
    #[default]
    Hoe,
    WateringCan,
    /// Plant first available seed type in inventory.
    Seeds,
    Hand,
}

impl HomesteadTool {
    pub const ALL: [Self; 4] = [Self::Hoe, Self::WateringCan, Self::Seeds, Self::Hand];

    pub fn label(self) -> &'static str {
        match self {
            Self::Hoe => "Hoe",
            Self::WateringCan => "Watering Can",
            Self::Seeds => "Seeds",
            Self::Hand => "Hand",
        }
    }

    pub fn energy_cost(self) -> f32 {
        match self {
            Self::Hoe => 5.0,
            Self::WateringCan => 3.0,
            Self::Seeds => 1.0,
            Self::Hand => 0.0,
        }
    }

    pub fn hotkey_index(self) -> Option<usize> {
        match self {
            Self::Hoe => Some(0),
            Self::WateringCan => Some(1),
            Self::Seeds => Some(2),
            Self::Hand => Some(3),
        }
    }

    pub fn from_hotkey_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }
}

/// Currently equipped homestead tool (overworld only).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EquippedTool(pub HomesteadTool);

/// Prefer turnip seeds, then potato.
pub fn first_available_seed_crop(has: impl Fn(MaterialId) -> bool) -> Option<CropKind> {
    if has(MaterialId::TurnipSeed) {
        Some(CropKind::Turnip)
    } else if has(MaterialId::PotatoSeed) {
        Some(CropKind::Potato)
    } else {
        None
    }
}
