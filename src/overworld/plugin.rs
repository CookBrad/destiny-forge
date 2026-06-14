use bevy::prelude::*;

use crate::core::GameState;
use crate::graphics::update_hub_player_sprite;

use super::hub::{cleanup_hub, enter_dungeon, hub_movement, setup_hub};

pub struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Hub), setup_hub)
            .add_systems(OnExit(GameState::Hub), cleanup_hub)
            .add_systems(
                Update,
                (hub_movement, update_hub_player_sprite, enter_dungeon)
                    .chain()
                    .run_if(in_state(GameState::Hub)),
            );
    }
}