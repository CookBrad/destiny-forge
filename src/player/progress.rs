use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::dungeon::DungeonProgress;

#[derive(Resource, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldProgress {
    pub boss_defeated_floor_1: bool,
}

impl Default for WorldProgress {
    fn default() -> Self {
        Self {
            boss_defeated_floor_1: false,
        }
    }
}

impl WorldProgress {
    pub fn apply_to_dungeon_progress(&self, progress: &mut DungeonProgress) {
        progress.boss_defeated = self.boss_defeated_floor_1;
    }

    pub fn record_boss_defeated_floor_1(&mut self) {
        self.boss_defeated_floor_1 = true;
    }
}
