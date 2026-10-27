use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::audio::AudioSettings;
use crate::combat::SkillBindings;
use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

use super::super::day_cycle::{DayClock, ToolEnergy, TOOL_ENERGY_MAX};
use super::super::season::{CalendarDate, CalendarRules, Season};

use super::autosave_block::{load_profile_for_slot, ProfileAutosaveBlock};
use super::profile::{ActiveProfile, PlayerProfile, PROFILE_COUNT};
use super::settings::GameSettings;
use super::storage::{save_profile, save_root_display, save_settings};
use super::ProfileDirty;

#[derive(SystemParam)]
pub(crate) struct HydratedRuntime<'w> {
    audio: ResMut<'w, AudioSettings>,
    bindings: ResMut<'w, SkillBindings>,
    inventory: ResMut<'w, Inventory>,
    loadout: ResMut<'w, Loadout>,
    progress: ResMut<'w, WorldProgress>,
    day_clock: ResMut<'w, DayClock>,
    tool_energy: ResMut<'w, ToolEnergy>,
    season: ResMut<'w, Season>,
}

#[derive(SystemParam)]
pub(crate) struct CapturedRuntime<'w> {
    audio: Res<'w, AudioSettings>,
    bindings: Res<'w, SkillBindings>,
    inventory: Res<'w, Inventory>,
    loadout: Res<'w, Loadout>,
    progress: Res<'w, WorldProgress>,
    day_clock: Res<'w, DayClock>,
    tool_energy: Res<'w, ToolEnergy>,
}

pub fn hydrate_runtime_from_memory(
    profile: Res<PlayerProfile>,
    active: Res<ActiveProfile>,
    runtime: HydratedRuntime,
    rules: Res<CalendarRules>,
) {
    let HydratedRuntime {
        mut audio,
        mut bindings,
        mut inventory,
        mut loadout,
        mut progress,
        mut day_clock,
        mut tool_energy,
        mut season,
    } = runtime;
    apply_profile_to_runtime(
        &profile,
        &mut inventory,
        &mut loadout,
        &mut progress,
        &mut day_clock,
        &mut tool_energy,
        &mut audio,
        &mut bindings,
        &mut season,
        &rules,
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
    runtime: CapturedRuntime,
    mut profile: ResMut<PlayerProfile>,
    mut dirty: ResMut<ProfileDirty>,
) {
    let CapturedRuntime {
        audio,
        bindings,
        inventory,
        loadout,
        progress,
        day_clock,
        tool_energy,
    } = runtime;
    if !audio.is_changed()
        && !bindings.is_changed()
        && !inventory.is_changed()
        && !loadout.is_changed()
        && !progress.is_changed()
        && !tool_energy.is_changed()
    {
        return;
    }

    snapshot_profile(
        &inventory,
        &loadout,
        &progress,
        &day_clock,
        &tool_energy,
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
    audio: &AudioSettings,
    bindings: &SkillBindings,
    active: &mut ActiveProfile,
    profile: &mut PlayerProfile,
    global: &mut GameSettings,
    dirty: &mut ProfileDirty,
    autosave_block: &mut ProfileAutosaveBlock,
) -> bool {
    let index = index.min(PROFILE_COUNT - 1);
    if active.0 == index {
        return !autosave_block.is_blocked(index);
    }

    snapshot_profile(
        inventory,
        loadout,
        progress,
        day_clock,
        tool_energy,
        audio,
        bindings,
        profile,
    );
    if let Err(error) = save_profile(active.index(), profile) {
        warn!("Failed to save profile before switch: {error}");
    } else {
        dirty.0 = false;
    }

    let Ok(loaded) = load_profile_for_slot(index, autosave_block) else {
        return false;
    };
    *profile = loaded;
    active.0 = index;
    global.last_active_profile = index;
    if let Err(error) = save_settings(global) {
        warn!("Failed to save global preferences: {error}");
    }

    dirty.0 = false;
    true
}

pub fn apply_profile_to_runtime(
    profile: &PlayerProfile,
    inventory: &mut Inventory,
    loadout: &mut Loadout,
    progress: &mut WorldProgress,
    day_clock: &mut DayClock,
    tool_energy: &mut ToolEnergy,
    audio: &mut AudioSettings,
    bindings: &mut SkillBindings,
    season: &mut Season,
    rules: &CalendarRules,
) {
    *inventory = profile.inventory.clone();
    *loadout = profile.loadout.clone();
    *progress = profile.progress.clone();
    *day_clock = DayClock::from_saved(profile.calendar_day, profile.day_phase);
    *tool_energy = ToolEnergy::from_saved(profile.tool_energy, TOOL_ENERGY_MAX);
    *season = CalendarDate::from_calendar_day(day_clock.calendar_day, rules).season;
    profile.settings.apply_audio(audio);
    profile.settings.apply_skill_bindings(bindings);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applying_a_profile_snaps_season_to_the_saved_day() {
        let profile = PlayerProfile {
            calendar_day: 200,
            ..PlayerProfile::default()
        };
        let mut inventory = Inventory::default();
        let mut loadout = Loadout::default();
        let mut progress = WorldProgress::default();
        let mut day_clock = DayClock::default();
        let mut tool_energy = ToolEnergy::default();
        let mut audio = AudioSettings::default();
        let mut bindings = SkillBindings::default();
        let mut season = Season::Spring;

        apply_profile_to_runtime(
            &profile,
            &mut inventory,
            &mut loadout,
            &mut progress,
            &mut day_clock,
            &mut tool_energy,
            &mut audio,
            &mut bindings,
            &mut season,
            &CalendarRules::load(),
        );

        assert_eq!(day_clock.calendar_day, 200);
        assert_eq!(season, Season::Winter);
    }
}
