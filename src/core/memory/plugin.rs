use bevy::prelude::*;

use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

use super::autosave::{debounced_autosave, flush_saves_on_exit, queue_autosave, ProfileDirty};
use super::autosave_block::{load_profile_for_slot, ProfileAutosaveBlock};
use super::header::{
    accrue_playtime, mark_profile_dirty_when_a_second_is_banked, remember_current_location,
    BankedPlaytimeSeconds,
};
use super::profile::ActiveProfile;
use super::storage::load_settings;
use super::sync::hydrate_runtime_from_memory;

fn runtime_profile_while_save_is_blocked() -> super::profile::PlayerProfile {
    super::profile::PlayerProfile::default()
}

pub struct MemoryPlugin;

impl Plugin for MemoryPlugin {
    fn build(&self, app: &mut App) {
        let global = load_settings();
        let active = ActiveProfile(global.last_active_profile);
        let mut autosave_block = ProfileAutosaveBlock::default();
        let profile = load_profile_for_slot(active.index(), &mut autosave_block)
            .unwrap_or_else(|_| runtime_profile_while_save_is_blocked());

        app.insert_resource(autosave_block)
            .insert_resource(global)
            .insert_resource(active)
            .insert_resource(profile)
            .insert_resource(Inventory::default())
            .insert_resource(Loadout::default())
            .insert_resource(WorldProgress::default())
            // DayClock / ToolEnergy are init'd in CorePlugin (after MemoryPlugin wiring).
            .init_resource::<ProfileDirty>()
            .init_resource::<super::autosave::AutosaveTimer>()
            .init_resource::<BankedPlaytimeSeconds>()
            .add_systems(Startup, hydrate_runtime_from_memory)
            .add_systems(
                Update,
                (
                    accrue_playtime,
                    mark_profile_dirty_when_a_second_is_banked,
                    remember_current_location,
                    super::sync::capture_profile_from_runtime,
                    queue_autosave,
                    debounced_autosave,
                )
                    .chain(),
            )
            .add_systems(Last, flush_saves_on_exit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioSettings;
    use crate::combat::SkillBindings;
    use crate::core::day_cycle::{DayClock, ToolEnergy};
    use crate::core::season::{CalendarRules, Season};
    use crate::core::state::GameState;

    #[test]
    fn memory_plugin_runs_one_update_with_a_blank_header() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(State::new(GameState::Title))
            .init_resource::<AudioSettings>()
            .init_resource::<SkillBindings>()
            .init_resource::<DayClock>()
            .init_resource::<ToolEnergy>()
            .init_resource::<Season>()
            .init_resource::<CalendarRules>()
            .add_plugins(MemoryPlugin);
        app.update();
        let profile = app
            .world()
            .resource::<super::super::profile::PlayerProfile>();
        assert_eq!(profile.header.playtime_seconds, 0);
        assert_eq!(profile.playtime_fraction, 0.0);
        assert!(!app.world().resource::<ProfileDirty>().0);
    }
}
