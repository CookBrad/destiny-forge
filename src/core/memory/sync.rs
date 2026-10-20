use bevy::ecs::system::SystemParam;
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
        &mut ProfileApply {
            inventory: &mut inventory,
            loadout: &mut loadout,
            progress: &mut progress,
            day_clock: &mut day_clock,
            tool_energy: &mut tool_energy,
            audio: &mut audio,
            bindings: &mut bindings,
            season: &mut season,
            rules: &rules,
            wallet: &mut wallet,
        },
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
    sources: ProfileCaptureInputs,
    mut profile: ResMut<PlayerProfile>,
    mut dirty: ResMut<ProfileDirty>,
) {
    if !sources.any_saved_value_changed_except_day_clock() {
        return;
    }

    snapshot_profile(&sources.capture(), &mut profile);
    dirty.mark();
}

#[derive(SystemParam)]
pub(super) struct ProfileCaptureInputs<'w> {
    inventory: Res<'w, Inventory>,
    loadout: Res<'w, Loadout>,
    progress: Res<'w, WorldProgress>,
    day_clock: Res<'w, DayClock>,
    tool_energy: Res<'w, ToolEnergy>,
    audio: Res<'w, AudioSettings>,
    bindings: Res<'w, SkillBindings>,
    wallet: Res<'w, Wallet>,
}

impl ProfileCaptureInputs<'_> {
    fn any_saved_value_changed_except_day_clock(&self) -> bool {
        self.audio.is_changed()
            || self.bindings.is_changed()
            || self.inventory.is_changed()
            || self.loadout.is_changed()
            || self.progress.is_changed()
            || self.tool_energy.is_changed()
            || self.wallet.is_changed()
    }

    pub(super) fn capture(&self) -> ProfileCapture<'_> {
        ProfileCapture {
            inventory: &self.inventory,
            loadout: &self.loadout,
            progress: &self.progress,
            day_clock: &self.day_clock,
            tool_energy: &self.tool_energy,
            audio: &self.audio,
            bindings: &self.bindings,
            wallet: &self.wallet,
        }
    }
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

pub struct ProfileSession<'a> {
    pub active: &'a mut ActiveProfile,
    pub profile: &'a mut PlayerProfile,
    pub global: &'a mut GameSettings,
    pub dirty: &'a mut ProfileDirty,
    pub autosave_block: &'a mut ProfileAutosaveBlock,
}

pub fn activate_profile(
    index: u8,
    sources: &ProfileCapture<'_>,
    session: &mut ProfileSession<'_>,
) -> bool {
    let index = index.min(PROFILE_COUNT - 1);
    if session.active.0 == index {
        return !session.autosave_block.is_blocked(index);
    }

    snapshot_profile(sources, session.profile);
    if let Err(error) = save_profile(session.active.index(), session.profile) {
        warn!("Failed to save profile before switch: {error}");
    } else {
        session.dirty.0 = false;
    }

    let Ok(loaded) = load_profile_for_slot(index, session.autosave_block) else {
        return false;
    };
    *session.profile = loaded;
    session.active.0 = index;
    session.global.last_active_profile = index;
    if let Err(error) = save_settings(session.global) {
        warn!("Failed to save global preferences: {error}");
    }

    session.dirty.0 = false;
    true
}

pub struct ProfileApply<'a> {
    pub inventory: &'a mut Inventory,
    pub loadout: &'a mut Loadout,
    pub progress: &'a mut WorldProgress,
    pub day_clock: &'a mut DayClock,
    pub tool_energy: &'a mut ToolEnergy,
    pub audio: &'a mut AudioSettings,
    pub bindings: &'a mut SkillBindings,
    pub season: &'a mut Season,
    pub rules: &'a CalendarRules,
    pub wallet: &'a mut Wallet,
}

pub fn apply_profile_to_runtime(profile: &PlayerProfile, runtime: &mut ProfileApply<'_>) {
    *runtime.inventory = profile.inventory.clone();
    *runtime.loadout = profile.loadout.clone();
    *runtime.progress = profile.progress.clone();
    *runtime.day_clock = DayClock::from_saved(profile.calendar_day, profile.day_phase);
    *runtime.tool_energy = ToolEnergy::from_saved(profile.tool_energy, TOOL_ENERGY_MAX);
    *runtime.season =
        CalendarDate::from_calendar_day(runtime.day_clock.calendar_day, runtime.rules).season;
    *runtime.wallet = profile.wallet;
    profile.settings.apply_audio(runtime.audio);
    profile.settings.apply_skill_bindings(runtime.bindings);
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

        let rules = CalendarRules::load();
        apply_profile_to_runtime(
            &profile,
            &mut ProfileApply {
                inventory: &mut inventory,
                loadout: &mut loadout,
                progress: &mut progress,
                day_clock: &mut day_clock,
                tool_energy: &mut tool_energy,
                audio: &mut audio,
                bindings: &mut bindings,
                season: &mut season,
                rules: &rules,
                wallet: &mut wallet,
            },
        );

        assert_eq!(day_clock.calendar_day, 200);
        assert_eq!(season, Season::Winter);
        assert_eq!(wallet, Wallet::default());
    }
}
