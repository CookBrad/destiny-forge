//! Apply saved zone/position when starting a profile run.

use bevy::prelude::*;

use crate::core::GameState;
use crate::player::{SavedLocation, SavedZone};

use super::setup::OverworldEntry;

/// Spawn override for lake / forest when resuming mid-zone.
#[derive(Resource, Clone, Copy, Debug)]
pub struct ZoneResumeSpawn(pub Vec2);

/// Queued by title UI (avoids packing Commands into already-full system signatures).
#[derive(Resource, Default, Debug)]
pub struct PendingResume {
    pub ready: bool,
    pub state: GameState,
    pub overworld: Option<OverworldEntry>,
    pub zone: Option<ZoneResumeSpawn>,
}

/// Configure entry resources and return the GameState to enter.
pub fn resume_destination(location: SavedLocation) -> (GameState, Option<OverworldEntry>, Option<ZoneResumeSpawn>) {
    match location.zone {
        SavedZone::Overworld => {
            let entry = if location.has_position() {
                Some(OverworldEntry::At(location.pos()))
            } else {
                Some(OverworldEntry::Yard)
            };
            (GameState::Overworld, entry, None)
        }
        SavedZone::Lake => {
            let spawn = if location.has_position() {
                ZoneResumeSpawn(location.pos())
            } else {
                // Default west pier approach
                ZoneResumeSpawn(Vec2::new(
                    crate::graphics::TILE * 4.5,
                    crate::graphics::TILE * 12.5,
                ))
            };
            (GameState::Lake, None, Some(spawn))
        }
        SavedZone::Forest => {
            let spawn = if location.has_position() {
                ZoneResumeSpawn(location.pos())
            } else {
                ZoneResumeSpawn(Vec2::new(
                    crate::graphics::TILE * 3.5,
                    crate::graphics::TILE * 4.5,
                ))
            };
            (GameState::Forest, None, Some(spawn))
        }
    }
}

pub fn queue_resume(pending: &mut PendingResume, location: SavedLocation) {
    let (state, overworld, zone) = resume_destination(location);
    pending.ready = true;
    pending.state = state;
    pending.overworld = overworld;
    pending.zone = zone;
}

/// Apply queued resume on the title screen after a profile is chosen.
pub fn apply_pending_resume(
    mut commands: Commands,
    mut pending: ResMut<PendingResume>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !pending.ready {
        return;
    }
    pending.ready = false;
    if let Some(entry) = pending.overworld.take() {
        commands.insert_resource(entry);
    } else {
        commands.remove_resource::<OverworldEntry>();
    }
    if let Some(spawn) = pending.zone.take() {
        commands.insert_resource(spawn);
    } else {
        commands.remove_resource::<ZoneResumeSpawn>();
    }
    next_state.set(pending.state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lake_resume_targets_lake_state() {
        let loc = SavedLocation::new(SavedZone::Lake, Vec2::new(220.0, 180.0));
        let (state, overworld, zone) = resume_destination(loc);
        assert_eq!(state, GameState::Lake);
        assert!(overworld.is_none());
        assert!(zone.is_some_and(|z| (z.0.x - 220.0).abs() < 0.01));
    }

    #[test]
    fn overworld_resume_with_position() {
        let loc = SavedLocation::new(SavedZone::Overworld, Vec2::new(50.0, 60.0));
        let (state, overworld, zone) = resume_destination(loc);
        assert_eq!(state, GameState::Overworld);
        assert!(matches!(overworld, Some(OverworldEntry::At(p)) if (p.x - 50.0).abs() < 0.01));
        assert!(zone.is_none());
    }
}
