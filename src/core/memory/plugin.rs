use bevy::prelude::*;

use crate::items::Inventory;
use crate::player::{Loadout, Wallet, WorldProgress};

use super::autosave::{debounced_autosave, flush_saves_on_exit, queue_autosave, ProfileDirty};
use super::autosave_block::{load_profile_for_slot, ProfileAutosaveBlock};
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
            .insert_resource(Wallet::default())
            // DayClock / ToolEnergy are init'd in CorePlugin (after MemoryPlugin wiring).
            .init_resource::<ProfileDirty>()
            .init_resource::<super::autosave::AutosaveTimer>()
            .add_systems(Startup, hydrate_runtime_from_memory)
            .add_systems(
                Update,
                (
                    super::sync::capture_profile_from_runtime,
                    queue_autosave,
                    debounced_autosave,
                )
                    .chain(),
            )
            .add_systems(Last, flush_saves_on_exit);
    }
}
