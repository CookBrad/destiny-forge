use bevy::prelude::*;

use crate::core::GameState;
use crate::overworld::setup::{cleanup_overworld, setup_overworld};
use crate::ui::forge_window::forge_closed;
use crate::ui::inventory_window::inventory_closed;

use super::hud::{cleanup_tool_hud, setup_tool_hud, sync_tool_hud};
use super::plots::sync_plot_visuals;
use super::select_action_bar_slot::{
    cycle_action_bar_slot_from_mouse_wheel, select_action_bar_slot_from_number_keys,
};
use super::tools::EquippedTool;
use super::use_tool::{
    capture_crop_plots_on_exit, ensure_starter_kit, update_player_facing, use_homestead_tool,
};
use super::ActionBar;

pub struct FarmingPlugin;

impl Plugin for FarmingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EquippedTool>()
            .init_resource::<ActionBar>()
            .add_systems(
                OnEnter(GameState::Overworld),
                (ensure_starter_kit, setup_tool_hud).after(setup_overworld),
            )
            .add_systems(
                OnExit(GameState::Overworld),
                (
                    capture_crop_plots_on_exit.before(cleanup_overworld),
                    cleanup_tool_hud,
                ),
            )
            .add_systems(
                Update,
                (
                    select_action_bar_slot_from_number_keys,
                    cycle_action_bar_slot_from_mouse_wheel,
                    update_player_facing,
                    use_homestead_tool,
                    sync_plot_visuals,
                    sync_tool_hud,
                )
                    .chain()
                    .run_if(in_state(GameState::Overworld))
                    .run_if(inventory_closed)
                    .run_if(forge_closed),
            );
    }
}
