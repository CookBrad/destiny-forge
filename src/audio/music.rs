use bevy::audio::{AudioPlayer, AudioSink, PlaybackSettings, Volume};
use bevy::prelude::*;

use crate::core::{DungeonPlayState, GameState};

#[derive(Component)]
pub struct DungeonMusic;

const DUNGEON_MUSIC_PATH: &str = "audio/dungeon_ambient.ogg";
const DUNGEON_MUSIC_VOLUME: f32 = 0.42;

pub fn start_dungeon_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<DungeonMusic>>,
) {
    if !existing.is_empty() {
        return;
    }

    commands.spawn((
        AudioPlayer::new(asset_server.load(DUNGEON_MUSIC_PATH)),
        PlaybackSettings::LOOP.with_volume(Volume::new(DUNGEON_MUSIC_VOLUME)),
        DungeonMusic,
    ));
}

pub fn stop_dungeon_music(mut commands: Commands, music: Query<Entity, With<DungeonMusic>>) {
    for entity in &music {
        commands.entity(entity).despawn();
    }
}

pub fn pause_dungeon_music(music: Query<&AudioSink, With<DungeonMusic>>) {
    for sink in &music {
        sink.pause();
    }
}

pub fn resume_dungeon_music(
    game_state: Res<State<GameState>>,
    play_state: Res<State<DungeonPlayState>>,
    music: Query<&AudioSink, With<DungeonMusic>>,
) {
    if *game_state.get() != GameState::Dungeon || *play_state.get() != DungeonPlayState::Running {
        return;
    }

    for sink in &music {
        if sink.is_paused() {
            sink.play();
        }
    }
}