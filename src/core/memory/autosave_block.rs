use bevy::prelude::*;

use super::profile::{PlayerProfile, PROFILE_COUNT};
use super::storage::{load_profile, ProfileLoadError};

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct ProfileAutosaveBlock {
    blocked: [bool; PROFILE_COUNT as usize],
}

impl ProfileAutosaveBlock {
    pub fn block(&mut self, index: u8) {
        self.blocked[slot(index)] = true;
    }

    pub fn is_blocked(&self, index: u8) -> bool {
        self.blocked[slot(index)]
    }
}

pub fn load_profile_for_slot(
    index: u8,
    autosave_block: &mut ProfileAutosaveBlock,
) -> Result<PlayerProfile, ProfileNeedsAttention> {
    match load_profile(index) {
        Ok(profile) => Ok(profile),
        Err(ProfileLoadError::Missing) => Ok(PlayerProfile::default()),
        Err(error) => {
            warn!("Profile save needs attention: {}", error.detail());
            autosave_block.block(index);
            Err(ProfileNeedsAttention)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProfileNeedsAttention;

fn slot(index: u8) -> usize {
    usize::from(index.min(PROFILE_COUNT - 1))
}
