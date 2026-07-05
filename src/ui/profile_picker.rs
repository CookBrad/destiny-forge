use bevy::prelude::*;

use crate::audio::AudioSettings;
use crate::combat::SkillBindings;
use crate::core::{
    activate_profile, apply_profile_to_runtime, load_profile, ActiveProfile, GameSettings,
    PlayerProfile, ProfileDirty, PROFILE_COUNT,
};
use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

#[derive(Clone, Debug)]
pub struct ProfileCardSummary {
    pub weapon: String,
    pub materials: u32,
    pub boss_cleared: bool,
}

#[derive(Resource)]
pub struct ProfilePicker {
    pub selected: u8,
    pub cards: [ProfileCardSummary; PROFILE_COUNT as usize],
}

impl Default for ProfilePicker {
    fn default() -> Self {
        Self::refresh(0)
    }
}

impl ProfilePicker {
    pub fn new(active: u8) -> Self {
        Self::refresh(active)
    }

    pub fn refresh(selected: u8) -> Self {
        let mut cards = Vec::with_capacity(PROFILE_COUNT as usize);
        for index in 0..PROFILE_COUNT {
            let profile = load_profile(index);
            cards.push(ProfileCardSummary {
                weapon: profile.summary_weapon().to_string(),
                materials: profile.summary_material_count(),
                boss_cleared: profile.summary_boss_cleared(),
            });
        }

        let mut card_array = [
            ProfileCardSummary {
                weapon: "Rusty Sword".to_string(),
                materials: 0,
                boss_cleared: false,
            },
            ProfileCardSummary {
                weapon: "Rusty Sword".to_string(),
                materials: 0,
                boss_cleared: false,
            },
            ProfileCardSummary {
                weapon: "Rusty Sword".to_string(),
                materials: 0,
                boss_cleared: false,
            },
        ];
        for (slot, summary) in card_array.iter_mut().zip(cards.into_iter()) {
            *slot = summary;
        }

        Self {
            selected: selected.min(PROFILE_COUNT - 1),
            cards: card_array,
        }
    }
}

pub fn refresh_profile_picker(active: Res<ActiveProfile>, mut picker: ResMut<ProfilePicker>) {
    *picker = ProfilePicker::refresh(active.index());
}

pub fn select_profile_for_run(
    picker: &ProfilePicker,
    inventory: &mut Inventory,
    loadout: &mut Loadout,
    progress: &mut WorldProgress,
    audio: &mut AudioSettings,
    bindings: &mut SkillBindings,
    active: &mut ActiveProfile,
    profile: &mut PlayerProfile,
    global: &mut GameSettings,
    profile_dirty: &mut ProfileDirty,
) {
    activate_profile(
        picker.selected,
        inventory,
        loadout,
        progress,
        audio,
        bindings,
        active,
        profile,
        global,
        profile_dirty,
    );
    apply_profile_to_runtime(profile, inventory, loadout, progress, audio, bindings);
}