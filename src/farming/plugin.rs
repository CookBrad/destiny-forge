use bevy::prelude::*;

use crate::core::GameState;
use crate::ui::forge_window::forge_closed;
use crate::ui::inventory_window::inventory_closed;

use super::plots::sync_plot_visuals;
use super::tools::EquippedTool;
use super::use_tool::{select_homestead_tool, update_player_facing, use_homestead_tool};

pub struct FarmingPlugin;

impl Plugin for FarmingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EquippedTool>().add_systems(
            Update,
            (
                update_player_facing,
                select_homestead_tool,
                use_homestead_tool,
                sync_plot_visuals,
            )
                .chain()
                .run_if(in_state(GameState::Overworld))
                .run_if(inventory_closed)
                .run_if(forge_closed),
        );
    }
}
