//! World-wide pause: overworld, forest, and dungeon share one flag.
//! Dungeon still uses `DungeonPlayState::Paused` for hunt-specific UI.

use bevy::prelude::*;

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldPause {
    pub paused: bool,
}

impl WorldPause {
    pub fn is_paused(self) -> bool {
        self.paused
    }
}

pub fn world_unpaused(pause: Res<WorldPause>) -> bool {
    !pause.paused
}

pub fn pause_virtual_time(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

pub fn resume_virtual_time(mut time: ResMut<Time<Virtual>>) {
    if time.is_paused() {
        time.unpause();
    }
}
