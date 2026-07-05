use bevy::prelude::*;

use crate::combat::SkillBindings;
use crate::core::{DungeonPlayState, GameState};
use crate::dungeon::move_enemies;

use super::health_bars::{
    cleanup_health_bars, despawn_orphan_enemy_health_bars, setup_health_bar_assets,
    spawn_enemy_health_bars, spawn_player_health_bar, update_enemy_health_bars,
    update_player_health_bar, HealthBarAssets,
};
use super::menu::{
    cleanup_death_menu, cleanup_pause_menu, cleanup_title_menu, death_menu_input,
    ensure_time_running, open_pause_menu, pause_game_time, pause_menu_input, resume_game_time,
    spawn_death_menu, spawn_pause_menu, spawn_title_menu, title_input,
};
use super::pause_audio::{handle_pause_audio_input, sync_pause_audio_display};
use super::skill_bar::{
    cleanup_skill_bar, handle_skill_bar_drag, spawn_skill_bar, sync_skill_bar, SkillBarDrag,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HealthBarAssets>()
            .init_resource::<SkillBindings>()
            .init_resource::<SkillBarDrag>()
            .add_systems(Startup, setup_health_bar_assets)
            .add_systems(
                OnEnter(GameState::Title),
                (
                    ensure_time_running,
                    cleanup_pause_menu,
                    cleanup_death_menu,
                    cleanup_skill_bar,
                    spawn_title_menu,
                )
                    .chain(),
            )
            .add_systems(OnExit(GameState::Title), cleanup_title_menu)
            .add_systems(Update, title_input.run_if(in_state(GameState::Title)))
            .add_systems(
                OnEnter(DungeonPlayState::Paused),
                (pause_game_time, spawn_pause_menu),
            )
            .add_systems(
                OnExit(DungeonPlayState::Paused),
                (resume_game_time, cleanup_pause_menu),
            )
            .add_systems(
                Update,
                (
                    open_pause_menu.run_if(in_state(DungeonPlayState::Running)),
                    pause_menu_input.run_if(in_state(DungeonPlayState::Paused)),
                    (
                        handle_pause_audio_input,
                        sync_pause_audio_display,
                    )
                        .chain()
                        .run_if(in_state(DungeonPlayState::Paused)),
                    death_menu_input.run_if(in_state(DungeonPlayState::Dead)),
                )
                    .run_if(in_state(GameState::Dungeon)),
            )
            .add_systems(
                OnEnter(GameState::Dungeon),
                (
                    setup_health_bar_assets,
                    spawn_skill_bar,
                    spawn_player_health_bar,
                    spawn_enemy_health_bars,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(DungeonPlayState::Dead),
                (pause_game_time, spawn_death_menu),
            )
            .add_systems(
                OnExit(DungeonPlayState::Dead),
                (resume_game_time, cleanup_death_menu),
            )
            .add_systems(
                OnExit(GameState::Dungeon),
                (
                    ensure_time_running,
                    cleanup_pause_menu,
                    cleanup_death_menu,
                    cleanup_skill_bar,
                    cleanup_health_bars,
                ),
            )
            .add_systems(
                Update,
                (
                    handle_skill_bar_drag,
                    sync_skill_bar,
                    update_player_health_bar,
                    despawn_orphan_enemy_health_bars,
                    spawn_enemy_health_bars,
                    update_enemy_health_bars.after(move_enemies),
                )
                    .run_if(in_state(GameState::Dungeon))
                    .run_if(
                        in_state(DungeonPlayState::Running)
                            .or(in_state(DungeonPlayState::Paused)),
                    ),
            );
    }
}