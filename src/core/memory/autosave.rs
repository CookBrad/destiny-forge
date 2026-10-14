use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::audio::AudioSettings;
use crate::combat::SkillBindings;
use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

use super::super::day_cycle::{DayClock, ToolEnergy};

use super::autosave_block::ProfileAutosaveBlock;
use super::profile::{ActiveProfile, PlayerProfile};
use super::storage::save_profile;
use super::sync::snapshot_profile;

const AUTOSAVE_DELAY_SECS: f32 = 0.75;

#[derive(Resource, Default)]
pub struct ProfileDirty(pub bool);

impl ProfileDirty {
    pub fn mark(&mut self) {
        self.0 = true;
    }
}

#[derive(Resource)]
pub struct AutosaveTimer(Timer);

impl Default for AutosaveTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(AUTOSAVE_DELAY_SECS, TimerMode::Once))
    }
}

pub fn queue_autosave(profile_dirty: Res<ProfileDirty>, mut timer: ResMut<AutosaveTimer>) {
    if profile_dirty.0 {
        timer.0.reset();
    }
}

#[derive(SystemParam)]
pub(crate) struct ProfileSaveState<'w> {
    active: Res<'w, ActiveProfile>,
    audio: Res<'w, AudioSettings>,
    bindings: Res<'w, SkillBindings>,
    inventory: Res<'w, Inventory>,
    loadout: Res<'w, Loadout>,
    progress: Res<'w, WorldProgress>,
    day_clock: Res<'w, DayClock>,
    tool_energy: Res<'w, ToolEnergy>,
    profile: ResMut<'w, PlayerProfile>,
    autosave_block: Res<'w, ProfileAutosaveBlock>,
}

fn profile_snapshot<'a>(
    inventory: &'a Inventory,
    loadout: &'a Loadout,
    progress: &'a WorldProgress,
    day_clock: &'a DayClock,
    tool_energy: &'a ToolEnergy,
    audio: &'a AudioSettings,
    bindings: &'a SkillBindings,
) -> super::sync::ProfileSnapshot<'a> {
    super::sync::ProfileSnapshot {
        inventory,
        loadout,
        progress,
        day_clock,
        tool_energy,
        audio,
        bindings,
    }
}

pub fn debounced_autosave(
    time: Res<Time>,
    mut timer: ResMut<AutosaveTimer>,
    mut profile_dirty: ResMut<ProfileDirty>,
    state: ProfileSaveState,
) {
    let ProfileSaveState {
        active,
        audio,
        bindings,
        inventory,
        loadout,
        progress,
        day_clock,
        tool_energy,
        mut profile,
        autosave_block,
    } = state;
    if !profile_dirty.0 || autosave_block.is_blocked(active.index()) {
        return;
    }

    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    snapshot_profile(
        profile_snapshot(
            &inventory,
            &loadout,
            &progress,
            &day_clock,
            &tool_energy,
            &audio,
            &bindings,
        ),
        &mut profile,
    );
    match save_profile(active.index(), &profile) {
        Ok(()) => profile_dirty.0 = false,
        Err(error) => warn!("Failed to save profile {}: {error}", active.index()),
    }
}

pub fn flush_saves_on_exit(mut profile_dirty: ResMut<ProfileDirty>, state: ProfileSaveState) {
    let ProfileSaveState {
        active,
        audio,
        bindings,
        inventory,
        loadout,
        progress,
        day_clock,
        tool_energy,
        mut profile,
        autosave_block,
    } = state;
    if !profile_dirty.0 || autosave_block.is_blocked(active.index()) {
        return;
    }

    snapshot_profile(
        profile_snapshot(
            &inventory,
            &loadout,
            &progress,
            &day_clock,
            &tool_energy,
            &audio,
            &bindings,
        ),
        &mut profile,
    );
    if let Err(error) = save_profile(active.index(), &profile) {
        warn!("Failed to flush profile on exit: {error}");
    } else {
        profile_dirty.0 = false;
    }
}
