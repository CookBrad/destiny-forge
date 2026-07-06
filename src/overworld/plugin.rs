use bevy::prelude::*;

use crate::core::GameState;

use super::animals::move_farm_animals;
use super::camera::{follow_overworld_camera, init_overworld_camera};
use super::interaction::overworld_interaction;
use super::movement::{animate_overworld_player, overworld_movement};
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
                init_overworld_camera,
                spawn_overworld_hud,
            )
                .chain(),
        )
        .add_systems(OnExit(GameState::Overworld), cleanup_overworld)
        .add_systems(
            Update,
            (
                overworld_movement,
                animate_overworld_player,
                move_farm_animals,
                follow_overworld_camera,
                overworld_interaction,
            )
                .chain()
                .run_if(in_state(GameState::Overworld)),
        );
    }
}