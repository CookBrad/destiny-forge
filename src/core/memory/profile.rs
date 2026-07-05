use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

use super::settings::ProfileSettings;

pub const PROFILE_COUNT: u8 = 3;
pub const PROFILE_VERSION: u32 = 2;

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
    pub inventory: Inventory,
    pub loadout: Loadout,
    pub progress: WorldProgress,
    #[serde(default)]
    pub settings: ProfileSettings,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            version: PROFILE_VERSION,
            inventory: Inventory::default(),
            loadout: Loadout::default(),
            progress: WorldProgress::default(),
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