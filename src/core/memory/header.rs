use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::super::day_cycle::DayPhase;
use super::super::state::GameState;
use super::profile::PlayerProfile;
use super::ProfileDirty;

pub(super) fn accrue_playtime(
    time: Res<Time>,
    state: Res<State<GameState>>,
    mut profile: ResMut<PlayerProfile>,
    mut dirty: ResMut<ProfileDirty>,
) {
    if state.get() == &GameState::Title {
        return;
    }
    let seconds_before = profile.header.playtime_seconds;
    let delta_seconds = time.delta_secs();
    let PlayerProfile {
        header,
        playtime_fraction,
        ..
    } = &mut *profile;
    add_playtime(header, playtime_fraction, delta_seconds);
    if header.playtime_seconds != seconds_before {
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

pub(super) fn remember_current_location(
    state: Res<State<GameState>>,
    mut profile: ResMut<PlayerProfile>,
    mut dirty: ResMut<ProfileDirty>,
) {
    if store_location(&mut profile.header, state.get()) {
        dirty.mark();
    }
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
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    fn app_in(state: GameState) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(State::new(state))
            .insert_resource(PlayerProfile::default())
            .insert_resource(ProfileDirty::default());
        app
    }

    fn elapse(app: &mut App, millis: u64) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(millis));
    }

    fn seconds_and_dirty(app: &App) -> (u64, f64, bool) {
        let profile = app.world().resource::<PlayerProfile>();
        (
            profile.header.playtime_seconds,
            profile.playtime_fraction,
            app.world().resource::<ProfileDirty>().0,
        )
    }

    #[test]
    fn add_playtime_banks_one_second_and_keeps_half_a_second() {
        let mut header = SaveHeader::default();
        let mut fraction = 0.0;
        add_playtime(&mut header, &mut fraction, 1.5);
        assert_eq!(header.playtime_seconds, 1);
        assert_eq!(fraction, 0.5);
    }

    #[test]
    fn add_playtime_keeps_half_a_second_when_the_total_stays_below_one() {
        let mut header = SaveHeader::default();
        let mut fraction = 0.0;
        add_playtime(&mut header, &mut fraction, 0.5);
        assert_eq!(header.playtime_seconds, 0);
        assert_eq!(fraction, 0.5);
    }

    #[test]
    fn add_playtime_ignores_a_zero_or_negative_delta() {
        let mut header = SaveHeader::default();
        let mut fraction = 0.25;
        add_playtime(&mut header, &mut fraction, 0.0);
        add_playtime(&mut header, &mut fraction, -1.0);
        assert_eq!(header.playtime_seconds, 0);
        assert_eq!(fraction, 0.25);
    }

    #[test]
    fn add_playtime_saturates_at_the_largest_second_count() {
        let mut header = SaveHeader {
            playtime_seconds: u64::MAX,
            ..SaveHeader::default()
        };
        let mut fraction = 0.0;
        add_playtime(&mut header, &mut fraction, 1.0);
        assert_eq!(header.playtime_seconds, u64::MAX);
        assert_eq!(fraction, 0.0);
    }

    fn run_playtime(app: &mut App) {
        app.world_mut().run_system_once(accrue_playtime).unwrap();
    }

    #[test]
    fn playing_banks_one_second_and_marks_the_profile_dirty() {
        let mut app = app_in(GameState::Overworld);
        elapse(&mut app, 1500);
        run_playtime(&mut app);
        assert_eq!(seconds_and_dirty(&app), (1, 0.5, true));
    }

    #[test]
    fn half_a_second_of_play_does_not_mark_the_profile_dirty() {
        let mut app = app_in(GameState::Overworld);
        elapse(&mut app, 500);
        run_playtime(&mut app);
        assert_eq!(seconds_and_dirty(&app), (0, 0.5, false));
    }

    #[test]
    fn the_title_screen_does_not_accrue_playtime() {
        let mut app = app_in(GameState::Title);
        elapse(&mut app, 1500);
        run_playtime(&mut app);
        assert_eq!(seconds_and_dirty(&app), (0, 0.0, false));
    }

    fn place(header: SaveHeader, state: GameState) -> (bool, SaveLocation) {
        let mut header = header;
        let changed = store_location(&mut header, &state);
        (changed, header.location)
    }

    #[test]
    fn store_location_records_the_forest() {
        let (changed, location) = place(SaveHeader::default(), GameState::Forest);
        assert!(changed);
        assert_eq!(location, SaveLocation::Forest);
    }

    #[test]
    fn store_location_leaves_an_unchanged_forest_alone() {
        let header = SaveHeader {
            location: SaveLocation::Forest,
            ..SaveHeader::default()
        };
        assert_eq!(
            place(header, GameState::Forest),
            (false, SaveLocation::Forest)
        );
    }

    #[test]
    fn store_location_leaves_the_mine_alone_on_the_title_screen() {
        let header = SaveHeader {
            location: SaveLocation::Mine,
            ..SaveHeader::default()
        };
        assert_eq!(place(header, GameState::Title), (false, SaveLocation::Mine));
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

    fn run_location(app: &mut App) {
        app.world_mut()
            .run_system_once(remember_current_location)
            .unwrap();
    }

    fn saved_location(app: &App) -> (SaveLocation, bool) {
        (
            app.world().resource::<PlayerProfile>().header.location,
            app.world().resource::<ProfileDirty>().0,
        )
    }

    #[test]
    fn entering_the_forest_records_that_location_and_marks_the_profile_dirty() {
        let mut app = app_in(GameState::Forest);
        run_location(&mut app);
        assert_eq!(saved_location(&app), (SaveLocation::Forest, true));
    }

    #[test]
    fn staying_in_the_forest_does_not_mark_the_profile_dirty() {
        let mut app = app_in(GameState::Forest);
        app.world_mut()
            .resource_mut::<PlayerProfile>()
            .header
            .location = SaveLocation::Forest;
        run_location(&mut app);
        assert_eq!(saved_location(&app), (SaveLocation::Forest, false));
    }

    #[test]
    fn the_title_screen_does_not_replace_a_saved_mine() {
        let mut app = app_in(GameState::Title);
        app.world_mut()
            .resource_mut::<PlayerProfile>()
            .header
            .location = SaveLocation::Mine;
        run_location(&mut app);
        assert_eq!(saved_location(&app), (SaveLocation::Mine, false));
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
}
