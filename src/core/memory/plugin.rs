use bevy::prelude::*;

use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

use super::autosave::{
    debounced_autosave, flush_saves_on_exit, queue_autosave, AutosaveTimer, ProfileDirty,
};
use super::autosave_block::{load_profile_for_slot, ProfileAutosaveBlock};
use super::header::{
    accrue_playtime, mark_profile_dirty_when_a_second_is_banked, BankedPlaytimeSeconds,
};
use super::profile::{ActiveProfile, PlayerProfile};
use super::storage::load_settings;
use super::sync::{capture_profile_from_runtime, hydrate_runtime_from_memory};

pub struct MemoryPlugin;

impl Plugin for MemoryPlugin {
    fn build(&self, app: &mut App) {
        load_save_folder_unless_a_player_profile_is_already_present(app);
        register_memory_systems_leaving_day_clock_and_tool_energy_for_core_plugin(app);
    }
}

fn load_save_folder_unless_a_player_profile_is_already_present(app: &mut App) {
    if app.world().contains_resource::<PlayerProfile>() {
        return;
    }
    insert_resources_from_the_save_folder(app);
}

fn insert_resources_from_the_save_folder(app: &mut App) {
    let settings = load_settings();
    let active = ActiveProfile(settings.last_active_profile);
    let mut autosave_block = ProfileAutosaveBlock::default();
    let profile = load_profile_for_slot(active.index(), &mut autosave_block)
        .unwrap_or_else(|_| runtime_profile_while_save_is_blocked());

    app.insert_resource(autosave_block)
        .insert_resource(settings)
        .insert_resource(active)
        .insert_resource(profile)
        .insert_resource(Inventory::default())
        .insert_resource(Loadout::default())
        .insert_resource(WorldProgress::default());
}

fn runtime_profile_while_save_is_blocked() -> PlayerProfile {
    PlayerProfile::default()
}

fn register_memory_systems_leaving_day_clock_and_tool_energy_for_core_plugin(app: &mut App) {
    app.init_resource::<ProfileDirty>()
        .init_resource::<AutosaveTimer>()
        .init_resource::<BankedPlaytimeSeconds>()
        .add_systems(Startup, hydrate_runtime_from_memory)
        .add_systems(
            Update,
            (
                accrue_playtime,
                mark_profile_dirty_when_a_second_is_banked,
                capture_profile_from_runtime,
                queue_autosave,
                debounced_autosave,
            )
                .chain(),
        )
        .add_systems(Last, flush_saves_on_exit);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioSettings;
    use crate::combat::SkillBindings;
    use crate::core::day_cycle::{DayClock, ToolEnergy};
    use crate::core::season::{CalendarRules, Season};
    use crate::core::state::GameState;

    use super::super::profile::PROFILE_VERSION;
    use super::super::settings::GameSettings;

    #[test]
    fn memory_plugin_runs_one_update_with_a_blank_header() {
        let mut app = app_with_memory_plugin(PlayerProfile::default());
        app.update();
        let profile = app.world().resource::<PlayerProfile>();
        assert_eq!(profile.header.name, "");
        assert_eq!(profile.header.playtime_seconds, 0);
        assert_eq!(profile.playtime_fraction, 0.0);
        assert!(!app.world().resource::<ProfileDirty>().0);
        assert!(!app.world().contains_resource::<GameSettings>());
    }

    #[test]
    fn memory_plugin_keeps_an_existing_profile_across_one_update() {
        let mut profile = PlayerProfile::default();
        profile.header.name = "Hearth".to_string();
        profile.header.playtime_seconds = 12;
        profile.playtime_fraction = 0.25;
        let mut app = app_with_memory_plugin(profile);
        app.update();
        let stored = app.world().resource::<PlayerProfile>();
        assert_eq!(stored.header.name, "Hearth");
        assert_eq!(stored.header.playtime_seconds, 12);
        assert_eq!(stored.playtime_fraction, 0.25);
        assert!(!app.world().resource::<ProfileDirty>().0);
        assert_eq!(app.world().resource::<BankedPlaytimeSeconds>().0, 0);
        assert!(!app.world().contains_resource::<GameSettings>());
    }

    #[test]
    fn an_existing_player_profile_skips_the_save_folder() {
        let mut app = App::new();
        let mut profile = PlayerProfile::default();
        profile.header.name = "Hearth".to_string();
        profile.header.playtime_seconds = 12;
        profile.playtime_fraction = 0.25;
        app.insert_resource(profile)
            .insert_resource(ActiveProfile(2));
        load_save_folder_unless_a_player_profile_is_already_present(&mut app);
        let stored = app.world().resource::<PlayerProfile>();
        assert_eq!(stored.header.name, "Hearth");
        assert_eq!(stored.header.playtime_seconds, 12);
        assert_eq!(stored.playtime_fraction, 0.25);
        assert_eq!(app.world().resource::<ActiveProfile>().0, 2);
        assert!(!app.world().contains_resource::<GameSettings>());
        assert!(!app.world().contains_resource::<Inventory>());
        assert!(!app.world().contains_resource::<ProfileAutosaveBlock>());
    }

    #[test]
    fn a_blank_player_profile_still_skips_the_save_folder() {
        let mut app = App::new();
        app.insert_resource(PlayerProfile::default());
        load_save_folder_unless_a_player_profile_is_already_present(&mut app);
        let stored = app.world().resource::<PlayerProfile>();
        assert_eq!(stored.header.name, "");
        assert_eq!(stored.header.playtime_seconds, 0);
        assert_eq!(stored.playtime_fraction, 0.0);
        assert!(!app.world().contains_resource::<GameSettings>());
        assert!(!app.world().contains_resource::<Inventory>());
        assert!(!app.world().contains_resource::<Loadout>());
        assert!(!app.world().contains_resource::<WorldProgress>());
    }

    #[test]
    fn a_blocked_save_starts_from_a_fresh_profile() {
        let profile = runtime_profile_while_save_is_blocked();
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.header.name, "");
        assert_eq!(profile.header.playtime_seconds, 0);
        assert_eq!(profile.playtime_fraction, 0.0);
        assert_eq!(profile.header.gold, 0);
        assert!(profile.crop_plots.is_empty());
        assert!(profile.depleted_ore_node_ids.is_empty());
    }

    #[test]
    fn registering_memory_systems_starts_a_clean_profile_without_a_day_clock() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        register_memory_systems_leaving_day_clock_and_tool_energy_for_core_plugin(&mut app);
        assert!(!app.world().resource::<ProfileDirty>().0);
        assert_eq!(app.world().resource::<BankedPlaytimeSeconds>().0, 0);
        assert!(!app.world().contains_resource::<DayClock>());
        assert!(!app.world().contains_resource::<ToolEnergy>());
    }

    #[test]
    fn registering_memory_systems_keeps_a_dirty_profile_dirty() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ProfileDirty(true));
        register_memory_systems_leaving_day_clock_and_tool_energy_for_core_plugin(&mut app);
        assert!(app.world().resource::<ProfileDirty>().0);
        assert_eq!(app.world().resource::<BankedPlaytimeSeconds>().0, 0);
        assert!(!app.world().contains_resource::<DayClock>());
        assert!(!app.world().contains_resource::<ToolEnergy>());
    }

    fn app_with_memory_plugin(profile: PlayerProfile) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(State::new(GameState::Title))
            .insert_resource(ProfileAutosaveBlock::default())
            .insert_resource(ActiveProfile::default())
            .insert_resource(profile)
            .insert_resource(Inventory::default())
            .insert_resource(Loadout::default())
            .insert_resource(WorldProgress::default())
            .init_resource::<AudioSettings>()
            .init_resource::<SkillBindings>()
            .init_resource::<DayClock>()
            .init_resource::<ToolEnergy>()
            .init_resource::<Season>()
            .init_resource::<CalendarRules>()
            .add_plugins(MemoryPlugin);
        app
    }
}
