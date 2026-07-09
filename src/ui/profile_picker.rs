use bevy::prelude::*;

use crate::audio::AudioSettings;
use crate::combat::SkillBindings;
use crate::core::{
    activate_profile, apply_profile_to_runtime, load_profile, ActiveProfile, DayClock, GameSettings,
    PlayerProfile, ProfileDirty, ToolEnergy, PROFILE_COUNT,
};
use crate::farming::HomesteadHotbar;
use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

#[derive(Clone, Debug)]
pub struct ProfileCardSummary {
    pub name: String,
    pub weapon: String,
    pub materials: u32,
    pub boss_cleared: bool,
}

#[derive(Resource)]
pub struct ProfilePicker {
    pub cards: [ProfileCardSummary; PROFILE_COUNT as usize],
}

impl Default for ProfilePicker {
    fn default() -> Self {
        Self::refresh()
    }
}

impl ProfilePicker {
    pub fn refresh() -> Self {
        let mut cards = Vec::with_capacity(PROFILE_COUNT as usize);
        for index in 0..PROFILE_COUNT {
            let profile = load_profile(index);
            cards.push(ProfileCardSummary {
                name: profile.display_name(index),
                weapon: profile.summary_weapon().to_string(),
                materials: profile.summary_material_count(),
                boss_cleared: profile.summary_boss_cleared(),
            });
        }

        let mut card_array = [
            ProfileCardSummary {
                name: PlayerProfile::default_name(0),
                weapon: "Rusty Sword".to_string(),
                materials: 0,
                boss_cleared: false,
            },
            ProfileCardSummary {
                name: PlayerProfile::default_name(1),
                weapon: "Rusty Sword".to_string(),
                materials: 0,
                boss_cleared: false,
            },
            ProfileCardSummary {
                name: PlayerProfile::default_name(2),
                weapon: "Rusty Sword".to_string(),
                materials: 0,
                boss_cleared: false,
            },
        ];
        for (slot, summary) in card_array.iter_mut().zip(cards.into_iter()) {
            *slot = summary;
        }

        Self { cards: card_array }
    }
}

pub fn refresh_profile_picker(mut picker: ResMut<ProfilePicker>) {
    *picker = ProfilePicker::refresh();
}

pub fn begin_profile_run(
    index: u8,
    inventory: &mut Inventory,
    loadout: &mut Loadout,
    progress: &mut WorldProgress,
    day_clock: &mut DayClock,
    tool_energy: &mut ToolEnergy,
    hotbar: &mut HomesteadHotbar,
    audio: &mut AudioSettings,
    bindings: &mut SkillBindings,
    active: &mut ActiveProfile,
    profile: &mut PlayerProfile,
    global: &mut GameSettings,
    profile_dirty: &mut ProfileDirty,
) {
    activate_profile(
        index,
        inventory,
        loadout,
        progress,
        day_clock,
        tool_energy,
        hotbar,
        audio,
        bindings,
        active,
        profile,
        global,
        profile_dirty,
    );
    apply_profile_to_runtime(
        profile,
        inventory,
        loadout,
        progress,
        day_clock,
        tool_energy,
        hotbar,
        audio,
        bindings,
    );
}
