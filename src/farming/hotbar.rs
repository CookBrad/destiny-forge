//! Homestead quickbar: 5 assignable slots (tools + items).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::items::MaterialId;

use super::crops::CropKind;
use super::tools::HomesteadTool;

pub const HOTBAR_SLOT_COUNT: usize = 5;

/// What can sit in a homestead hotbar slot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HotbarEntry {
    #[default]
    Empty,
    Tool(HomesteadTool),
    /// Inventory material shortcut (e.g. specific seed).
    Item(MaterialId),
}

impl HotbarEntry {
    pub fn label(self) -> String {
        match self {
            Self::Empty => "Empty".to_string(),
            Self::Tool(tool) => tool.label().to_string(),
            Self::Item(material) => material.display_name().to_string(),
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Empty => "—",
            Self::Tool(HomesteadTool::Hoe) => "Hoe",
            Self::Tool(HomesteadTool::WateringCan) => "Water",
            Self::Tool(HomesteadTool::Seeds) => "Seed",
            Self::Tool(HomesteadTool::Hand) => "Hand",
            Self::Item(MaterialId::TurnipSeed) => "T.Sd",
            Self::Item(MaterialId::PotatoSeed) => "P.Sd",
            Self::Item(MaterialId::Turnip) => "Trnp",
            Self::Item(MaterialId::Potato) => "Pota",
            Self::Item(_) => "Item",
        }
    }

    pub fn icon_color(self) -> Color {
        match self {
            Self::Empty => Color::srgb(0.12, 0.12, 0.14),
            Self::Tool(HomesteadTool::Hoe) => Color::srgb(0.55, 0.4, 0.22),
            Self::Tool(HomesteadTool::WateringCan) => Color::srgb(0.28, 0.48, 0.72),
            Self::Tool(HomesteadTool::Seeds) => Color::srgb(0.55, 0.62, 0.28),
            Self::Tool(HomesteadTool::Hand) => Color::srgb(0.72, 0.58, 0.45),
            Self::Item(MaterialId::TurnipSeed) => Color::srgb(0.45, 0.55, 0.28),
            Self::Item(MaterialId::PotatoSeed) => Color::srgb(0.55, 0.42, 0.22),
            Self::Item(MaterialId::Turnip) => Color::srgb(0.72, 0.55, 0.78),
            Self::Item(MaterialId::Potato) => Color::srgb(0.78, 0.68, 0.42),
            Self::Item(_) => Color::srgb(0.4, 0.4, 0.45),
        }
    }

    pub fn energy_cost(self) -> f32 {
        match self {
            Self::Empty => 0.0,
            Self::Tool(tool) => tool.energy_cost(),
            Self::Item(m) if m.is_seed() => 1.0,
            Self::Item(_) => 0.0,
        }
    }
}

/// Bottom quickbar for overworld farming / tools.
#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct HomesteadHotbar {
    pub slots: [HotbarEntry; HOTBAR_SLOT_COUNT],
    /// Selected slot index 0..HOTBAR_SLOT_COUNT.
    pub selected: usize,
}

impl Default for HomesteadHotbar {
    fn default() -> Self {
        Self {
            slots: [
                HotbarEntry::Tool(HomesteadTool::Hoe),
                HotbarEntry::Tool(HomesteadTool::WateringCan),
                HotbarEntry::Item(MaterialId::TurnipSeed),
                HotbarEntry::Item(MaterialId::PotatoSeed),
                HotbarEntry::Tool(HomesteadTool::Hand),
            ],
            selected: 0,
        }
    }
}

impl HomesteadHotbar {
    pub fn selected_entry(&self) -> HotbarEntry {
        self.slots[self.selected.min(HOTBAR_SLOT_COUNT - 1)]
    }

    pub fn select(&mut self, index: usize) {
        if index < HOTBAR_SLOT_COUNT {
            self.selected = index;
        }
    }

    /// Crop to plant for the selected entry, if any.
    pub fn plant_crop_for_selected(
        &self,
        has_material: impl Fn(MaterialId) -> bool,
    ) -> Option<CropKind> {
        match self.selected_entry() {
            HotbarEntry::Tool(HomesteadTool::Seeds) => {
                super::tools::first_available_seed_crop(has_material)
            }
            HotbarEntry::Item(material) => CropKind::from_seed(material).filter(|_| has_material(material)),
            _ => None,
        }
    }
}
