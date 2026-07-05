use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::audio::AudioSettings;
use crate::combat::{SkillBindings, SkillKind, SKILL_SLOT_COUNT};

pub const SETTINGS_VERSION: u32 = 2;

/// Machine-wide preferences (not tied to a save profile).
#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameSettings {
    pub version: u32,
    pub last_active_profile: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioSettingsData {
    pub music_enabled: bool,
    pub music_volume: f32,
    pub sfx_enabled: bool,
    pub sfx_volume: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfileSettings {
    pub audio: AudioSettingsData,
    pub skill_bindings: [Option<SkillKind>; SKILL_SLOT_COUNT],
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            last_active_profile: 0,
        }
    }
}

impl Default for AudioSettingsData {
    fn default() -> Self {
        let audio = AudioSettings::default();
        Self {
            music_enabled: audio.music_enabled,
            music_volume: audio.music_volume,
            sfx_enabled: audio.sfx_enabled,
            sfx_volume: audio.sfx_volume,
        }
    }
}

impl Default for ProfileSettings {
    fn default() -> Self {
        let defaults = SkillBindings::default();
        Self {
            audio: AudioSettingsData::default(),
            skill_bindings: defaults.slots,
        }
    }
}

impl GameSettings {
    pub fn migrate(mut self) -> Self {
        if self.version < SETTINGS_VERSION {
            self.version = SETTINGS_VERSION;
        }
        self.last_active_profile = self.last_active_profile.min(super::profile::PROFILE_COUNT - 1);
        self
    }
}

impl ProfileSettings {
    pub fn apply_audio(&self, audio: &mut AudioSettings) {
        audio.music_enabled = self.audio.music_enabled;
        audio.music_volume = self.audio.music_volume.clamp(0.0, 1.0);
        audio.sfx_enabled = self.audio.sfx_enabled;
        audio.sfx_volume = self.audio.sfx_volume.clamp(0.0, 1.0);
    }

    pub fn apply_skill_bindings(&self, bindings: &mut SkillBindings) {
        bindings.slots = self.skill_bindings;
    }

    pub fn capture_audio(&mut self, audio: &AudioSettings) {
        self.audio.music_enabled = audio.music_enabled;
        self.audio.music_volume = audio.music_volume;
        self.audio.sfx_enabled = audio.sfx_enabled;
        self.audio.sfx_volume = audio.sfx_volume;
    }

    pub fn capture_skill_bindings(&mut self, bindings: &SkillBindings) {
        self.skill_bindings = bindings.slots;
    }
}