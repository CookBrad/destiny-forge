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
    mut runtime: ProfileApplyInputs,
) {
    runtime.apply_saved_profile(&profile);
    log_loaded_profile(&profile, active.index(), &runtime.day_clock);
}

pub fn capture_profile_from_runtime(
    sources: ProfileCaptureInputs,
    mut profile: ResMut<PlayerProfile>,
    mut dirty: ResMut<ProfileDirty>,
) {
    if !sources.gameplay_changed() {
        return;
    }

    snapshot_profile(&sources.capture(), &mut profile);
    dirty.mark();
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
    sources: &ProfileCapture<'_>,
    session: &mut ProfileSession<'_>,
) -> bool {
    let index = index.min(PROFILE_COUNT - 1);
    if session.active.0 == index {
        return !session.autosave_block.is_blocked(index);
    }

    snapshot_profile(sources, session.profile);
    save_profile_before_switch(session);
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

fn log_loaded_profile(profile: &PlayerProfile, active_index: u8, day_clock: &DayClock) {
    info!(
        "Loaded profile {} — weapon: {}, materials: {}, boss cleared: {}, {}",
        active_index + 1,
        profile.summary_weapon(),
        profile.summary_material_count(),
        profile.summary_boss_cleared(),
        day_clock.hud_label(),
    );
    info!("Save directory: {}", save_root_display());
}

fn save_profile_before_switch(session: &mut ProfileSession<'_>) {
    if let Err(error) = save_profile(session.active.index(), session.profile) {
        warn!("Failed to save profile before switch: {error}");
    } else {
        session.dirty.0 = false;
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

pub struct ProfileSession<'a> {
    pub active: &'a mut ActiveProfile,
    pub profile: &'a mut PlayerProfile,
    pub global: &'a mut GameSettings,
    pub dirty: &'a mut ProfileDirty,
    pub autosave_block: &'a mut ProfileAutosaveBlock,
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

    fn gameplay_changed(&self) -> bool {
        self.audio.is_changed()
            || self.bindings.is_changed()
            || self.inventory.is_changed()
            || self.loadout.is_changed()
            || self.progress.is_changed()
            || self.tool_energy.is_changed()
            || self.wallet.is_changed()
    }
}

#[derive(SystemParam)]
pub(super) struct ProfileApplyInputs<'w> {
    inventory: ResMut<'w, Inventory>,
    loadout: ResMut<'w, Loadout>,
    progress: ResMut<'w, WorldProgress>,
    day_clock: ResMut<'w, DayClock>,
    tool_energy: ResMut<'w, ToolEnergy>,
    audio: ResMut<'w, AudioSettings>,
    bindings: ResMut<'w, SkillBindings>,
    season: ResMut<'w, Season>,
    rules: Res<'w, CalendarRules>,
    wallet: ResMut<'w, Wallet>,
}

impl ProfileApplyInputs<'_> {
    fn apply_saved_profile(&mut self, profile: &PlayerProfile) {
        apply_profile_to_runtime(
            profile,
            &mut ProfileApply {
                inventory: &mut self.inventory,
                loadout: &mut self.loadout,
                progress: &mut self.progress,
                day_clock: &mut self.day_clock,
                tool_energy: &mut self.tool_energy,
                audio: &mut self.audio,
                bindings: &mut self.bindings,
                season: &mut self.season,
                rules: &self.rules,
                wallet: &mut self.wallet,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::DestinySpark;

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

    #[test]
    fn snapshot_and_apply_keep_the_wallet() {
        let mut profile = PlayerProfile::default();
        let inventory = Inventory::default();
        let loadout = Loadout::default();
        let progress = WorldProgress::default();
        let day_clock = DayClock::default();
        let tool_energy = ToolEnergy::default();
        let audio = AudioSettings::default();
        let bindings = SkillBindings::default();
        let wallet = Wallet {
            gold: 40,
            sparks: DestinySpark(2),
        };
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
        assert_eq!(profile.wallet, wallet);

        let mut inventory = Inventory::default();
        let mut loadout = Loadout::default();
        let mut progress = WorldProgress::default();
        let mut day_clock = DayClock::default();
        let mut tool_energy = ToolEnergy::default();
        let mut audio = AudioSettings::default();
        let mut bindings = SkillBindings::default();
        let mut season = Season::Spring;
        let rules = CalendarRules::load();
        let mut restored = Wallet::default();
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
                wallet: &mut restored,
            },
        );
        assert_eq!(restored, wallet);
    }

    #[test]
    fn snapshot_replaces_a_saved_wallet_with_an_empty_one() {
        let mut profile = PlayerProfile {
            wallet: Wallet {
                gold: 5,
                sparks: DestinySpark(1),
            },
            ..PlayerProfile::default()
        };
        let sources = HeldSources::with_wallet(Wallet::default());
        snapshot_profile(&sources.capture(), &mut profile);
        assert_eq!(profile.wallet, Wallet::default());
    }

    #[test]
    fn snapshot_keeps_the_largest_wallet_amounts() {
        let mut profile = PlayerProfile::default();
        let sources = HeldSources::with_wallet(Wallet {
            gold: u32::MAX,
            sparks: DestinySpark(u32::MAX),
        });
        snapshot_profile(&sources.capture(), &mut profile);
        assert_eq!(profile.wallet.gold, u32::MAX);
        assert_eq!(profile.wallet.sparks.amount(), u32::MAX);
    }

    #[test]
    fn hydrate_copies_the_saved_wallet_into_the_runtime() {
        let profile = PlayerProfile {
            wallet: Wallet {
                gold: 40,
                sparks: DestinySpark(2),
            },
            ..PlayerProfile::default()
        };
        let mut app = app_holding(profile, Wallet::default());
        app.add_systems(Update, hydrate_runtime_from_memory);
        app.update();
        let wallet = app.world().resource::<Wallet>();
        assert_eq!(wallet.gold, 40);
        assert_eq!(wallet.sparks, DestinySpark(2));
    }

    #[test]
    fn hydrate_clamps_a_zero_calendar_day_to_day_one() {
        let profile = PlayerProfile {
            calendar_day: 0,
            ..PlayerProfile::default()
        };
        let mut app = app_holding(profile, Wallet::default());
        app.add_systems(Update, hydrate_runtime_from_memory);
        app.update();
        assert_eq!(app.world().resource::<DayClock>().calendar_day, 1);
        assert_eq!(*app.world().resource::<Wallet>(), Wallet::default());
    }

    #[test]
    fn capture_stores_a_changed_wallet_and_marks_the_profile_dirty() {
        let mut app = app_holding(PlayerProfile::default(), Wallet::default());
        app.add_systems(Update, capture_profile_from_runtime);
        app.update();
        app.world_mut().resource_mut::<ProfileDirty>().0 = false;
        {
            let mut wallet = app.world_mut().resource_mut::<Wallet>();
            wallet.gold = 15;
            wallet.sparks = DestinySpark(6);
        }
        app.update();
        let profile = app.world().resource::<PlayerProfile>();
        assert_eq!(profile.wallet.gold, 15);
        assert_eq!(profile.wallet.sparks, DestinySpark(6));
        assert!(app.world().resource::<ProfileDirty>().0);
    }

    #[test]
    fn capture_ignores_a_day_clock_change() {
        let mut app = app_holding(PlayerProfile::default(), Wallet::default());
        app.add_systems(Update, capture_profile_from_runtime);
        app.update();
        app.world_mut().resource_mut::<ProfileDirty>().0 = false;
        app.world_mut().resource_mut::<DayClock>().calendar_day = 9;
        app.update();
        assert!(!app.world().resource::<ProfileDirty>().0);
        assert_eq!(app.world().resource::<PlayerProfile>().calendar_day, 1);
    }

    #[test]
    fn capture_leaves_the_profile_clean_when_nothing_changed() {
        let mut app = app_holding(PlayerProfile::default(), Wallet::default());
        app.add_systems(Update, capture_profile_from_runtime);
        app.update();
        app.world_mut().resource_mut::<ProfileDirty>().0 = false;
        app.world_mut().resource_mut::<PlayerProfile>().wallet.gold = 3;
        app.update();
        assert!(!app.world().resource::<ProfileDirty>().0);
        assert_eq!(app.world().resource::<PlayerProfile>().wallet.gold, 3);
    }

    #[test]
    fn staying_on_the_same_profile_keeps_its_wallet() {
        let mut profile = PlayerProfile {
            wallet: Wallet {
                gold: 4,
                sparks: DestinySpark(1),
            },
            ..PlayerProfile::default()
        };
        let kept = profile.wallet;
        let started = activate_on(&mut profile, ActiveProfile(0), 0, false);
        assert!(started);
        assert_eq!(profile.wallet, kept);
    }

    #[test]
    fn staying_on_a_blocked_profile_does_not_start() {
        let mut profile = PlayerProfile::default();
        let started = activate_on(&mut profile, ActiveProfile(0), 0, true);
        assert!(!started);
    }

    #[test]
    fn an_index_past_the_last_profile_stays_on_that_profile() {
        let mut profile = PlayerProfile {
            wallet: Wallet {
                gold: 8,
                sparks: DestinySpark(0),
            },
            ..PlayerProfile::default()
        };
        let started = activate_on(&mut profile, ActiveProfile(PROFILE_COUNT - 1), 99, false);
        assert!(started);
        assert_eq!(profile.wallet.gold, 8);
    }

    struct HeldSources {
        inventory: Inventory,
        loadout: Loadout,
        progress: WorldProgress,
        day_clock: DayClock,
        tool_energy: ToolEnergy,
        audio: AudioSettings,
        bindings: SkillBindings,
        wallet: Wallet,
    }

    impl HeldSources {
        fn with_wallet(wallet: Wallet) -> Self {
            Self {
                inventory: Inventory::default(),
                loadout: Loadout::default(),
                progress: WorldProgress::default(),
                day_clock: DayClock::default(),
                tool_energy: ToolEnergy::default(),
                audio: AudioSettings::default(),
                bindings: SkillBindings::default(),
                wallet,
            }
        }

        fn capture(&self) -> ProfileCapture<'_> {
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

    fn app_holding(profile: PlayerProfile, wallet: Wallet) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(profile)
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
            .insert_resource(ProfileDirty::default());
        app
    }

    fn activate_on(
        profile: &mut PlayerProfile,
        mut active: ActiveProfile,
        index: u8,
        blocked: bool,
    ) -> bool {
        let sources = HeldSources::with_wallet(profile.wallet);
        let mut global = GameSettings::default();
        let mut dirty = ProfileDirty::default();
        let mut autosave_block = ProfileAutosaveBlock::default();
        if blocked {
            autosave_block.block(active.index());
        }
        activate_profile(
            index,
            &sources.capture(),
            &mut ProfileSession {
                active: &mut active,
                profile,
                global: &mut global,
                dirty: &mut dirty,
                autosave_block: &mut autosave_block,
            },
        )
    }
}
