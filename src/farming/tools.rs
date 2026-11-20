use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::items::MaterialId;

use super::crops::CropKind;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HomesteadTool {
    #[default]
    Hoe,
    WateringCan,
    Seeds,
    Hand,
}

impl HomesteadTool {
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
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EquippedTool(pub HomesteadTool);

pub fn first_available_seed_crop(has: impl Fn(MaterialId) -> bool) -> Option<CropKind> {
    if has(MaterialId::TurnipSeed) {
        Some(CropKind::Turnip)
    } else if has(MaterialId::PotatoSeed) {
        Some(CropKind::Potato)
    } else {
        None
    }
}
