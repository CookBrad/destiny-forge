use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

use super::super::day_cycle::DayPhase;

use super::crop_save::SavedCropPlot;
use super::settings::ProfileSettings;

pub const PROFILE_COUNT: u8 = 3;
/// v6 = crop_plots (Homestead #72). Gear stash (#58) lives on Loadout with
/// #[serde(default)] and does not bump this.
/// v7 = 32 inventory slots (#22) and Loadout.food_buff (#21). Older saves keep
/// their 24 slots padded with empties; see `Inventory` slot deserialization.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{MaterialId, INVENTORY_SLOT_COUNT};

    const V6_PROFILE: &str = include_str!("fixtures/profile_v6_24_slots.ron");

    #[test]
    fn v6_profile_with_24_slots_loads_without_losing_items() {
        let profile: PlayerProfile = ron::from_str(V6_PROFILE).unwrap();
        let profile = profile.migrate();

        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.name, "Old Save");
        assert_eq!(profile.calendar_day, 4);
        assert_eq!(profile.inventory.slots.len(), INVENTORY_SLOT_COUNT);
        assert_eq!(profile.inventory.count(MaterialId::Hoe), 1);
        assert_eq!(profile.inventory.count(MaterialId::TurnipSeed), 8);
        assert_eq!(profile.inventory.count(MaterialId::Turnip), 5);
        assert_eq!(profile.inventory.count(MaterialId::Potato), 3);
        assert_eq!(profile.inventory.count(MaterialId::SlimeGel), 120);
        assert_eq!(profile.inventory.slots[6].count, 99);
        assert_eq!(profile.inventory.slots[7].count, 21);
        assert_eq!(
            profile.inventory.slots[23].material,
            Some(MaterialId::IronScrap)
        );
        assert_eq!(profile.inventory.slots[23].count, 2);
        assert!(profile.inventory.slots[24..]
            .iter()
            .all(|slot| slot.material.is_none()));
        assert_eq!(profile.loadout.food_buff, None);
    }

    #[test]
    fn v7_profile_keeps_new_slots_and_food_buff() {
        let mut profile = PlayerProfile::default();
        profile.inventory.slots[INVENTORY_SLOT_COUNT - 1].material = Some(MaterialId::PotatoStew);
        profile.inventory.slots[INVENTORY_SLOT_COUNT - 1].count = 3;
        profile.loadout.food_buff = Some(MaterialId::RoastTurnip);

        let pretty = ron::ser::PrettyConfig::new().depth_limit(4);
        let text = ron::ser::to_string_pretty(&profile, pretty).unwrap();
        let loaded: PlayerProfile = ron::from_str(&text).unwrap();

        assert_eq!(loaded.version, PROFILE_VERSION);
        assert_eq!(loaded.inventory, profile.inventory);
        assert_eq!(loaded.loadout.food_buff, Some(MaterialId::RoastTurnip));
    }
}
