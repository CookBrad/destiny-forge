use bevy::prelude::*;

use crate::audio::AudioSettings;
use crate::combat::SkillBindings;
use crate::cooking::ActiveFoodBuff;
use crate::farming::HomesteadHotbar;
use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

use super::super::day_cycle::{DayClock, ToolEnergy, TOOL_ENERGY_MAX};

use super::profile::{ActiveProfile, PlayerProfile, PROFILE_COUNT};
use super::settings::GameSettings;
use super::storage::{load_profile, save_profile, save_settings, save_root_display};
use super::ProfileDirty;

pub fn hydrate_runtime_from_memory(
    profile: Res<PlayerProfile>,
    active: Res<ActiveProfile>,
    mut audio: ResMut<AudioSettings>,
    mut bindings: ResMut<SkillBindings>,
    mut inventory: ResMut<Inventory>,
    mut loadout: ResMut<Loadout>,
    mut progress: ResMut<WorldProgress>,
    mut day_clock: ResMut<DayClock>,
    mut tool_energy: ResMut<ToolEnergy>,
    mut hotbar: ResMut<HomesteadHotbar>,
    mut food_buff: ResMut<ActiveFoodBuff>,
) {
    apply_profile_to_runtime(
        &profile,
        &mut inventory,
        &mut loadout,
        &mut progress,
        &mut day_clock,
        &mut tool_energy,
        &mut hotbar,
        &mut food_buff,
        &mut audio,
        &mut bindings,
    );

    info!(
        "Loaded profile {} — weapon: {}, materials: {}, boss cleared: {}, {}",
        active.index() + 1,
        profile.summary_weapon(),
        profile.summary_material_count(),
        profile.summary_boss_cleared(),
        day_clock.hud_label(),
    );
    info!("Save directory: {}", save_root_display());
}

pub fn capture_profile_from_runtime(
    audio: Res<AudioSettings>,
    bindings: Res<SkillBindings>,
    inventory: Res<Inventory>,
    loadout: Res<Loadout>,
    progress: Res<WorldProgress>,
    day_clock: Res<DayClock>,
    tool_energy: Res<ToolEnergy>,
    hotbar: Res<HomesteadHotbar>,
    food_buff: Res<ActiveFoodBuff>,
    mut profile: ResMut<PlayerProfile>,
    mut dirty: ResMut<ProfileDirty>,
) {
    // Day phase/calendar are written to profile on phase advance + sleep (not every timer tick).
    if !audio.is_changed()
        && !bindings.is_changed()
        && !inventory.is_changed()
        && !loadout.is_changed()
        && !progress.is_changed()
        && !tool_energy.is_changed()
        && !hotbar.is_changed()
        && !food_buff.is_changed()
    {
        return;
    }

    snapshot_profile(
        &inventory,
        &loadout,
        &progress,
        &day_clock,
        &tool_energy,
        &hotbar,
        &food_buff,
        &audio,
        &bindings,
        &mut profile,
    );
    dirty.mark();
}

pub fn snapshot_profile(
    inventory: &Inventory,
    loadout: &Loadout,
    progress: &WorldProgress,
    day_clock: &DayClock,
    tool_energy: &ToolEnergy,
    hotbar: &HomesteadHotbar,
    food_buff: &ActiveFoodBuff,
    audio: &AudioSettings,
    bindings: &SkillBindings,
    profile: &mut PlayerProfile,
) {
    profile.inventory = inventory.clone();
    profile.loadout = loadout.clone();
    profile.progress = progress.clone();
    profile.calendar_day = day_clock.calendar_day;
    profile.day_phase = day_clock.phase;
    profile.tool_energy = tool_energy.current;
    profile.hotbar = hotbar.clone();
    profile.food_buff = food_buff.clone();
    profile.settings.capture_audio(audio);
    profile.settings.capture_skill_bindings(bindings);
}

pub fn activate_profile(
    index: u8,
    inventory: &Inventory,
    loadout: &Loadout,
    progress: &WorldProgress,
    day_clock: &DayClock,
    tool_energy: &ToolEnergy,
    hotbar: &HomesteadHotbar,
    food_buff: &ActiveFoodBuff,
    audio: &AudioSettings,
    bindings: &SkillBindings,
    active: &mut ActiveProfile,
    profile: &mut PlayerProfile,
    global: &mut GameSettings,
    dirty: &mut ProfileDirty,
) {
    let index = index.min(PROFILE_COUNT - 1);
    if active.0 == index {
        return;
    }

    snapshot_profile(
        inventory,
        loadout,
        progress,
        day_clock,
        tool_energy,
        hotbar,
        food_buff,
        audio,
        bindings,
        profile,
    );
    if let Err(error) = save_profile(active.index(), profile) {
        warn!("Failed to save profile before switch: {error}");
    } else {
        dirty.0 = false;
    }

    *profile = load_profile(index);
    active.0 = index;
    global.last_active_profile = index;
    if let Err(error) = save_settings(global) {
        warn!("Failed to save global preferences: {error}");
    }

    dirty.0 = false;
}

pub fn apply_profile_to_runtime(
    profile: &PlayerProfile,
    inventory: &mut Inventory,
    loadout: &mut Loadout,
    progress: &mut WorldProgress,
    day_clock: &mut DayClock,
    tool_energy: &mut ToolEnergy,
    hotbar: &mut HomesteadHotbar,
    food_buff: &mut ActiveFoodBuff,
    audio: &mut AudioSettings,
    bindings: &mut SkillBindings,
) {
    *inventory = profile.inventory.clone();
    *loadout = profile.loadout.clone();
    *progress = profile.progress.clone();
    *day_clock = DayClock::from_saved(profile.calendar_day, profile.day_phase);
    *tool_energy = ToolEnergy::from_saved(profile.tool_energy, TOOL_ENERGY_MAX);
    *hotbar = profile.hotbar.clone();
    *food_buff = profile.food_buff.clone();
    profile.settings.apply_audio(audio);
    profile.settings.apply_skill_bindings(bindings);
}
