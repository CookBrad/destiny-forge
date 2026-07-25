use bevy::prelude::*;

use crate::core::GameState;

use super::cast::{
    clear_cast_on_overworld_exit, tick_active_cast, use_fishing_rod_system, ActiveCast,
};
use super::ui::{cleanup_fishing_bar, sync_fishing_bar_ui};

pub struct FishingPlugin;

impl Plugin for FishingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveCast>()
            .add_systems(
                Update,
                (
                    // Tick result timer even if menus open so cast never soft-locks.
                    tick_active_cast,
                    use_fishing_rod_system,
                    sync_fishing_bar_ui,
                )
                    .chain()
                    .run_if(in_state(GameState::Overworld)),
            )
            .add_systems(
                OnExit(GameState::Overworld),
                (clear_cast_on_overworld_exit, cleanup_fishing_bar),
            );
    }
}
