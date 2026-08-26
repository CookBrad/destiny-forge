//! Persisted exploration location (zone + position) for resume after title.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::GameState;

/// Which exploration map the player last occupied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SavedZone {
    #[default]
    Overworld,
    Forest,
    Lake,
}

/// Last known world position for resume-from-title.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedLocation {
    pub zone: SavedZone,
    pub x: f32,
    pub y: f32,
}

impl Default for SavedLocation {
    fn default() -> Self {
        // (0,0) means “use default yard spawn” for overworld.
        Self {
            zone: SavedZone::Overworld,
            x: 0.0,
            y: 0.0,
        }
    }
}

impl SavedLocation {
    pub fn new(zone: SavedZone, pos: Vec2) -> Self {
        Self {
            zone,
            x: pos.x,
            y: pos.y,
        }
    }

    pub fn pos(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn has_position(self) -> bool {
        self.x.abs() > 0.5 || self.y.abs() > 0.5
    }

    pub fn from_game_state(state: GameState, pos: Vec2) -> Option<Self> {
        let zone = match state {
            GameState::Overworld => SavedZone::Overworld,
            GameState::Forest => SavedZone::Forest,
            GameState::Lake => SavedZone::Lake,
            _ => return None,
        };
        Some(Self::new(zone, pos))
    }

    pub fn to_game_state(self) -> GameState {
        match self.zone {
            SavedZone::Overworld => GameState::Overworld,
            SavedZone::Forest => GameState::Forest,
            SavedZone::Lake => GameState::Lake,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lake_location_round_trips_zone() {
        let loc = SavedLocation::new(SavedZone::Lake, Vec2::new(100.0, 200.0));
        assert_eq!(loc.to_game_state(), GameState::Lake);
        assert!(loc.has_position());
        assert!((loc.pos().x - 100.0).abs() < 0.01);
    }

    #[test]
    fn default_is_overworld_without_forced_position() {
        let loc = SavedLocation::default();
        assert_eq!(loc.zone, SavedZone::Overworld);
        assert!(!loc.has_position());
    }
}
