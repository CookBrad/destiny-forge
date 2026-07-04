use bevy::prelude::*;

use super::music::{DungeonMusic, DUNGEON_MUSIC_BASE_VOLUME};

#[derive(Resource, Clone)]
pub struct AudioSettings {
    pub music_enabled: bool,
    pub music_volume: f32,
    pub sfx_enabled: bool,
    pub sfx_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            music_enabled: true,
            music_volume: 1.0,
            sfx_enabled: true,
            sfx_volume: 1.0,
        }
    }
}

impl AudioSettings {
    pub fn music_gain(&self) -> f32 {
        if self.music_enabled {
            self.music_volume.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    pub fn sfx_gain(&self) -> f32 {
        if self.sfx_enabled {
            self.sfx_volume.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

pub fn apply_music_volume(
    settings: Res<AudioSettings>,
    music: Query<&bevy::audio::AudioSink, With<DungeonMusic>>,
) {
    if !settings.is_changed() {
        return;
    }

    let volume = settings.music_gain() * DUNGEON_MUSIC_BASE_VOLUME;
    for sink in &music {
        sink.set_volume(volume);
    }
}