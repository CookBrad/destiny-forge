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

pub fn world_paused(pause: Res<WorldPause>) -> bool {
    pause.paused
}

pub fn set_world_paused(pause: &mut WorldPause) {
    pause.paused = true;
}

pub fn clear_world_pause(pause: &mut WorldPause) {
    pause.paused = false;
}

pub fn pause_virtual_time(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

pub fn resume_virtual_time(mut time: ResMut<Time<Virtual>>) {
    if time.is_paused() {
        time.unpause();
    }
}

#[cfg(test)]
mod tests {
    use super::WorldPause;

    #[test]
    fn world_pause_defaults_unpaused() {
        let pause = WorldPause::default();
        assert!(!pause.is_paused());
        assert!(!pause.paused);
    }

    #[test]
    fn world_pause_set_and_clear() {
        let mut pause = WorldPause::default();
        super::set_world_paused(&mut pause);
        assert!(pause.is_paused());
        super::clear_world_pause(&mut pause);
        assert!(!pause.is_paused());
    }
}
