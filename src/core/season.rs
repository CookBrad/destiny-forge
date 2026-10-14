use std::sync::OnceLock;

use bevy::prelude::*;
use serde::Deserialize;

use crate::core::data_load::load_ron_from_assets_or_embedded;

const CALENDAR_RULES_PATH: &str = "assets/data/calendar_rules.ron";
const EMBEDDED_CALENDAR_RULES: &str = include_str!("../../assets/data/calendar_rules.ron");

const SEASONS_IN_YEAR: [Season; 4] = [Season::Spring, Season::Summer, Season::Fall, Season::Winter];

const WEEKDAYS: [Weekday; 7] = [
    Weekday::Gearday,
    Weekday::Steamday,
    Weekday::Sparkday,
    Weekday::Forgeday,
    Weekday::Brassday,
    Weekday::Harvestday,
    Weekday::Restday,
];

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct CalendarRules {
    days_per_season: u32,
}

impl Default for CalendarRules {
    fn default() -> Self {
        Self::load()
    }
}

impl CalendarRules {
    pub fn load() -> Self {
        static LOADED: OnceLock<CalendarRules> = OnceLock::new();
        *LOADED.get_or_init(Self::read_from_data)
    }

    fn read_from_data() -> Self {
        load_ron_from_assets_or_embedded::<Self>(
            CALENDAR_RULES_PATH,
            EMBEDDED_CALENDAR_RULES,
            "calendar rules",
        )
        .map(Self::sanitized)
        .unwrap_or(Self { days_per_season: 1 })
    }

    pub fn with_season_length(days_per_season: u32) -> Self {
        Self { days_per_season }.sanitized()
    }

    pub fn season_length(self) -> u32 {
        self.days_per_season.max(1)
    }

    fn sanitized(self) -> Self {
        Self {
            days_per_season: self.days_per_season.max(1),
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Season {
    #[default]
    Spring,
    Summer,
    Fall,
    Winter,
}

impl Season {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Spring => "Spring",
            Self::Summer => "Summer",
            Self::Fall => "Fall",
            Self::Winter => "Winter",
        }
    }

    fn from_year_day(day_in_year: u32, season_length: u32) -> Self {
        let index = (day_in_year / season_length) as usize;
        SEASONS_IN_YEAR[index.min(SEASONS_IN_YEAR.len() - 1)]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Weekday {
    Gearday,
    Steamday,
    Sparkday,
    Forgeday,
    Brassday,
    Harvestday,
    Restday,
}

impl Weekday {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Gearday => "Gearday",
            Self::Steamday => "Steamday",
            Self::Sparkday => "Sparkday",
            Self::Forgeday => "Forgeday",
            Self::Brassday => "Brassday",
            Self::Harvestday => "Harvestday",
            Self::Restday => "Restday",
        }
    }

    fn from_day_index(day_index: u32) -> Self {
        WEEKDAYS[(day_index % WEEKDAYS.len() as u32) as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarDate {
    pub calendar_day: u32,
    pub year: u32,
    pub season: Season,
    pub day_of_season: u32,
    pub weekday: Weekday,
}

impl CalendarDate {
    pub fn from_calendar_day(calendar_day: u32, rules: &CalendarRules) -> Self {
        let calendar_day = calendar_day.max(1);
        let day_index = calendar_day - 1;
        let season_length = rules.season_length();
        let days_per_year = season_length.saturating_mul(SEASONS_IN_YEAR.len() as u32);
        let year = day_index / days_per_year + 1;
        let day_in_year = day_index % days_per_year;
        Self {
            calendar_day,
            year,
            season: Season::from_year_day(day_in_year, season_length),
            day_of_season: day_in_year % season_length + 1,
            weekday: Weekday::from_day_index(day_index),
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeasonRollover {
    pub ended: Season,
    pub started: Season,
    pub year: u32,
    pub day_of_season: u32,
    pub weekday: Weekday,
    pub calendar_day: u32,
}

pub fn rollover_between(before: CalendarDate, after: CalendarDate) -> Option<SeasonRollover> {
    if before.season == after.season {
        return None;
    }
    Some(SeasonRollover {
        ended: before.season,
        started: after.season,
        year: after.year,
        day_of_season: after.day_of_season,
        weekday: after.weekday,
        calendar_day: after.calendar_day,
    })
}

pub(crate) fn register_season(app: &mut App) {
    app.init_resource::<CalendarRules>()
        .init_resource::<Season>()
        .add_event::<SeasonRollover>();
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::day_cycle::{perform_sleep, DayClock, DayPhase, ToolEnergy};

    fn shipped_rules() -> CalendarRules {
        CalendarRules::with_season_length(28)
    }

    fn date_on(calendar_day: u32, rules: &CalendarRules) -> CalendarDate {
        CalendarDate::from_calendar_day(calendar_day, rules)
    }

    #[test]
    fn shipped_rules_keep_a_twenty_eight_day_season() {
        assert_eq!(CalendarRules::load().season_length(), 28);
    }

    #[test]
    fn opening_morning_is_year_one_spring_gearday() {
        let date = date_on(1, &shipped_rules());
        assert_eq!(date.year, 1);
        assert_eq!(date.season, Season::Spring);
        assert_eq!(date.day_of_season, 1);
        assert_eq!(date.weekday, Weekday::Gearday);
        assert_eq!(date.season.display_name(), "Spring");
        assert_eq!(date.weekday.display_name(), "Gearday");
    }

    #[test]
    fn zero_calendar_day_matches_the_first_morning() {
        let rules = shipped_rules();
        assert_eq!(date_on(0, &rules), date_on(1, &rules));
    }

    #[test]
    fn seasons_years_and_weekdays_follow_calendar_day() {
        let rules = shipped_rules();
        let samples = [
            (28, 1, Season::Spring, 28, Weekday::Restday),
            (29, 1, Season::Summer, 1, Weekday::Gearday),
            (56, 1, Season::Summer, 28, Weekday::Restday),
            (57, 1, Season::Fall, 1, Weekday::Gearday),
            (84, 1, Season::Fall, 28, Weekday::Restday),
            (85, 1, Season::Winter, 1, Weekday::Gearday),
            (112, 1, Season::Winter, 28, Weekday::Restday),
            (113, 2, Season::Spring, 1, Weekday::Gearday),
            (224, 2, Season::Winter, 28, Weekday::Restday),
            (225, 3, Season::Spring, 1, Weekday::Gearday),
        ];

        for (calendar_day, year, season, day_of_season, weekday) in samples {
            let date = date_on(calendar_day, &rules);
            assert_eq!(date.year, year, "year on day {calendar_day}");
            assert_eq!(date.season, season, "season on day {calendar_day}");
            assert_eq!(
                date.day_of_season, day_of_season,
                "day of season on day {calendar_day}"
            );
            assert_eq!(date.weekday, weekday, "weekday on day {calendar_day}");
        }
    }

    #[test]
    fn weekday_names_run_gearday_through_restday() {
        let rules = shipped_rules();
        let names = [
            Weekday::Gearday,
            Weekday::Steamday,
            Weekday::Sparkday,
            Weekday::Forgeday,
            Weekday::Brassday,
            Weekday::Harvestday,
            Weekday::Restday,
        ];
        for day in 1..=21 {
            assert_eq!(
                date_on(day, &rules).weekday,
                names[(day as usize - 1) % names.len()]
            );
        }
    }

    #[test]
    fn a_shorter_season_rolls_over_on_the_configured_day() {
        let rules = CalendarRules::with_season_length(10);
        let last = date_on(10, &rules);
        assert_eq!(last.season, Season::Spring);
        assert_eq!(last.day_of_season, 10);
        assert_eq!(last.weekday, Weekday::Sparkday);

        let next = date_on(11, &rules);
        assert_eq!(next.season, Season::Summer);
        assert_eq!(next.day_of_season, 1);
        assert_eq!(next.year, 1);
        assert_eq!(next.weekday, Weekday::Forgeday);

        let year_two = date_on(41, &rules);
        assert_eq!(year_two.year, 2);
        assert_eq!(year_two.season, Season::Spring);
        assert_eq!(year_two.day_of_season, 1);
    }

    #[test]
    fn a_zero_season_length_counts_as_one_day() {
        let rules = CalendarRules::with_season_length(0);
        assert_eq!(rules.season_length(), 1);
        let date = date_on(5, &rules);
        assert_eq!(date.year, 2);
        assert_eq!(date.season, Season::Spring);
        assert_eq!(date.day_of_season, 1);
    }

    #[test]
    fn sleep_inside_a_season_keeps_spring() {
        let rules = shipped_rules();
        let mut clock = DayClock::default();
        let mut energy = ToolEnergy::default();
        let mut season = Season::Spring;

        let outcome = perform_sleep(&mut clock, &mut energy, &mut season, &rules);

        assert_eq!(outcome.calendar_day, 2);
        assert_eq!(season, Season::Spring);
        assert_eq!(outcome.season_rollover, None);
        assert_eq!(date_on(2, &rules).weekday, Weekday::Steamday);
    }

    #[test]
    fn sleep_on_each_season_boundary_reports_the_next_season() {
        let rules = shipped_rules();
        let boundaries = [
            (28, Season::Spring, Season::Summer, 1),
            (56, Season::Summer, Season::Fall, 1),
            (84, Season::Fall, Season::Winter, 1),
            (112, Season::Winter, Season::Spring, 2),
        ];

        for (last_day, ended, started, year) in boundaries {
            let mut clock = DayClock::from_saved(last_day, DayPhase::Evening);
            let mut energy = ToolEnergy::default();
            let mut season = date_on(last_day, &rules).season;

            let outcome = perform_sleep(&mut clock, &mut energy, &mut season, &rules);

            assert_eq!(season, started, "season after day {last_day}");
            assert_eq!(
                outcome.season_rollover,
                Some(SeasonRollover {
                    ended,
                    started,
                    year,
                    day_of_season: 1,
                    weekday: Weekday::Gearday,
                    calendar_day: last_day + 1,
                }),
                "rollover leaving day {last_day}"
            );
        }
    }

    #[test]
    fn sleep_uses_the_configured_season_length() {
        let rules = CalendarRules::with_season_length(10);
        let mut clock = DayClock::from_saved(10, DayPhase::Evening);
        let mut energy = ToolEnergy::default();
        let mut season = Season::Spring;

        let outcome = perform_sleep(&mut clock, &mut energy, &mut season, &rules);

        assert_eq!(outcome.calendar_day, 11);
        assert_eq!(season, Season::Summer);
        assert_eq!(
            outcome.season_rollover,
            Some(SeasonRollover {
                ended: Season::Spring,
                started: Season::Summer,
                year: 1,
                day_of_season: 1,
                weekday: Weekday::Forgeday,
                calendar_day: 11,
            })
        );
    }

    #[test]
    fn a_distant_calendar_day_is_year_two_winter_forgeday() {
        let date = date_on(200, &shipped_rules());
        assert_eq!(date.season, Season::Winter);
        assert_eq!(date.year, 2);
        assert_eq!(date.day_of_season, 4);
        assert_eq!(date.weekday, Weekday::Forgeday);
    }
}
