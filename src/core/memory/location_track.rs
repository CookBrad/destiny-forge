//! Periodically store the player's exploration zone + position on the profile.

use bevy::prelude::*;

use crate::core::{GameState, PlayerProfile, ProfileDirty};
use crate::overworld::movement::OverworldPlayer;
use crate::player::SavedLocation;

/// Throttle location writes so we don't dirty the profile every frame.
#[derive(Resource)]
pub struct LocationTrackTimer(Timer);

impl Default for LocationTrackTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

pub fn track_exploration_location(
    time: Res<Time>,
    mut timer: ResMut<LocationTrackTimer>,
    state: Res<State<GameState>>,
    player: Query<&Transform, With<OverworldPlayer>>,
    mut profile: ResMut<PlayerProfile>,
    mut dirty: ResMut<ProfileDirty>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    let Ok(tf) = player.get_single() else {
        return;
    };
    let Some(loc) = SavedLocation::from_game_state(*state.get(), tf.translation.truncate()) else {
        return;
    };
    // Skip no-op writes
    if profile.location == loc {
        return;
    }
    profile.location = loc;
    dirty.mark();
}
