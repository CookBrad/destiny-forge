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
    if save_profile_before_switch(session) {
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

fn save_profile_before_switch(session: &ProfileSession<'_>) -> bool {
    match save_profile(session.active.index(), session.profile) {
        Ok(()) => true,
        Err(error) => {
            warn!("Failed to save profile before switch: {error}");
            false
        }
    }
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
    use crate::combat::WeaponKind;
    use crate::player::DestinySpark;
    #[test]
    fn applying_a_profile_snaps_season_to_the_saved_day() {
        let saved = Wallet {
            gold: 40,
            sparks: DestinySpark(2),
        };
        let profile = PlayerProfile {
            calendar_day: 200,
            wallet: saved,
            ..PlayerProfile::default()
        };
        let mut wallet = Wallet {
            gold: 9,
            sparks: DestinySpark(8),
        };
        let mut day_clock = DayClock::default();
        let mut season = Season::Spring;
        apply_saved_wallet(&profile, &mut day_clock, &mut season, &mut wallet);

        assert_eq!(day_clock.calendar_day, 200);
        assert_eq!(season, Season::Winter);
        assert_eq!(wallet, saved);
    }

    #[test]
    fn applying_an_empty_wallet_replaces_the_runtime_wallet() {
        let mut wallet = Wallet {
            gold: 9,
            sparks: DestinySpark(8),
        };
        let mut day_clock = DayClock::default();
        let mut season = Season::Spring;
        apply_saved_wallet(
            &PlayerProfile::default(),
            &mut day_clock,
            &mut season,
            &mut wallet,
        );
        assert_eq!(wallet, Wallet::default());
    }

    #[test]
    fn applying_a_profile_keeps_the_largest_wallet_amounts() {
        let saved = Wallet {
            gold: u32::MAX,
            sparks: DestinySpark(u32::MAX),
        };
        let profile = PlayerProfile {
            wallet: saved,
            ..PlayerProfile::default()
        };
        let mut wallet = Wallet::default();
        let mut day_clock = DayClock::default();
        let mut season = Season::Spring;
        apply_saved_wallet(&profile, &mut day_clock, &mut season, &mut wallet);
        assert_eq!(wallet.gold, u32::MAX);
        assert_eq!(wallet.sparks.amount(), u32::MAX);
    }

    #[test]
    fn snapshot_keeps_the_wallet() {
        let mut profile = PlayerProfile::default();
        let wallet = Wallet {
            gold: 40,
            sparks: DestinySpark(2),
        };
        snapshot_wallet(&mut profile, &wallet);
        assert_eq!(profile.wallet, wallet);
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
        snapshot_wallet(&mut profile, &Wallet::default());
        assert_eq!(profile.wallet, Wallet::default());
    }

    #[test]
    fn snapshot_keeps_the_largest_wallet_amounts() {
        let mut profile = PlayerProfile::default();
        snapshot_wallet(
            &mut profile,
            &Wallet {
                gold: u32::MAX,
                sparks: DestinySpark(u32::MAX),
            },
        );
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
    fn capture_stores_a_changed_wallet() {
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
    fn staying_on_the_same_profile_keeps_its_wallet() {
        let mut profile = PlayerProfile {
            wallet: Wallet {
                gold: 4,
                sparks: DestinySpark(1),
            },
            ..PlayerProfile::default()
        };
        let kept = profile.wallet;
        let (started, _) = activate_open_profile(&mut profile, ActiveProfile(0), 0);
        assert!(started);
        assert_eq!(profile.wallet, kept);
    }

    #[test]
    fn staying_on_a_blocked_profile_does_not_start() {
        let mut profile = PlayerProfile::default();
        let (started, _) = activate_blocked_profile(&mut profile, ActiveProfile(0), 0);
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
        let (started, active) =
            activate_open_profile(&mut profile, ActiveProfile(PROFILE_COUNT - 1), 99);
        assert!(started);
        assert_eq!(profile.wallet.gold, 8);
        assert_eq!(active.index(), PROFILE_COUNT - 1);
    }

    fn apply_saved_wallet(
        profile: &PlayerProfile,
        day_clock: &mut DayClock,
        season: &mut Season,
        wallet: &mut Wallet,
    ) {
        let mut inventory = Inventory::default();
        let mut loadout = Loadout::default();
        let mut progress = WorldProgress::default();
        let mut tool_energy = ToolEnergy::default();
        let mut audio = AudioSettings::default();
        let mut bindings = SkillBindings::default();
        let rules = CalendarRules::load();
        apply_profile_to_runtime(
            profile,
            &mut ProfileApply {
                inventory: &mut inventory,
                loadout: &mut loadout,
                progress: &mut progress,
                day_clock,
                tool_energy: &mut tool_energy,
                audio: &mut audio,
                bindings: &mut bindings,
                season,
                rules: &rules,
                wallet,
            },
        );
    }

    fn snapshot_wallet(profile: &mut PlayerProfile, wallet: &Wallet) {
        let inventory = Inventory::default();
        let loadout = Loadout::default();
        let progress = WorldProgress::default();
        let day_clock = DayClock::default();
        let tool_energy = ToolEnergy::default();
        let audio = AudioSettings::default();
        let bindings = SkillBindings::default();
        snapshot_profile(
            &ProfileCapture {
                inventory: &inventory,
                loadout: &loadout,
                progress: &progress,
                day_clock: &day_clock,
                tool_energy: &tool_energy,
                audio: &audio,
                bindings: &bindings,
                wallet,
            },
            profile,
        );
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

    fn activate_open_profile(
        profile: &mut PlayerProfile,
        active: ActiveProfile,
        index: u8,
    ) -> (bool, ActiveProfile) {
        let mut autosave_block = ProfileAutosaveBlock::default();
        activate_profile_with_block(profile, active, index, &mut autosave_block)
    }

    fn activate_blocked_profile(
        profile: &mut PlayerProfile,
        active: ActiveProfile,
        index: u8,
    ) -> (bool, ActiveProfile) {
        let mut autosave_block = ProfileAutosaveBlock::default();
        autosave_block.block(active.index());
        activate_profile_with_block(profile, active, index, &mut autosave_block)
    }

    fn activate_profile_with_block(
        profile: &mut PlayerProfile,
        mut active: ActiveProfile,
        index: u8,
        autosave_block: &mut ProfileAutosaveBlock,
    ) -> (bool, ActiveProfile) {
        let inventory = Inventory::default();
        let loadout = Loadout::default();
        let progress = WorldProgress::default();
        let day_clock = DayClock::default();
        let tool_energy = ToolEnergy::default();
        let audio = AudioSettings::default();
        let bindings = SkillBindings::default();
        let wallet = profile.wallet;
        let mut global = GameSettings::default();
        let mut dirty = ProfileDirty::default();
        let started = activate_profile(
            index,
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
            &mut ProfileSession {
                active: &mut active,
                profile,
                global: &mut global,
                dirty: &mut dirty,
                autosave_block,
            },
        );
        (started, active)
    }

    #[test]
    fn apply_restores_the_wallet() {
        let mut restored = Wallet {
            gold: 9,
            sparks: DestinySpark(8),
        };
        restore_wallet(
            Wallet {
                gold: 40,
                sparks: DestinySpark(2),
            },
            &mut restored,
        );
        assert_eq!(restored.gold, 40);
        assert_eq!(restored.sparks.amount(), 2);

        restored = Wallet {
            gold: 9,
            sparks: DestinySpark(8),
        };
        restore_wallet(Wallet::default(), &mut restored);
        assert_eq!(restored, Wallet::default());

        restored = Wallet::default();
        restore_wallet(
            Wallet {
                gold: u32::MAX,
                sparks: DestinySpark(u32::MAX),
            },
            &mut restored,
        );
        assert_eq!(restored.gold, u32::MAX);
        assert_eq!(restored.sparks.amount(), u32::MAX);
    }

    fn restore_wallet(saved: Wallet, restored: &mut Wallet) {
        let profile = PlayerProfile {
            wallet: saved,
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
                wallet: restored,
            },
        );
    }

    #[test]
    fn capture_reads_the_wallet_from_the_runtime() {
        let seen = see_capture(
            Wallet {
                gold: 40,
                sparks: DestinySpark(2),
            },
            |_| {},
        );
        assert_eq!((seen.gold, seen.sparks), (40, 2));
    }

    #[test]
    fn capture_reads_an_empty_wallet() {
        let seen = see_capture(Wallet::default(), |_| {});
        assert_eq!((seen.gold, seen.sparks), (0, 0));
    }

    #[test]
    fn capture_reads_the_largest_wallet_from_the_runtime() {
        let seen = see_capture(
            Wallet {
                gold: u32::MAX,
                sparks: DestinySpark(u32::MAX),
            },
            |_| {},
        );
        assert_eq!((seen.gold, seen.sparks), (u32::MAX, u32::MAX));
    }

    #[test]
    fn a_wallet_change_counts_as_a_saved_value() {
        let seen = see_capture(Wallet::default(), |app| {
            app.world_mut().resource_mut::<Wallet>().gold = 6;
        });
        assert!(seen.saved_value_changed);
        assert_eq!(seen.gold, 6);
    }

    #[test]
    fn an_unchanged_runtime_does_not_count_as_a_saved_value() {
        let seen = see_capture(Wallet::default(), |_| {});
        assert!(!seen.saved_value_changed);
    }

    #[test]
    fn a_day_clock_change_does_not_count_as_a_saved_value() {
        let seen = see_capture(Wallet::default(), |app| {
            app.world_mut().resource_mut::<DayClock>().calendar_day = 12;
        });
        assert!(!seen.saved_value_changed);
        assert_eq!(seen.day, 12);
    }

    fn see_capture(wallet: Wallet, change: impl FnOnce(&mut App)) -> SeenCapture {
        let mut app = runtime_app(wallet);
        app.insert_resource(SeenCapture::default())
            .add_systems(Update, read_capture);
        app.update();
        app.world_mut().resource_mut::<SeenCapture>().saved_value_changed = false;
        change(&mut app);
        app.update();
        *app.world().resource::<SeenCapture>()
    }

    fn runtime_app(wallet: Wallet) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PlayerProfile::default())
            .insert_resource(Inventory::default())
            .insert_resource(Loadout::default())
            .insert_resource(WorldProgress::default())
            .insert_resource(DayClock::default())
            .insert_resource(ToolEnergy::default())
            .insert_resource(AudioSettings::default())
            .insert_resource(SkillBindings::default())
            .insert_resource(wallet)
            .insert_resource(ProfileDirty::default());
        app
    }

    fn read_capture(sources: ProfileCaptureInputs, mut seen: ResMut<SeenCapture>) {
        let captured = sources.capture();
        seen.gold = captured.wallet.gold;
        seen.sparks = captured.wallet.sparks.amount();
        seen.day = captured.day_clock.calendar_day;
        seen.saved_value_changed = sources.any_saved_value_changed_except_day_clock();
    }

    #[derive(Resource, Default, Clone, Copy)]
    struct SeenCapture {
        gold: u32,
        sparks: u32,
        day: u32,
        saved_value_changed: bool,
    }

    #[test]
    fn capture_fails_when_the_wallet_is_missing() {
        let mut app = runtime_app(Wallet::default());
        app.world_mut().remove_resource::<Wallet>();
        app.add_systems(Update, capture_profile_from_runtime);
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| app.update()))
            .expect_err("capture panics");
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .unwrap();
        assert_eq!(message, "destiny_forge::core::memory::sync::capture_profile_from_runtime could not access system parameter Res<'_, Wallet>");
    }

    #[test]
    fn changing_inventory_marks_the_profile_dirty() {
        let app = capture_after(|app| {
            app.world_mut().resource_mut::<Inventory>().slots[0].count = 4;
        });
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(saved_profile(&app).inventory.slots[0].count, 4);
    }

    fn capture_after(change: impl FnOnce(&mut App)) -> App {
        let mut app = runtime_app(Wallet::default());
        app.add_systems(Update, capture_profile_from_runtime);
        app.update();
        mark_clean(&mut app);
        change(&mut app);
        app.update();
        app
    }

    fn mark_clean(app: &mut App) {
        app.world_mut().resource_mut::<ProfileDirty>().0 = false;
    }

    fn saved_profile(app: &App) -> &PlayerProfile {
        app.world().resource::<PlayerProfile>()
    }

    #[test]
    fn changing_the_loadout_marks_the_profile_dirty() {
        let app = capture_after(|app| {
            app.world_mut().resource_mut::<Loadout>().weapon = WeaponKind::RustySpear;
        });
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(saved_profile(&app).loadout.weapon, WeaponKind::RustySpear);
    }

    #[test]
    fn changing_progress_marks_the_profile_dirty() {
        let app = capture_after(|app| {
            app.world_mut().resource_mut::<WorldProgress>().boss_defeated_floor_1 = true;
        });
        assert!(app.world().resource::<ProfileDirty>().0);
        assert!(saved_profile(&app).progress.boss_defeated_floor_1);
    }

    #[test]
    fn changing_tool_energy_marks_the_profile_dirty() {
        let app = capture_after(|app| {
            app.world_mut().resource_mut::<ToolEnergy>().current = 3.0;
        });
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(saved_profile(&app).tool_energy, 3.0);
    }

    #[test]
    fn changing_the_wallet_marks_the_profile_dirty() {
        let app = capture_after(|app| {
            let mut wallet = app.world_mut().resource_mut::<Wallet>();
            wallet.gold = 9;
            wallet.sparks = DestinySpark(2);
        });
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(saved_profile(&app).wallet.gold, 9);
        assert_eq!(saved_profile(&app).wallet.sparks.amount(), 2);
    }

    #[test]
    fn changing_audio_marks_the_profile_dirty() {
        let app = capture_after(|app| {
            app.world_mut().resource_mut::<AudioSettings>().music_enabled = false;
        });
        assert!(app.world().resource::<ProfileDirty>().0);
        assert!(!saved_profile(&app).settings.audio.music_enabled);
    }

    #[test]
    fn changing_bindings_marks_the_profile_dirty() {
        let app = capture_after(|app| {
            app.world_mut().resource_mut::<SkillBindings>().slots[0] = None;
        });
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(saved_profile(&app).settings.skill_bindings[0], None);
    }

    #[test]
    fn a_day_clock_change_alone_leaves_the_profile_clean() {
        let mut app = runtime_app(Wallet::default());
        app.add_systems(Update, capture_profile_from_runtime);
        app.update();
        mark_clean(&mut app);
        app.world_mut().resource_mut::<DayClock>().calendar_day = 9;
        app.update();
        assert!(!app.world().resource::<ProfileDirty>().0);
        assert_eq!(saved_profile(&app).calendar_day, 1);
    }

    #[test]
    fn editing_the_saved_profile_without_a_runtime_change_leaves_the_profile_clean() {
        let mut app = runtime_app(Wallet::default());
        app.add_systems(Update, capture_profile_from_runtime);
        app.update();
        mark_clean(&mut app);
        app.world_mut().resource_mut::<PlayerProfile>().wallet.gold = 3;
        app.update();
        assert!(!app.world().resource::<ProfileDirty>().0);
        assert_eq!(saved_profile(&app).wallet.gold, 3);
    }

    #[test]
    fn hydrate_fails_when_the_wallet_is_missing() {
        let mut app = app_holding(PlayerProfile::default(), Wallet::default());
        app.world_mut().remove_resource::<Wallet>();
        app.add_systems(Update, hydrate_runtime_from_memory);
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| app.update()))
            .expect_err("hydrate panics");
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .unwrap();
        assert!(message.contains("Wallet"));
    }
}
