use bevy::prelude::*;

use crate::audio::AudioSettings;
use crate::combat::SkillBindings;
use crate::items::Inventory;
use crate::player::{Loadout, Wallet, WorldProgress};

use super::super::day_cycle::{DayClock, ToolEnergy, TOOL_ENERGY_MAX};
use super::super::season::{CalendarDate, CalendarRules, Season};

use super::autosave_block::{load_profile_for_slot, ProfileAutosaveBlock};
use super::profile::{ActiveProfile, PlayerProfile, PROFILE_COUNT};
use super::settings::GameSettings;
use super::storage::{save_profile, save_root_display, save_settings};
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
    mut season: ResMut<Season>,
    rules: Res<CalendarRules>,
    mut wallet: ResMut<Wallet>,
) {
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
        &mut wallet,
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
    wallet: Res<Wallet>,
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
        && !wallet.is_changed()
    {
        return;
    }

    snapshot_profile(
        &ProfileCapture {
            inventory: &inventory,
            loadout: &loadout,
            progress: &progress,
            day_clock: &day_clock,
            tool_energy: &tool_energy,
            audio: &audio,
            bindings: &bindings,
            wallet: &wallet,
        },
        &mut profile,
    );
    dirty.mark();
}

pub struct ProfileCapture<'a> {
    pub inventory: &'a Inventory,
    pub loadout: &'a Loadout,
    pub progress: &'a WorldProgress,
    pub day_clock: &'a DayClock,
    pub tool_energy: &'a ToolEnergy,
    pub audio: &'a AudioSettings,
    pub bindings: &'a SkillBindings,
    pub wallet: &'a Wallet,
}

pub fn snapshot_profile(sources: &ProfileCapture<'_>, profile: &mut PlayerProfile) {
    profile.inventory = sources.inventory.clone();
    profile.loadout = sources.loadout.clone();
    profile.progress = sources.progress.clone();
    profile.calendar_day = sources.day_clock.calendar_day;
    profile.day_phase = sources.day_clock.phase;
    profile.tool_energy = sources.tool_energy.current;
    profile.wallet = *sources.wallet;
    profile.settings.capture_audio(sources.audio);
    profile.settings.capture_skill_bindings(sources.bindings);
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
    wallet: &Wallet,
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
        &ProfileCapture {
            inventory,
            loadout,
            progress,
            day_clock,
            tool_energy,
            audio,
            bindings,
            wallet,
        },
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
    wallet: &mut Wallet,
) {
    *inventory = profile.inventory.clone();
    *loadout = profile.loadout.clone();
    *progress = profile.progress.clone();
    *day_clock = DayClock::from_saved(profile.calendar_day, profile.day_phase);
    *tool_energy = ToolEnergy::from_saved(profile.tool_energy, TOOL_ENERGY_MAX);
    *season = CalendarDate::from_calendar_day(day_clock.calendar_day, rules).season;
    *wallet = profile.wallet;
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
        let mut wallet = Wallet::default();

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
            &mut wallet,
        );

        assert_eq!(day_clock.calendar_day, 200);
        assert_eq!(season, Season::Winter);
        assert_eq!(wallet, Wallet::default());
    }
}
