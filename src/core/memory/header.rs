use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::super::day_cycle::DayPhase;
use super::super::state::GameState;
use super::autosave::{debounced_autosave, queue_autosave, ProfileDirty};
use super::profile::{bank_playtime, PlayerProfile};
use super::sync::capture_profile_from_runtime;

#[derive(Resource, Default)]
pub(super) struct BankedPlaytimeSeconds(pub u64);

pub(super) fn add_save_header_systems(app: &mut App) {
    app.init_resource::<BankedPlaytimeSeconds>()
        .add_systems(
            Update,
            (
                accrue_playtime,
                mark_profile_dirty_when_a_second_is_banked,
                remember_current_location,
                capture_profile_from_runtime,
                queue_autosave,
                debounced_autosave,
            )
                .chain(),
        );
}

pub(super) fn accrue_playtime(
    time: Res<Time>,
    state: Res<State<GameState>>,
    mut profile: ResMut<PlayerProfile>,
    mut banked: ResMut<BankedPlaytimeSeconds>,
) {
    if state.get() == &GameState::Title {
        banked.0 = 0;
        return;
    }
    banked.0 = bank_playtime(&mut profile, time.delta_secs());
}

pub(super) fn mark_profile_dirty_when_a_second_is_banked(
    banked: Res<BankedPlaytimeSeconds>,
    mut dirty: ResMut<ProfileDirty>,
) {
    if banked.0 > 0 {
        dirty.mark();
    }
}

pub(super) fn remember_current_location(
    state: Res<State<GameState>>,
    mut profile: ResMut<PlayerProfile>,
    mut dirty: ResMut<ProfileDirty>,
) {
    if location_changed_after_storing(&mut profile.header, state.get()) {
        dirty.mark();
    }
}

fn location_changed_after_storing(header: &mut SaveHeader, state: &GameState) -> bool {
    let Some(location) = SaveLocation::from_game_state(state) else {
        return false;
    };
    if header.location == location {
        return false;
    }
    header.location = location;
    true
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_location_records_the_forest() {
        let (changed, location) =
            store_location_in_copy(SaveHeader::default(), GameState::Forest);
        assert!(changed);
        assert_eq!(location, SaveLocation::Forest);
    }

    fn store_location_in_copy(header: SaveHeader, state: GameState) -> (bool, SaveLocation) {
        let mut header = header;
        let changed = location_changed_after_storing(&mut header, &state);
        (changed, header.location)
    }

    #[test]
    fn store_location_leaves_an_unchanged_forest_alone() {
        let header = SaveHeader {
            location: SaveLocation::Forest,
            ..SaveHeader::default()
        };
        assert_eq!(
            store_location_in_copy(header, GameState::Forest),
            (false, SaveLocation::Forest)
        );
    }

    #[test]
    fn store_location_leaves_the_mine_alone_on_the_title_screen() {
        let header = SaveHeader {
            location: SaveLocation::Mine,
            ..SaveHeader::default()
        };
        assert_eq!(
            store_location_in_copy(header, GameState::Title),
            (false, SaveLocation::Mine)
        );
    }

    #[test]
    fn each_zone_maps_to_its_save_location() {
        assert_eq!(
            SaveLocation::from_game_state(&GameState::Overworld),
            Some(SaveLocation::Homestead)
        );
        assert_eq!(
            SaveLocation::from_game_state(&GameState::Forest),
            Some(SaveLocation::Forest)
        );
        assert_eq!(
            SaveLocation::from_game_state(&GameState::Mine),
            Some(SaveLocation::Mine)
        );
        assert_eq!(
            SaveLocation::from_game_state(&GameState::Dungeon),
            Some(SaveLocation::Dungeon)
        );
    }

    #[test]
    fn the_title_screen_has_no_save_location() {
        assert_eq!(SaveLocation::from_game_state(&GameState::Title), None);
    }

    #[test]
    fn entering_the_forest_records_that_location_and_marks_the_profile_dirty() {
        let mut app = app_at_location(GameState::Forest);
        remember_location_once(&mut app);
        assert_eq!(recorded_location(&app), (SaveLocation::Forest, true));
    }

    fn app_at_location(state: GameState) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(State::new(state))
            .insert_resource(PlayerProfile::default())
            .insert_resource(ProfileDirty::default());
        app
    }

    fn remember_location_once(app: &mut App) {
        app.add_systems(Update, remember_current_location);
        app.update();
    }

    fn recorded_location(app: &App) -> (SaveLocation, bool) {
        (
            app.world().resource::<PlayerProfile>().header.location,
            app.world().resource::<ProfileDirty>().0,
        )
    }

    #[test]
    fn staying_in_the_forest_does_not_mark_the_profile_dirty() {
        let mut app = app_at_location(GameState::Forest);
        app.world_mut()
            .resource_mut::<PlayerProfile>()
            .header
            .location = SaveLocation::Forest;
        remember_location_once(&mut app);
        assert_eq!(recorded_location(&app), (SaveLocation::Forest, false));
    }

    #[test]
    fn the_title_screen_does_not_replace_a_saved_mine() {
        let mut app = app_at_location(GameState::Title);
        app.world_mut()
            .resource_mut::<PlayerProfile>()
            .header
            .location = SaveLocation::Mine;
        remember_location_once(&mut app);
        assert_eq!(recorded_location(&app), (SaveLocation::Mine, false));
    }

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
            .insert_resource(BankedPlaytimeSeconds::default())
            .add_systems(
                Update,
                (accrue_playtime, mark_profile_dirty_when_a_second_is_banked).chain(),
            );
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
    fn accrue_playtime_banks_one_second_and_leaves_the_profile_clean() {
        let mut app = app_accruing_playtime(GameState::Overworld);
        set_frame_milliseconds(&mut app, 1500);
        app.update();
        assert_eq!(
            stored_playtime(&app),
            StoredPlaytime {
                seconds: 1,
                fraction: 0.5,
                dirty: false,
            }
        );
    }

    fn app_accruing_playtime(state: GameState) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(State::new(state))
            .insert_resource(PlayerProfile::default())
            .insert_resource(ProfileDirty::default())
            .insert_resource(BankedPlaytimeSeconds::default())
            .add_systems(Update, accrue_playtime);
        app.world_mut()
            .resource_mut::<Time<bevy::time::Virtual>>()
            .set_max_delta(std::time::Duration::from_secs(2));
        set_frame_milliseconds(&mut app, 0);
        app.update();
        app
    }

    #[test]
    fn a_banked_second_marks_the_profile_dirty() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ProfileDirty::default())
            .insert_resource(BankedPlaytimeSeconds(1))
            .add_systems(Update, mark_profile_dirty_when_a_second_is_banked);
        app.update();
        assert!(app.world().resource::<ProfileDirty>().0);
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
