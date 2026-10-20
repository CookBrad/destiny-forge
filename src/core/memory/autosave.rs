use bevy::prelude::*;

use super::autosave_block::ProfileAutosaveBlock;
use super::profile::{ActiveProfile, PlayerProfile};
use super::storage::save_profile;
use super::sync::{snapshot_profile, ProfileCaptureInputs};

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

pub fn debounced_autosave(
    time: Res<Time>,
    mut timer: ResMut<AutosaveTimer>,
    mut profile_dirty: ResMut<ProfileDirty>,
    active: Res<ActiveProfile>,
    sources: ProfileCaptureInputs,
    mut profile: ResMut<PlayerProfile>,
    autosave_block: Res<ProfileAutosaveBlock>,
) {
    if !profile_dirty.0 || autosave_block.is_blocked(active.index()) {
        return;
    }

    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    snapshot_profile(&sources.capture(), &mut profile);
    match save_profile(active.index(), &profile) {
        Ok(()) => profile_dirty.0 = false,
        Err(error) => warn!("Failed to save profile {}: {error}", active.index()),
    }
}

pub fn flush_saves_on_exit(
    active: Res<ActiveProfile>,
    sources: ProfileCaptureInputs,
    mut profile: ResMut<PlayerProfile>,
    mut profile_dirty: ResMut<ProfileDirty>,
    autosave_block: Res<ProfileAutosaveBlock>,
) {
    if !profile_dirty.0 || autosave_block.is_blocked(active.index()) {
        return;
    }

    snapshot_profile(&sources.capture(), &mut profile);
    if let Err(error) = save_profile(active.index(), &profile) {
        warn!("Failed to flush profile on exit: {error}");
    } else {
        profile_dirty.0 = false;
    }
}
