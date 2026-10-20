use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::audio::AudioSettings;
use crate::combat::SkillBindings;
use crate::core::{
    activate_profile, apply_profile_to_runtime, load_profile_for_slot, ActiveProfile,
    CalendarRules, DayClock, GameSettings, PlayerProfile, ProfileApply, ProfileAutosaveBlock,
    ProfileCapture, ProfileDirty, ProfileNeedsAttention, ProfileSession, Season, ToolEnergy,
    PROFILE_COUNT,
};
use crate::items::Inventory;
use crate::player::{Loadout, Wallet, WorldProgress};

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

#[derive(SystemParam)]
pub(super) struct ProfileRun<'w> {
    inventory: ResMut<'w, Inventory>,
    loadout: ResMut<'w, Loadout>,
    progress: ResMut<'w, WorldProgress>,
    day_clock: ResMut<'w, DayClock>,
    tool_energy: ResMut<'w, ToolEnergy>,
    season: ResMut<'w, Season>,
    rules: Res<'w, CalendarRules>,
    audio: ResMut<'w, AudioSettings>,
    bindings: ResMut<'w, SkillBindings>,
    wallet: ResMut<'w, Wallet>,
    active: ResMut<'w, ActiveProfile>,
    profile: ResMut<'w, PlayerProfile>,
    global: ResMut<'w, GameSettings>,
    profile_dirty: ResMut<'w, ProfileDirty>,
    autosave_block: ResMut<'w, ProfileAutosaveBlock>,
}

pub fn begin_profile_run(index: u8, run: &mut ProfileRun) -> bool {
    if run.autosave_block.is_blocked(index) {
        return false;
    }
    let switched = activate_profile(
        index,
        &ProfileCapture {
            inventory: &run.inventory,
            loadout: &run.loadout,
            progress: &run.progress,
            day_clock: &run.day_clock,
            tool_energy: &run.tool_energy,
            audio: &run.audio,
            bindings: &run.bindings,
            wallet: &run.wallet,
        },
        &mut ProfileSession {
            active: &mut run.active,
            profile: &mut run.profile,
            global: &mut run.global,
            dirty: &mut run.profile_dirty,
            autosave_block: &mut run.autosave_block,
        },
    );
    if !switched {
        return false;
    }
    apply_profile_to_runtime(
        &run.profile,
        &mut ProfileApply {
            inventory: &mut run.inventory,
            loadout: &mut run.loadout,
            progress: &mut run.progress,
            day_clock: &mut run.day_clock,
            tool_energy: &mut run.tool_energy,
            audio: &mut run.audio,
            bindings: &mut run.bindings,
            season: &mut run.season,
            rules: &run.rules,
            wallet: &mut run.wallet,
        },
    );
    true
}
