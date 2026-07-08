//! Apply soft-day cost when entering a dungeon hunt.

use bevy::prelude::*;

use super::day_cycle::DayClock;
use super::memory::{PlayerProfile, ProfileDirty};

/// On dungeon enter: spend a large share of the day (tunable via `HUNT_DAY_COST_STEPS`).
pub fn apply_hunt_day_cost_on_dungeon_enter(
    mut clock: ResMut<DayClock>,
    mut profile: ResMut<PlayerProfile>,
    mut dirty: ResMut<ProfileDirty>,
) {
    let before = clock.phase;
    let changed = clock.apply_hunt_day_cost();
    profile.calendar_day = clock.calendar_day;
    profile.day_phase = clock.phase;
    dirty.mark();

    if changed {
        info!(
            "Hunt day cost: {} → {} (day {})",
            before.display_name(),
            clock.phase.display_name(),
            clock.calendar_day
        );
    } else {
        info!(
            "Hunt day cost: already {}, day stays {}",
            clock.phase.display_name(),
            clock.calendar_day
        );
    }
}
