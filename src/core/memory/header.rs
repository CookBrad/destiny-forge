use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::super::day_cycle::DayPhase;
use super::super::state::GameState;
use super::autosave::{debounced_autosave, queue_autosave, ProfileDirty};
use super::profile::{bank_playtime, PlayerProfile};
use super::sync::capture_profile_from_runtime;

pub(super) fn add_save_header_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            accrue_playtime_and_mark_dirty,
            capture_profile_from_runtime,
            queue_autosave,
            debounced_autosave,
        )
            .chain(),
    );
}

pub(super) fn accrue_playtime_and_mark_dirty(
    time: Res<Time>,
    state: Res<State<GameState>>,
    mut profile: ResMut<PlayerProfile>,
    mut dirty: ResMut<ProfileDirty>,
) {
    if state.get() == &GameState::Title {
        return;
    }
    let banked_seconds = bank_playtime(&mut profile, time.delta_secs());
    mark_dirty_when_seconds_were_banked(&mut dirty, banked_seconds);
}

fn mark_dirty_when_seconds_were_banked(dirty: &mut ProfileDirty, banked_seconds: u64) {
    if banked_seconds > 0 {
        dirty.mark();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaveLocation {
    #[default]
    Homestead,
    Forest,
    Mine,
    Dungeon,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_save_header_starts_blank() {
        let header = SaveHeader::default();
        assert_eq!(header.name, "");
        assert_eq!(header.playtime_seconds, 0);
        assert_eq!(header.calendar, SaveCalendar::default());
        assert_eq!(header.location, SaveLocation::Homestead);
        assert_eq!(header.gold, 0);
    }

    #[test]
    fn a_new_save_calendar_starts_on_the_first_morning() {
        let calendar = SaveCalendar::default();
        assert_eq!(calendar.calendar_day, 1);
        assert_eq!(calendar.day_phase, DayPhase::Morning);
    }

    #[test]
    fn a_new_save_location_starts_at_the_homestead() {
        assert_eq!(SaveLocation::default(), SaveLocation::Homestead);
    }

    #[test]
    fn playing_three_quarters_of_a_second_then_half_a_second_banks_one_and_marks_dirty() {
        let mut app = app_recording_playtime(GameState::Overworld);
        set_frame_milliseconds(&mut app, 750);
        app.update();
        assert_eq!(
            stored_playtime(&app),
            StoredPlaytime {
                seconds: 0,
                fraction: 0.75,
                dirty: false,
            }
        );
        set_frame_milliseconds(&mut app, 500);
        app.update();
        assert_eq!(
            stored_playtime(&app),
            StoredPlaytime {
                seconds: 1,
                fraction: 0.25,
                dirty: true,
            }
        );
    }

    fn app_recording_playtime(state: GameState) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(State::new(state))
            .insert_resource(PlayerProfile::default())
            .insert_resource(ProfileDirty::default())
            .add_systems(Update, accrue_playtime_and_mark_dirty);
        app.world_mut()
            .resource_mut::<Time<bevy::time::Virtual>>()
            .set_max_delta(std::time::Duration::from_secs(2));
        set_frame_milliseconds(&mut app, 0);
        app.update();
        app
    }

    fn set_frame_milliseconds(app: &mut App, milliseconds: u64) {
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(milliseconds),
        ));
    }

    fn stored_playtime(app: &App) -> StoredPlaytime {
        let profile = app.world().resource::<PlayerProfile>();
        StoredPlaytime {
            seconds: profile.header.playtime_seconds,
            fraction: profile.playtime_fraction,
            dirty: app.world().resource::<ProfileDirty>().0,
        }
    }

    #[derive(Debug, PartialEq)]
    struct StoredPlaytime {
        seconds: u64,
        fraction: f64,
        dirty: bool,
    }

    #[test]
    fn playing_banks_one_second_and_marks_the_profile_dirty() {
        let mut app = app_recording_playtime(GameState::Overworld);
        set_frame_milliseconds(&mut app, 1500);
        app.update();
        assert_eq!(
            stored_playtime(&app),
            StoredPlaytime {
                seconds: 1,
                fraction: 0.5,
                dirty: true,
            }
        );
    }

    #[test]
    fn half_a_second_of_play_does_not_mark_the_profile_dirty() {
        let mut app = app_recording_playtime(GameState::Overworld);
        set_frame_milliseconds(&mut app, 500);
        app.update();
        assert_eq!(
            stored_playtime(&app),
            StoredPlaytime {
                seconds: 0,
                fraction: 0.5,
                dirty: false,
            }
        );
    }

    #[test]
    fn the_title_screen_does_not_accrue_playtime() {
        let mut app = app_recording_playtime(GameState::Title);
        set_frame_milliseconds(&mut app, 1500);
        app.update();
        assert_eq!(
            stored_playtime(&app),
            StoredPlaytime {
                seconds: 0,
                fraction: 0.0,
                dirty: false,
            }
        );
    }
}
