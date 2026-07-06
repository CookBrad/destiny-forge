use bevy::prelude::*;

use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

use super::autosave::{debounced_autosave, flush_saves_on_exit, queue_autosave, ProfileDirty};
use super::profile::ActiveProfile;
use super::storage::{load_profile, load_settings};
use super::sync::hydrate_runtime_from_memory;

pub struct MemoryPlugin;

impl Plugin for MemoryPlugin {
    fn build(&self, app: &mut App) {
        let global = load_settings();
        let active = ActiveProfile(global.last_active_profile);
        let profile = load_profile(active.index());

        app.insert_resource(global)
            .insert_resource(active)
            .insert_resource(profile)
            .insert_resource(Inventory::default())
            .insert_resource(Loadout::default())
            .insert_resource(WorldProgress::default())
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