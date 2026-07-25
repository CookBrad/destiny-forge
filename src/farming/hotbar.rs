//! Homestead quickbar: 5 empty slots filled by dragging inventory items.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::items::MaterialId;

pub const HOTBAR_SLOT_COUNT: usize = 5;

/// Inventory item assigned to a hotbar slot (or empty).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HotbarEntry {
    #[default]
    Empty,
    Item(MaterialId),
}

impl HotbarEntry {
    pub fn material(self) -> Option<MaterialId> {
        match self {
            Self::Empty => None,
            Self::Item(m) => Some(m),
        }
    }

    pub fn label(self) -> String {
        match self {
            Self::Empty => "Empty".to_string(),
            Self::Item(material) => material.display_name().to_string(),
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Empty => "—",
            Self::Item(material) => material.short_label(),
        }
    }

    pub fn icon_color(self) -> Color {
        match self {
            Self::Empty => Color::srgb(0.12, 0.12, 0.14),
            Self::Item(MaterialId::Hoe) => Color::srgb(0.55, 0.4, 0.22),
            Self::Item(MaterialId::WateringCan) => Color::srgb(0.28, 0.48, 0.72),
            Self::Item(MaterialId::Pickaxe) => Color::srgb(0.5, 0.52, 0.55),
            Self::Item(MaterialId::FishingRod) => Color::srgb(0.35, 0.45, 0.55),
            Self::Item(MaterialId::TurnipSeed) => Color::srgb(0.45, 0.55, 0.28),
            Self::Item(MaterialId::PotatoSeed) => Color::srgb(0.55, 0.42, 0.22),
            Self::Item(MaterialId::Turnip) => Color::srgb(0.72, 0.55, 0.78),
            Self::Item(MaterialId::Potato) => Color::srgb(0.78, 0.68, 0.42),
            Self::Item(MaterialId::IronOre) => Color::srgb(0.48, 0.5, 0.55),
            Self::Item(MaterialId::RiverFish) => Color::srgb(0.35, 0.55, 0.7),
            Self::Item(MaterialId::HeartyStew) => Color::srgb(0.7, 0.45, 0.25),
            Self::Item(MaterialId::SpicySashimi) => Color::srgb(0.85, 0.4, 0.35),
            Self::Item(_) => Color::srgb(0.4, 0.4, 0.45),
        }
    }

    pub fn energy_cost(self) -> f32 {
        match self {
            Self::Empty => 0.0,
            Self::Item(m) => m.energy_cost(),
        }
    }
}

/// Bottom quickbar for overworld. Starts empty; player assigns via drag.
#[derive(Resource, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomesteadHotbar {
    pub slots: [HotbarEntry; HOTBAR_SLOT_COUNT],
    /// Selected slot (highlighted) — this is the active action.
    #[serde(default)]
    pub selected: usize,
}

impl Default for HomesteadHotbar {
    fn default() -> Self {
        Self {
            slots: [HotbarEntry::Empty; HOTBAR_SLOT_COUNT],
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

    pub fn assign(&mut self, index: usize, material: MaterialId) {
        if index < HOTBAR_SLOT_COUNT {
            self.slots[index] = HotbarEntry::Item(material);
            self.selected = index;
        }
    }

    pub fn clear_slot(&mut self, index: usize) {
        if index < HOTBAR_SLOT_COUNT {
            self.slots[index] = HotbarEntry::Empty;
        }
    }
}
