use bevy::prelude::*;

use crate::core::GameState;
use crate::overworld::camera::{follow_exploration_camera, init_exploration_camera};
use crate::overworld::movement::{
    animate_overworld_player, exploration_movement, tick_map_transition_cooldown,
};
use crate::ui::inventory_window::inventory_closed;

use super::interaction::{mine_interaction, swing_at_ore_node, update_mine_interaction_prompt};
use super::setup::{cleanup_mine, set_mine_clear_color, setup_mine};

pub struct MiningPlugin;

impl Plugin for MiningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Mine),
            (set_mine_clear_color, setup_mine, init_exploration_camera).chain(),
        )
        .add_systems(OnExit(GameState::Mine), cleanup_mine)
        .add_systems(
            Update,
            (
                exploration_movement,
                tick_map_transition_cooldown,
                animate_overworld_player,
                follow_exploration_camera,
                swing_at_ore_node,
                mine_interaction,
            )
                .chain()
                .run_if(in_state(GameState::Mine))
                .run_if(inventory_closed),
        )
        .add_systems(
            Update,
            update_mine_interaction_prompt.run_if(in_state(GameState::Mine)),
        );
    }
}
