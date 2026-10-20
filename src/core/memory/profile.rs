use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

use super::header::SaveHeader;
use super::sections::HomesteadSave;
use super::settings::ProfileSettings;

pub const PROFILE_COUNT: u8 = 3;
pub const PROFILE_VERSION: u32 = 11;
pub const MAX_PROFILE_NAME_LEN: usize = 24;

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
    pub header: SaveHeader,
    pub inventory: Inventory,
    pub loadout: Loadout,
    pub progress: WorldProgress,
    pub homestead: HomesteadSave,
    pub depleted_ore_node_ids: Vec<u16>,
    #[serde(default)]
    pub settings: ProfileSettings,
    #[serde(skip)]
    pub playtime_fraction: f64,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            version: PROFILE_VERSION,
            header: SaveHeader::default(),
            inventory: Inventory::with_starter_kit(),
            loadout: Loadout::default(),
            progress: WorldProgress::default(),
            homestead: HomesteadSave::default(),
            depleted_ore_node_ids: Vec::new(),
            settings: ProfileSettings::default(),
            playtime_fraction: 0.0,
        }
    }
}

impl PlayerProfile {
    pub fn default_name(index: u8) -> String {
        format!("Profile {}", index.saturating_add(1))
    }

    pub fn display_name(&self, index: u8) -> String {
        let trimmed = self.header.name.trim();
        if trimmed.is_empty() {
            Self::default_name(index)
        } else {
            trimmed.to_string()
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.header.name = sanitize_profile_name(name.to_string());
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

pub fn rename_profile_on_disk(index: u8, name: String) -> Option<PlayerProfile> {
    let Ok(mut profile) =
        super::storage::profile_or_missing_default(super::storage::load_profile(index))
    else {
        return None;
    };
    profile.set_name(&name);
    if let Err(error) = super::storage::save_profile(index, &profile) {
        warn!("Failed to save profile name: {error}");
    }
    Some(profile)
}

#[cfg(test)]
mod tests {
    use super::super::super::day_cycle::DayPhase;
    use super::super::header::SaveLocation;
    use super::*;

    #[test]
    fn a_new_profile_starts_with_a_filled_header() {
        let profile = PlayerProfile::default();
        assert_eq!(profile.header.name, "");
        assert_eq!(profile.header.playtime_seconds, 0);
        assert_eq!(profile.header.calendar.calendar_day, 1);
        assert_eq!(profile.header.calendar.day_phase, DayPhase::Morning);
        assert_eq!(profile.header.location, SaveLocation::Homestead);
        assert_eq!(profile.header.gold, 0);
        assert_eq!(profile.display_name(0), "Profile 1");
    }

    #[test]
    fn set_name_writes_the_header() {
        let mut profile = PlayerProfile::default();
        profile.set_name("  Hearth\n");
        assert_eq!(profile.header.name, "Hearth");
        assert_eq!(profile.display_name(0), "Hearth");
    }

    #[test]
    fn set_name_drops_a_name_made_only_of_control_characters() {
        let mut profile = PlayerProfile::default();
        profile.set_name("\n\t");
        assert_eq!(profile.header.name, "");
        assert_eq!(profile.display_name(2), "Profile 3");
    }
}
