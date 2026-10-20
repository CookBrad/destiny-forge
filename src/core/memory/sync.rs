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
    use crate::combat::WeaponKind;
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

    #[derive(Resource, Default, Clone, Copy)]
    struct SeenCapture {
        gold: u32,
        sparks: u32,
        day: u32,
        saved_value_changed: bool,
    }

    fn read_capture(sources: ProfileCaptureInputs, mut seen: ResMut<SeenCapture>) {
        let captured = sources.capture();
        seen.gold = captured.wallet.gold;
        seen.sparks = captured.wallet.sparks.amount();
        seen.day = captured.day_clock.calendar_day;
        seen.saved_value_changed = sources.any_saved_value_changed_except_day_clock();
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

    fn saved_profile(app: &App) -> &PlayerProfile {
        app.world().resource::<PlayerProfile>()
    }

    fn mark_clean(app: &mut App) {
        app.world_mut().resource_mut::<ProfileDirty>().0 = false;
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

    #[test]
    fn capture_reads_gold_and_sparks_from_the_runtime() {
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
    fn capture_reads_the_largest_gold_and_spark_amounts() {
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

    #[test]
    fn capture_fails_when_the_wallet_is_missing() {
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
            .insert_resource(ProfileDirty::default())
            .add_systems(Update, capture_profile_from_runtime);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| app.update()));
        assert!(result.is_err());
    }

    #[test]
    fn each_saved_runtime_value_marks_the_profile_dirty_and_snapshots_that_value() {
        let mut app = runtime_app(Wallet::default());
        app.add_systems(Update, capture_profile_from_runtime);
        app.update();
        mark_clean(&mut app);

        app.world_mut().resource_mut::<Inventory>().slots[0].count = 4;
        app.update();
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(saved_profile(&app).inventory.slots[0].count, 4);
        mark_clean(&mut app);

        app.world_mut().resource_mut::<Loadout>().weapon = WeaponKind::RustySpear;
        app.update();
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(saved_profile(&app).loadout.weapon, WeaponKind::RustySpear);
        mark_clean(&mut app);

        app.world_mut().resource_mut::<WorldProgress>().boss_defeated_floor_1 = true;
        app.update();
        assert!(app.world().resource::<ProfileDirty>().0);
        assert!(saved_profile(&app).progress.boss_defeated_floor_1);
        mark_clean(&mut app);

        app.world_mut().resource_mut::<ToolEnergy>().current = 3.0;
        app.update();
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(saved_profile(&app).tool_energy, 3.0);
        mark_clean(&mut app);

        {
            let mut wallet = app.world_mut().resource_mut::<Wallet>();
            wallet.gold = 9;
            wallet.sparks = DestinySpark(2);
        }
        app.update();
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(saved_profile(&app).wallet.gold, 9);
        assert_eq!(saved_profile(&app).wallet.sparks.amount(), 2);
        mark_clean(&mut app);

        app.world_mut().resource_mut::<AudioSettings>().music_enabled = false;
        app.update();
        assert!(app.world().resource::<ProfileDirty>().0);
        assert!(!saved_profile(&app).settings.audio.music_enabled);
        mark_clean(&mut app);

        app.world_mut().resource_mut::<SkillBindings>().slots[0] = None;
        app.update();
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
}
