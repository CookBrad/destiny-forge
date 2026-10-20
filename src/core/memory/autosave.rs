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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioSettings;
    use crate::combat::SkillBindings;
    use crate::core::day_cycle::{DayClock, ToolEnergy};
    use crate::core::season::{CalendarRules, Season};
    use crate::items::Inventory;
    use crate::player::{DestinySpark, Loadout, Wallet, WorldProgress};

    use super::super::autosave_block::ProfileAutosaveBlock;
    use super::super::storage::load_profile;

    #[test]
    fn debounced_autosave_writes_the_wallet_after_the_delay() {
        let _lock = SaveDirectoryLock::acquire("debounce-write");
        let mut app = autosave_app(Wallet {
            gold: 8,
            sparks: DestinySpark(1),
        });
        finish_timer(&mut app);
        app.add_systems(Update, debounced_autosave);
        app.update();
        assert!(!app.world().resource::<ProfileDirty>().0);
        let saved = load_profile(0).unwrap();
        assert_eq!(saved.wallet.gold, 8);
        assert_eq!(saved.wallet.sparks, DestinySpark(1));
    }

    #[test]
    fn debounced_autosave_waits_while_the_timer_is_running() {
        let mut app = autosave_app(Wallet {
            gold: 8,
            sparks: DestinySpark(1),
        });
        app.add_systems(Update, debounced_autosave);
        app.update();
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(app.world().resource::<PlayerProfile>().wallet.gold, 0);
    }

    #[test]
    fn debounced_autosave_skips_a_clean_profile() {
        let mut app = autosave_app(Wallet::default());
        app.world_mut().resource_mut::<ProfileDirty>().0 = false;
        finish_timer(&mut app);
        app.add_systems(Update, debounced_autosave);
        app.update();
        assert!(!app.world().resource::<ProfileDirty>().0);
        assert_eq!(
            app.world().resource::<PlayerProfile>().wallet,
            Wallet::default()
        );
    }

    #[test]
    fn debounced_autosave_skips_a_blocked_profile() {
        let mut app = autosave_app(Wallet {
            gold: 8,
            sparks: DestinySpark(1),
        });
        app.world_mut()
            .resource_mut::<ProfileAutosaveBlock>()
            .block(0);
        finish_timer(&mut app);
        app.add_systems(Update, debounced_autosave);
        app.update();
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(app.world().resource::<PlayerProfile>().wallet.gold, 0);
    }

    #[test]
    fn flush_saves_on_exit_writes_the_wallet() {
        let _lock = SaveDirectoryLock::acquire("flush-write");
        let mut app = autosave_app(Wallet {
            gold: 21,
            sparks: DestinySpark(5),
        });
        app.add_systems(Update, flush_saves_on_exit);
        app.update();
        assert!(!app.world().resource::<ProfileDirty>().0);
        let saved = load_profile(0).unwrap();
        assert_eq!(saved.wallet.gold, 21);
        assert_eq!(saved.wallet.sparks.amount(), 5);
    }

    #[test]
    fn flush_saves_on_exit_skips_a_clean_profile() {
        let mut app = autosave_app(Wallet::default());
        app.world_mut().resource_mut::<ProfileDirty>().0 = false;
        app.add_systems(Update, flush_saves_on_exit);
        app.update();
        assert_eq!(
            app.world().resource::<PlayerProfile>().wallet,
            Wallet::default()
        );
    }

    #[test]
    fn flush_saves_on_exit_skips_a_blocked_profile() {
        let mut app = autosave_app(Wallet {
            gold: 21,
            sparks: DestinySpark(5),
        });
        app.world_mut()
            .resource_mut::<ProfileAutosaveBlock>()
            .block(0);
        app.add_systems(Update, flush_saves_on_exit);
        app.update();
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(app.world().resource::<PlayerProfile>().wallet.gold, 0);
    }

    fn autosave_app(wallet: Wallet) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PlayerProfile::default())
            .insert_resource(ActiveProfile(0))
            .insert_resource(Inventory::default())
            .insert_resource(Loadout::default())
            .insert_resource(WorldProgress::default())
            .insert_resource(DayClock::default())
            .insert_resource(ToolEnergy::default())
            .insert_resource(AudioSettings::default())
            .insert_resource(SkillBindings::default())
            .insert_resource(Season::Spring)
            .insert_resource(CalendarRules::load())
            .insert_resource(wallet)
            .insert_resource(ProfileDirty(true))
            .insert_resource(AutosaveTimer::default())
            .insert_resource(ProfileAutosaveBlock::default());
        app
    }

    fn finish_timer(app: &mut App) {
        let mut timer = app.world_mut().resource_mut::<AutosaveTimer>();
        timer.0 = Timer::from_seconds(0.0, TimerMode::Once);
    }

    struct SaveDirectoryLock;

    impl SaveDirectoryLock {
        fn acquire(label: &str) -> Self {
            let path = std::env::temp_dir().join("destiny_forge_wallet_save_lock");
            loop {
                if std::fs::create_dir(&path).is_ok() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            let root = std::env::temp_dir().join(format!(
                "destiny_forge_wallet_{label}_{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(&root).unwrap();
            std::env::set_var("XDG_DATA_HOME", &root);
            Self
        }
    }

    impl Drop for SaveDirectoryLock {
        fn drop(&mut self) {
            let _ =
                std::fs::remove_dir(std::env::temp_dir().join("destiny_forge_wallet_save_lock"));
        }
    }
}
