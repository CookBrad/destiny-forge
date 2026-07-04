use bevy::prelude::*;

use crate::core::{DungeonPlayState, GameState};

use super::music::{
    pause_dungeon_music, resume_dungeon_music, start_dungeon_music, stop_dungeon_music,
};

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Dungeon), start_dungeon_music)
            .add_systems(OnExit(GameState::Dungeon), stop_dungeon_music)
            .add_systems(OnEnter(DungeonPlayState::Paused), pause_dungeon_music)
            .add_systems(OnEnter(DungeonPlayState::Dead), pause_dungeon_music)
            .add_systems(OnExit(DungeonPlayState::Paused), resume_dungeon_music)
            .add_systems(OnExit(DungeonPlayState::Dead), resume_dungeon_music);
    }
}