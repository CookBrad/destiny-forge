//! Homestead tools and energy costs. Equip with keys 1–4 (not a drag hotbar).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::items::MaterialId;

use super::crops::CropKind;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HomesteadTool {
    #[default]
    Hoe,
    WateringCan,
    /// Plant first available seed type in inventory (turnip, then potato).
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

    pub fn hotkey(self) -> &'static str {
        match self {
            Self::Hoe => "1",
            Self::WateringCan => "2",
            Self::Seeds => "3",
            Self::Hand => "4",
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

    pub fn from_digit_key(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::Digit1 | KeyCode::Numpad1 => Some(Self::Hoe),
            KeyCode::Digit2 | KeyCode::Numpad2 => Some(Self::WateringCan),
            KeyCode::Digit3 | KeyCode::Numpad3 => Some(Self::Seeds),
            KeyCode::Digit4 | KeyCode::Numpad4 => Some(Self::Hand),
            _ => None,
        }
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
