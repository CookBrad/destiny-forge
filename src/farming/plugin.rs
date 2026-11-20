use bevy::prelude::*;

use crate::core::GameState;
use crate::overworld::setup::{cleanup_overworld, setup_overworld};
use crate::ui::forge_window::forge_closed;
use crate::ui::inventory_window::{inventory_closed, inventory_window_open};

use super::action_bar_art::{
    cleanup_action_bar_hud, load_action_bar_art, setup_action_bar_hud, sync_action_bar_highlight,
    sync_action_bar_slot_contents,
};
use super::action_bar_drag::{
    clear_action_bar_drag, release_action_bar_drag, start_action_bar_drag,
    update_action_bar_drag_ghost, ActionBarDrag,
};
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
            .init_resource::<ActionBarDrag>()
            .add_systems(
                OnEnter(GameState::Overworld),
                (
                    ensure_starter_kit,
                    load_action_bar_art,
                    setup_action_bar_hud,
                    setup_tool_hud,
                )
                    .chain()
                    .after(setup_overworld),
            )
            .add_systems(
                OnExit(GameState::Overworld),
                (
                    clear_action_bar_drag,
                    capture_crop_plots_on_exit.before(cleanup_overworld),
                    cleanup_tool_hud,
                    cleanup_action_bar_hud,
                ),
            )
            .add_systems(
                Update,
                (
                    select_action_bar_slot_from_number_keys,
                    cycle_action_bar_slot_from_mouse_wheel,
                    sync_action_bar_highlight,
                    sync_action_bar_slot_contents,
                    update_player_facing,
                    use_homestead_tool,
                    sync_plot_visuals,
                    sync_tool_hud,
                )
                    .chain()
                    .run_if(in_state(GameState::Overworld))
                    .run_if(inventory_closed)
                    .run_if(forge_closed),
            )
            .add_systems(
                Update,
                (
                    start_action_bar_drag,
                    update_action_bar_drag_ghost.after(start_action_bar_drag),
                    release_action_bar_drag.after(update_action_bar_drag_ghost),
                    sync_action_bar_slot_contents.after(release_action_bar_drag),
                )
                    .run_if(in_state(GameState::Overworld))
                    .run_if(inventory_window_open),
            )
            .add_systems(
                Update,
                clear_action_bar_drag
                    .run_if(in_state(GameState::Overworld))
                    .run_if(inventory_closed),
            );
    }
}
