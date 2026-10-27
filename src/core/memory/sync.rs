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

#[derive(Clone, Copy)]
pub struct ProfileSnapshot<'a> {
    pub inventory: &'a Inventory,
    pub loadout: &'a Loadout,
    pub progress: &'a WorldProgress,
    pub day_clock: &'a DayClock,
    pub tool_energy: &'a ToolEnergy,
    pub audio: &'a AudioSettings,
    pub bindings: &'a SkillBindings,
}

pub struct RuntimeProfile<'a> {
    pub inventory: &'a mut Inventory,
    pub loadout: &'a mut Loadout,
    pub progress: &'a mut WorldProgress,
    pub day_clock: &'a mut DayClock,
    pub tool_energy: &'a mut ToolEnergy,
    pub season: &'a mut Season,
    pub audio: &'a mut AudioSettings,
    pub bindings: &'a mut SkillBindings,
}

pub struct ProfileSwitch<'a> {
    pub active: &'a mut ActiveProfile,
    pub profile: &'a mut PlayerProfile,
    pub global: &'a mut GameSettings,
    pub dirty: &'a mut ProfileDirty,
    pub autosave_block: &'a mut ProfileAutosaveBlock,
}

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
        ProfileSnapshot {
            inventory: &inventory,
            loadout: &loadout,
            progress: &progress,
            day_clock: &day_clock,
            tool_energy: &tool_energy,
            audio: &audio,
            bindings: &bindings,
        },
        &mut profile,
    );
    dirty.mark();
}

pub fn snapshot_profile(source: ProfileSnapshot<'_>, profile: &mut PlayerProfile) {
    profile.inventory = source.inventory.clone();
    profile.loadout = source.loadout.clone();
    profile.progress = source.progress.clone();
    profile.calendar_day = source.day_clock.calendar_day;
    profile.day_phase = source.day_clock.phase;
    profile.tool_energy = source.tool_energy.current;
    profile.settings.capture_audio(source.audio);
    profile.settings.capture_skill_bindings(source.bindings);
}

fn switch_profile(index: u8, source: &ProfileSnapshot<'_>, switch: &mut ProfileSwitch<'_>) -> bool {
    let index = index.min(PROFILE_COUNT - 1);
    if switch.active.0 == index {
        return !switch.autosave_block.is_blocked(index);
    }

    snapshot_profile(*source, switch.profile);
    if let Err(error) = save_profile(switch.active.index(), switch.profile) {
        warn!("Failed to save profile before switch: {error}");
    } else {
        switch.dirty.0 = false;
    }

    let Ok(loaded) = load_profile_for_slot(index, switch.autosave_block) else {
        return false;
    };
    *switch.profile = loaded;
    switch.active.0 = index;
    switch.global.last_active_profile = index;
    if let Err(error) = save_settings(switch.global) {
        warn!("Failed to save global preferences: {error}");
    }

    switch.dirty.0 = false;
    true
}

fn apply_saved_profile(
    profile: &PlayerProfile,
    runtime: &mut RuntimeProfile<'_>,
    rules: &CalendarRules,
) {
    *runtime.inventory = profile.inventory.clone();
    *runtime.loadout = profile.loadout.clone();
    *runtime.progress = profile.progress.clone();
    *runtime.day_clock = DayClock::from_saved(profile.calendar_day, profile.day_phase);
    *runtime.tool_energy = ToolEnergy::from_saved(profile.tool_energy, TOOL_ENERGY_MAX);
    *runtime.season = CalendarDate::from_calendar_day(runtime.day_clock.calendar_day, rules).season;
    profile.settings.apply_audio(runtime.audio);
    profile.settings.apply_skill_bindings(runtime.bindings);
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
    let source = ProfileSnapshot {
        inventory,
        loadout,
        progress,
        day_clock,
        tool_energy,
        audio,
        bindings,
    };
    switch_profile(
        index,
        &source,
        &mut ProfileSwitch {
            active,
            profile,
            global,
            dirty,
            autosave_block,
        },
    )
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
    apply_saved_profile(
        profile,
        &mut RuntimeProfile {
            inventory,
            loadout,
            progress,
            day_clock,
            tool_energy,
            season,
            audio,
            bindings,
        },
        rules,
    );
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
