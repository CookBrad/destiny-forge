use bevy::prelude::*;

use crate::core::GameState;
use crate::ui::forge_window::forge_closed;
use crate::ui::inventory_window::inventory_closed;

use super::cast::{tick_active_cast, use_fishing_rod_system, ActiveCast};

pub struct FishingPlugin;

impl Plugin for FishingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveCast>().add_systems(
            Update,
            (tick_active_cast, use_fishing_rod_system)
                .chain()
                .run_if(in_state(GameState::Overworld))
                .run_if(inventory_closed)
                .run_if(forge_closed),
        );
    }
}
