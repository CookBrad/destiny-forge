use bevy::prelude::*;

use crate::core::GameState;
use crate::ui::inventory_window::inventory_closed;

use super::animals::move_farm_animals;
use super::camera::{follow_exploration_camera, init_exploration_camera};
use super::interaction::overworld_interaction;
use super::movement::{animate_overworld_player, exploration_movement, tick_map_transition_cooldown};
use super::setup::{cleanup_overworld, setup_overworld, spawn_overworld_hud};

fn set_overworld_clear_color(mut clear: ResMut<ClearColor>) {
    clear.0 = Color::srgb(0.22, 0.28, 0.18);
}

pub struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Overworld),
            (
                set_overworld_clear_color,
                setup_overworld,
                init_exploration_camera,
                spawn_overworld_hud,
            )
                .chain(),
        )
        .add_systems(OnExit(GameState::Overworld), cleanup_overworld)
        .add_systems(
            Update,
            (
                exploration_movement,
                tick_map_transition_cooldown,
                animate_overworld_player,
                move_farm_animals,
                follow_exploration_camera,
                overworld_interaction,
            )
                .chain()
                .run_if(in_state(GameState::Overworld))
                .run_if(inventory_closed),
        );
    }
}