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
        apply_profile_to_runtime(
            profile,
            &mut inventory,
            &mut loadout,
            &mut progress,
            day_clock,
            &mut tool_energy,
            &mut audio,
            &mut bindings,
            season,
            &CalendarRules::load(),
            wallet,
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
            &inventory,
            &loadout,
            &progress,
            &day_clock,
            &tool_energy,
            &audio,
            &bindings,
            &wallet,
            &mut active,
            profile,
            &mut global,
            &mut dirty,
            autosave_block,
        );
        (started, active)
    }
}
