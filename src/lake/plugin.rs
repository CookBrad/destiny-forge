use bevy::prelude::*;

use crate::core::GameState;
use crate::ui::inventory_window::inventory_closed;
use crate::overworld::camera::{follow_exploration_camera, init_exploration_camera};
use crate::overworld::movement::{
    animate_overworld_player, exploration_movement, tick_map_transition_cooldown,
};

use super::interaction::{lake_interaction, update_lake_interaction_prompt};
use super::setup::{cleanup_lake, setup_lake};

fn set_lake_clear_color(mut clear: ResMut<ClearColor>) {
    clear.0 = Color::srgb(0.1, 0.18, 0.28);
}

pub struct LakePlugin;

impl Plugin for LakePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Lake),
            (set_lake_clear_color, setup_lake, init_exploration_camera).chain(),
        )
        .add_systems(OnExit(GameState::Lake), cleanup_lake)
        .add_systems(
            Update,
            (
                exploration_movement,
                tick_map_transition_cooldown,
                animate_overworld_player,
                follow_exploration_camera,
                lake_interaction,
                update_lake_interaction_prompt,
            )
                .chain()
                .run_if(in_state(GameState::Lake))
                .run_if(inventory_closed),
        );
    }
}
