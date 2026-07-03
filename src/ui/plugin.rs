use bevy::prelude::*;

use crate::core::GameState;
use crate::dungeon::move_enemies;

use super::controls::{cleanup_controls_help, spawn_controls_help, update_controls_help};
use super::health_bars::{
    cleanup_health_bars, setup_health_bar_assets, spawn_enemy_health_bars, spawn_player_health_bar,
    update_enemy_health_bars, update_player_health_bar, HealthBarAssets,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HealthBarAssets>()
            .add_systems(Startup, setup_health_bar_assets)
            .add_systems(
                OnEnter(GameState::Dungeon),
                (
                    setup_health_bar_assets,
                    spawn_controls_help,
                    spawn_player_health_bar,
                    spawn_enemy_health_bars,
                )
                    .chain(),
            )
            .add_systems(
                OnExit(GameState::Dungeon),
                (cleanup_controls_help, cleanup_health_bars),
            )
            .add_systems(
                Update,
                (
                    update_controls_help,
                    update_player_health_bar,
                    update_enemy_health_bars.after(move_enemies),
                )
                    .run_if(in_state(GameState::Dungeon)),
            );
    }
}