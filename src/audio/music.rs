use bevy::audio::{AudioPlayer, AudioSink, PlaybackSettings, Volume};
use bevy::prelude::*;

#[derive(Component)]
pub struct DungeonMusic;

pub const DUNGEON_MUSIC_BASE_VOLUME: f32 = 0.38;

const DUNGEON_MUSIC_PATH: &str = "audio/dungeon_music.mp3";

pub fn start_dungeon_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<crate::audio::AudioSettings>,
    existing: Query<Entity, With<DungeonMusic>>,
) {
    if !existing.is_empty() {
        return;
    }

    let volume = settings.music_gain() * DUNGEON_MUSIC_BASE_VOLUME;
    commands.spawn((
        AudioPlayer::new(asset_server.load(DUNGEON_MUSIC_PATH)),
        PlaybackSettings::LOOP.with_volume(Volume::new(volume)),
        DungeonMusic,
    ));
}

pub fn stop_dungeon_music(mut commands: Commands, music: Query<Entity, With<DungeonMusic>>) {
    for entity in &music {
        commands.entity(entity).try_despawn();
    }
}

pub fn pause_dungeon_music(music: Query<&AudioSink, With<DungeonMusic>>) {
    for sink in &music {
        sink.pause();
    }
}

pub fn resume_dungeon_music(music: Query<&AudioSink, With<DungeonMusic>>) {
    for sink in &music {
        if sink.is_paused() {
            sink.play();
        }
    }
}