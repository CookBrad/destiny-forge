use bevy::prelude::*;

use crate::core::GameState;
use crate::ui::forge_window::forge_closed;
use crate::ui::inventory_window::inventory_closed;

use super::hotbar::HomesteadHotbar;
use super::plots::sync_plot_visuals;
use super::tools::EquippedTool;
use super::use_tool::{ensure_starter_seeds, update_player_facing, use_homestead_tool};

pub struct FarmingPlugin;

impl Plugin for FarmingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EquippedTool>()
            .init_resource::<HomesteadHotbar>()
            .add_systems(OnEnter(GameState::Overworld), ensure_starter_seeds)
            .add_systems(
                Update,
                (
                    update_player_facing,
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
