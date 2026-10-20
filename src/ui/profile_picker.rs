use bevy::prelude::*;

use crate::core::{
    activate_profile, apply_profile_to_runtime, load_profile_for_slot, CalendarRules,
    PlayerProfile, ProfileAutosaveBlock, ProfileNeedsAttention, ProfileSnapshot, ProfileSwitch,
    RuntimeProfile, PROFILE_COUNT,
};

pub const SAVE_NEEDS_ATTENTION: &str = "Your save needs attention";

#[derive(Clone, Debug)]
pub struct ProfileCardSummary {
    pub name: String,
    pub weapon: String,
    pub materials: u32,
    pub boss_cleared: bool,
    pub needs_attention: bool,
}

#[derive(Resource)]
pub struct ProfilePicker {
    pub cards: [ProfileCardSummary; PROFILE_COUNT as usize],
}

impl Default for ProfilePicker {
    fn default() -> Self {
        Self {
            cards: std::array::from_fn(|slot| {
                summary_from_profile(slot as u8, &PlayerProfile::default())
            }),
        }
    }
}

impl ProfilePicker {
    pub fn refresh(autosave_block: &mut ProfileAutosaveBlock) -> Self {
        Self {
            cards: std::array::from_fn(|slot| summary_for_slot(slot as u8, autosave_block)),
        }
    }
}

fn summary_for_slot(index: u8, autosave_block: &mut ProfileAutosaveBlock) -> ProfileCardSummary {
    match load_profile_for_slot(index, autosave_block) {
        Ok(profile) => summary_from_profile(index, &profile),
        Err(ProfileNeedsAttention) => ProfileCardSummary {
            name: PlayerProfile::default_name(index),
            weapon: String::new(),
            materials: 0,
            boss_cleared: false,
            needs_attention: true,
        },
    }
}

fn summary_from_profile(index: u8, profile: &PlayerProfile) -> ProfileCardSummary {
    ProfileCardSummary {
        name: profile.display_name(index),
        weapon: profile.summary_weapon().to_string(),
        materials: profile.summary_material_count(),
        boss_cleared: profile.summary_boss_cleared(),
        needs_attention: false,
    }
}

pub fn refresh_profile_picker(
    mut picker: ResMut<ProfilePicker>,
    mut autosave_block: ResMut<ProfileAutosaveBlock>,
) {
    *picker = ProfilePicker::refresh(&mut autosave_block);
}

pub fn begin_profile_run(
    index: u8,
    runtime: &mut RuntimeProfile<'_>,
    rules: &CalendarRules,
    switch: &mut ProfileSwitch<'_>,
) -> bool {
    if switch.autosave_block.is_blocked(index) {
        return false;
    }
    let switched = {
        let source = ProfileSnapshot {
            inventory: &*runtime.inventory,
            loadout: &*runtime.loadout,
            progress: &*runtime.progress,
            day_clock: &*runtime.day_clock,
            tool_energy: &*runtime.tool_energy,
            audio: &*runtime.audio,
            bindings: &*runtime.bindings,
        };
        activate_profile(index, &source, switch)
    };
    if !switched {
        return false;
    }
    apply_profile_to_runtime(switch.profile, runtime, rules);
    true
}
