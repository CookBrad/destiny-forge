use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

use super::super::day_cycle::DayPhase;

use super::crop_save::SavedCropPlot;
use super::settings::ProfileSettings;

pub const PROFILE_COUNT: u8 = 3;
/// v6 = crop_plots (Homestead #72). v7 = Loadout.stash persisted (Forge #58).
/// Pre-v7 saves deserialize an empty stash via #[serde(default)] on Loadout.
pub const PROFILE_VERSION: u32 = 7;
pub const MAX_PROFILE_NAME_LEN: usize = 24;

fn default_calendar_day() -> u32 {
    1
}

fn default_tool_energy() -> f32 {
    super::super::day_cycle::TOOL_ENERGY_MAX
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ActiveProfile(pub u8);

impl ActiveProfile {
    pub fn index(self) -> u8 {
        self.0.min(PROFILE_COUNT - 1)
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub version: u32,
    #[serde(default)]
    pub name: String,
    pub inventory: Inventory,
    pub loadout: Loadout,
    pub progress: WorldProgress,
    /// Soft day cycle calendar (persisted).
    #[serde(default = "default_calendar_day")]
    pub calendar_day: u32,
    #[serde(default)]
    pub day_phase: DayPhase,
    /// Homestead tool energy (0..=max). Restored on sleep.
    #[serde(default = "default_tool_energy")]
    pub tool_energy: f32,
    /// Homestead crop plots. Empty vec = virgin field / all Soil (new / pre-v6 saves).
    /// Sparse: only tiles whose stage is not Soil.
    #[serde(default)]
    pub crop_plots: Vec<SavedCropPlot>,
    #[serde(default)]
    pub settings: ProfileSettings,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            version: PROFILE_VERSION,
            name: String::new(),
            inventory: Inventory::with_starter_seeds(),
            loadout: Loadout::default(),
            progress: WorldProgress::default(),
            calendar_day: 1,
            day_phase: DayPhase::Morning,
            tool_energy: default_tool_energy(),
            crop_plots: Vec::new(),
            settings: ProfileSettings::default(),
        }
    }
}

impl PlayerProfile {
    pub fn migrate(mut self) -> Self {
        if self.version < PROFILE_VERSION {
            self.version = PROFILE_VERSION;
        }
        self
    }

    pub fn default_name(index: u8) -> String {
        format!("Profile {}", index.saturating_add(1))
    }

    pub fn display_name(&self, index: u8) -> String {
        let trimmed = self.name.trim();
        if trimmed.is_empty() {
            Self::default_name(index)
        } else {
            trimmed.to_string()
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = sanitize_profile_name(name.to_string());
    }

    pub fn summary_weapon(&self) -> &'static str {
        self.loadout.weapon_label()
    }

    pub fn summary_material_count(&self) -> u32 {
        self.inventory.total_items()
    }

    pub fn summary_boss_cleared(&self) -> bool {
        self.progress.boss_defeated_floor_1
    }
}

pub fn sanitize_profile_name(name: String) -> String {
    name.chars()
        .filter(|ch| !ch.is_control())
        .take(MAX_PROFILE_NAME_LEN)
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn rename_profile_on_disk(index: u8, name: String) -> PlayerProfile {
    let mut profile = super::storage::load_profile(index);
    profile.set_name(&name);
    if let Err(error) = super::storage::save_profile(index, &profile) {
        warn!("Failed to save profile name: {error}");
    }
    profile
}
