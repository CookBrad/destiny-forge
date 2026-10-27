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

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use super::*;

    const SAVE_SLOT: u8 = 2;
    const SAVED_DAY: u32 = 18;

    fn lock_save_slot() -> std::sync::MutexGuard<'static, ()> {
        super::super::storage::lock_profile_files()
    }

    #[test]
    fn a_dirty_profile_saves_once_the_debounce_elapses() {
        let _slot = lock_save_slot();
        let restored = RestoredSave::new(SAVE_SLOT);
        let mut app = autosave_app(Duration::ZERO, true);
        app.world_mut().resource_mut::<DayClock>().calendar_day = SAVED_DAY;
        app.update();
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(std::fs::read(&restored.path).ok(), restored.previous);
        let delay = Duration::from_secs_f32(AUTOSAVE_DELAY_SECS);
        app.world_mut().resource_mut::<Time<()>>().advance_by(delay);
        app.update();
        assert!(!app.world().resource::<ProfileDirty>().0);
        assert_eq!(
            super::super::storage::load_profile(SAVE_SLOT)
                .unwrap()
                .calendar_day,
            SAVED_DAY
        );
    }

    fn autosave_app(step: Duration, dirty: bool) -> App {
        let mut time = Time::<()>::default();
        time.advance_by(step);
        let mut app = App::new();
        app.insert_resource(time)
            .insert_resource(ActiveProfile(SAVE_SLOT))
            .insert_resource(AudioSettings::default())
            .insert_resource(SkillBindings::default())
            .insert_resource(Inventory::default())
            .insert_resource(Loadout::default())
            .insert_resource(WorldProgress::default())
            .insert_resource(DayClock::default())
            .insert_resource(ToolEnergy::default())
            .insert_resource(PlayerProfile::default())
            .insert_resource(ProfileAutosaveBlock::default())
            .insert_resource(ProfileDirty(dirty))
            .insert_resource(AutosaveTimer::default())
            .add_systems(Update, debounced_autosave);
        app
    }

    struct RestoredSave {
        path: PathBuf,
        previous: Option<Vec<u8>>,
        backups_before: Vec<PathBuf>,
    }

    impl RestoredSave {
        fn new(index: u8) -> Self {
            let path = super::super::storage::profile_path(index);
            let previous = std::fs::read(&path).ok();
            let backups_before = corrupt_backups(&path);
            Self {
                path,
                previous,
                backups_before,
            }
        }
    }

    impl Drop for RestoredSave {
        fn drop(&mut self) {
            match &self.previous {
                Some(bytes) => {
                    if let Some(parent) = self.path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::write(&self.path, bytes);
                }
                None => {
                    let _ = std::fs::remove_file(&self.path);
                }
            }
            for backup in corrupt_backups(&self.path) {
                if !self.backups_before.contains(&backup) {
                    let _ = std::fs::remove_file(backup);
                }
            }
        }
    }

    fn corrupt_backups(path: &Path) -> Vec<PathBuf> {
        let Some(parent) = path.parent() else {
            return Vec::new();
        };
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            return Vec::new();
        };
        let prefix = format!("{stem}.corrupt-");
        std::fs::read_dir(parent)
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|candidate| {
                let name = candidate.file_name().unwrap_or_default().to_string_lossy();
                candidate.is_file() && name.starts_with(&prefix) && name.ends_with(".ron")
            })
            .collect()
    }

    #[test]
    fn an_unchanged_profile_is_not_written() {
        let _slot = lock_save_slot();
        let restored = RestoredSave::new(SAVE_SLOT);
        let mut app = autosave_app(Duration::from_secs_f32(AUTOSAVE_DELAY_SECS), false);
        app.world_mut().resource_mut::<DayClock>().calendar_day = SAVED_DAY;
        app.update();
        assert!(!app.world().resource::<ProfileDirty>().0);
        assert_eq!(
            app.world().resource::<PlayerProfile>().calendar_day,
            PlayerProfile::default().calendar_day
        );
        assert_eq!(std::fs::read(&restored.path).ok(), restored.previous);
    }

    #[test]
    fn a_corrupt_profile_stays_dirty_when_the_write_fails() {
        let _slot = lock_save_slot();
        let restored = RestoredSave::new(SAVE_SLOT);
        let corrupt = b"this is not a profile {{{{".to_vec();
        if let Some(parent) = restored.path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&restored.path, &corrupt).unwrap();
        let mut app = autosave_app(Duration::from_secs_f32(AUTOSAVE_DELAY_SECS), true);
        app.world_mut().resource_mut::<DayClock>().calendar_day = SAVED_DAY;
        app.update();
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(std::fs::read(&restored.path).unwrap(), corrupt);
        assert_eq!(
            app.world().resource::<PlayerProfile>().calendar_day,
            SAVED_DAY
        );
    }
}
