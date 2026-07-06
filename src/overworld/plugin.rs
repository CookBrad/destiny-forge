use bevy::prelude::*;

use crate::core::GameState;

use super::camera::{follow_overworld_camera, init_overworld_camera};
use super::interaction::overworld_interaction;
use super::movement::{animate_overworld_player, overworld_movement};
use super::setup::{cleanup_overworld, setup_overworld, spawn_overworld_hud};

fn set_overworld_clear_color(mut clear: ResMut<ClearColor>) {
    clear.0 = Color::srgb(0.45, 0.62, 0.38);
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
                follow_overworld_camera,
                overworld_interaction,
            )
                .chain()
                .run_if(in_state(GameState::Overworld)),
        );
    }
}