use bevy::prelude::*;

use crate::core::GameState;
use crate::ui::inventory_window::inventory_closed;
use crate::overworld::camera::{follow_exploration_camera, init_exploration_camera};
use crate::overworld::movement::{
    animate_overworld_player, exploration_movement, tick_map_transition_cooldown,
};
use crate::overworld::setup::spawn_overworld_hud;

use super::interaction::forest_interaction;
use super::setup::{cleanup_forest, setup_forest};

fn set_forest_clear_color(mut clear: ResMut<ClearColor>) {
    clear.0 = Color::srgb(0.12, 0.2, 0.1);
}

pub struct ForestPlugin;

impl Plugin for ForestPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Forest),
            (
                set_forest_clear_color,
                setup_forest,
                init_exploration_camera,
                spawn_overworld_hud,
            )
                .chain(),
        )
        .add_systems(OnExit(GameState::Forest), cleanup_forest)
        .add_systems(
            Update,
            (
                exploration_movement,
                tick_map_transition_cooldown,
                animate_overworld_player,
                follow_exploration_camera,
                forest_interaction,
            )
                .chain()
                .run_if(in_state(GameState::Forest))
                .run_if(inventory_closed),
        );
    }
}