use bevy::prelude::*;

use crate::core::{DungeonPlayState, DungeonUiTeardown, GameState};

use super::music::{
    pause_dungeon_music, resume_dungeon_music, start_dungeon_music, stop_dungeon_music,
};
use super::settings::{apply_music_volume, AudioSettings};
use super::sfx::{play_combat_sfx, setup_combat_sfx, CombatSfxAssets};

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioSettings>()
            .init_resource::<CombatSfxAssets>()
            .add_event::<super::sfx::CombatSfx>()
            .add_systems(OnEnter(GameState::Dungeon), (setup_combat_sfx, start_dungeon_music).chain())
            .add_systems(
                OnExit(GameState::Dungeon),
                stop_dungeon_music.after(DungeonUiTeardown),
            )
            .add_systems(OnEnter(DungeonPlayState::Paused), pause_dungeon_music)
            .add_systems(OnEnter(DungeonPlayState::Dead), pause_dungeon_music)
            .add_systems(OnEnter(DungeonPlayState::Running), resume_dungeon_music)
            .add_systems(
                Update,
                apply_music_volume.run_if(in_state(GameState::Dungeon)),
            )
            .add_systems(
                Update,
                play_combat_sfx
                    .run_if(in_state(GameState::Dungeon))
                    .run_if(in_state(DungeonPlayState::Running)),
            );
    }
}