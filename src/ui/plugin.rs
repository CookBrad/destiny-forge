use bevy::prelude::*;

use crate::core::GameState;

use super::controls::{cleanup_controls_help, spawn_controls_help, update_controls_help};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Dungeon), spawn_controls_help)
            .add_systems(OnExit(GameState::Dungeon), cleanup_controls_help)
            .add_systems(
                Update,
                update_controls_help.run_if(in_state(GameState::Dungeon)),
            );
    }
}