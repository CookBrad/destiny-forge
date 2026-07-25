use bevy::prelude::*;

use crate::core::GameState;
use crate::ui::forge_window::forge_closed;
use crate::ui::inventory_window::inventory_closed;

use super::use_pickaxe::use_pickaxe_system;

pub struct MiningPlugin;

impl Plugin for MiningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            use_pickaxe_system
                .run_if(in_state(GameState::Overworld))
                .run_if(inventory_closed)
                .run_if(forge_closed),
        );
    }
}
