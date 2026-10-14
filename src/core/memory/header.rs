use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::super::day_cycle::DayPhase;
use super::super::state::GameState;
use super::profile::PlayerProfile;
use super::ProfileDirty;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaveLocation {
    #[default]
    Homestead,
    Forest,
    Mine,
    Dungeon,
}

impl SaveLocation {
    fn from_game_state(state: &GameState) -> Option<Self> {
        match state {
            GameState::Title => None,
            GameState::Overworld => Some(Self::Homestead),
            GameState::Forest => Some(Self::Forest),
            GameState::Mine => Some(Self::Mine),
            GameState::Dungeon => Some(Self::Dungeon),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveCalendar {
    pub calendar_day: u32,
    pub day_phase: DayPhase,
}

impl Default for SaveCalendar {
    fn default() -> Self {
        Self {
            calendar_day: 1,
            day_phase: DayPhase::Morning,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveHeader {
    pub name: String,
    pub playtime_seconds: u64,
    pub calendar: SaveCalendar,
    pub location: SaveLocation,
    pub gold: u32,
}

impl Default for SaveHeader {
    fn default() -> Self {
        Self {
            name: String::new(),
            playtime_seconds: 0,
            calendar: SaveCalendar::default(),
            location: SaveLocation::Homestead,
            gold: 0,
        }
    }
}

pub(super) fn accrue_playtime(
    time: Res<Time>,
    state: Res<State<GameState>>,
    mut profile: ResMut<PlayerProfile>,
) {
    if state.get() == &GameState::Title {
        return;
    }
    let delta_seconds = time.delta_secs();
    let PlayerProfile {
        header,
        playtime_fraction,
        ..
    } = &mut *profile;
    add_playtime(header, playtime_fraction, delta_seconds);
}

pub(super) fn remember_current_location(
    state: Res<State<GameState>>,
    mut profile: ResMut<PlayerProfile>,
    mut dirty: ResMut<ProfileDirty>,
) {
    if store_location(&mut profile.header, state.get()) {
        dirty.mark();
    }
}

fn add_playtime(header: &mut SaveHeader, fraction: &mut f64, delta_seconds: f32) {
    if delta_seconds <= 0.0 {
        return;
    }
    *fraction += f64::from(delta_seconds);
    let whole_seconds = fraction.floor();
    if whole_seconds < 1.0 {
        return;
    }
    *fraction -= whole_seconds;
    header.playtime_seconds = header.playtime_seconds.saturating_add(whole_seconds as u64);
}

fn store_location(header: &mut SaveHeader, state: &GameState) -> bool {
    let Some(location) = SaveLocation::from_game_state(state) else {
        return false;
    };
    if header.location == location {
        return false;
    }
    header.location = location;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playtime_banks_whole_seconds_and_keeps_the_fraction() {
        let mut header = SaveHeader::default();
        let mut fraction = 0.0;
        add_playtime(&mut header, &mut fraction, 0.4);
        assert_eq!(header.playtime_seconds, 0);
        add_playtime(&mut header, &mut fraction, 0.7);
        assert_eq!(header.playtime_seconds, 1);
        add_playtime(&mut header, &mut fraction, 2.2);
        assert_eq!(header.playtime_seconds, 3);
        assert!(fraction > 0.0 && fraction < 1.0);
    }

    #[test]
    fn title_does_not_replace_a_saved_location() {
        let mut header = SaveHeader {
            location: SaveLocation::Mine,
            ..SaveHeader::default()
        };
        assert!(!store_location(&mut header, &GameState::Title));
        assert_eq!(header.location, SaveLocation::Mine);
    }

    #[test]
    fn entering_a_zone_records_that_location_once() {
        let mut header = SaveHeader::default();
        assert!(store_location(&mut header, &GameState::Forest));
        assert_eq!(header.location, SaveLocation::Forest);
        assert!(!store_location(&mut header, &GameState::Forest));
        assert!(store_location(&mut header, &GameState::Dungeon));
        assert_eq!(header.location, SaveLocation::Dungeon);
    }
}
