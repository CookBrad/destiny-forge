use bevy::prelude::*;

pub const DAYS_PER_WEEK: u32 = 7;
pub const DAYS_PER_SEASON: u32 = 28;
pub const SEASONS_PER_YEAR: u32 = 4;
pub const DAYS_PER_YEAR: u32 = DAYS_PER_SEASON * SEASONS_PER_YEAR;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Season {
    #[default]
    Spring,
    Summer,
    Fall,
    Winter,
}

const SEASONS_IN_YEAR: [Season; SEASONS_PER_YEAR as usize] =
    [Season::Spring, Season::Summer, Season::Fall, Season::Winter];

impl Season {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Spring => "Spring",
            Self::Summer => "Summer",
            Self::Fall => "Fall",
            Self::Winter => "Winter",
        }
    }

    fn from_year_day(day_in_year: u32) -> Self {
        SEASONS_IN_YEAR[(day_in_year / DAYS_PER_SEASON) as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

const WEEKDAYS: [Weekday; DAYS_PER_WEEK as usize] = [
    Weekday::Monday,
    Weekday::Tuesday,
    Weekday::Wednesday,
    Weekday::Thursday,
    Weekday::Friday,
    Weekday::Saturday,
    Weekday::Sunday,
];

impl Weekday {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
            Self::Sunday => "Sunday",
        }
    }

    fn from_day_index(day_index: u32) -> Self {
        WEEKDAYS[(day_index % DAYS_PER_WEEK) as usize]
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
    pub fn from_calendar_day(calendar_day: u32) -> Self {
        let calendar_day = calendar_day.max(1);
        let day_index = calendar_day - 1;
        let year = day_index / DAYS_PER_YEAR + 1;
        let day_in_year = day_index % DAYS_PER_YEAR;
        Self {
            calendar_day,
            year,
            season: Season::from_year_day(day_in_year),
            day_of_season: day_in_year % DAYS_PER_SEASON + 1,
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
    app.init_resource::<Season>().add_event::<SeasonRollover>();
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::day_cycle::{perform_sleep, DayClock, ToolEnergy};

    #[test]
    fn opening_morning_is_year_one_spring_monday() {
        let date = CalendarDate::from_calendar_day(1);
        assert_eq!(date.year, 1);
        assert_eq!(date.season, Season::Spring);
        assert_eq!(date.day_of_season, 1);
        assert_eq!(date.weekday, Weekday::Monday);
        assert_eq!(date.season.display_name(), "Spring");
        assert_eq!(date.weekday.display_name(), "Monday");
    }

    #[test]
    fn zero_calendar_day_matches_the_first_morning() {
        assert_eq!(
            CalendarDate::from_calendar_day(0),
            CalendarDate::from_calendar_day(1)
        );
    }

    #[test]
    fn seasons_years_and_weekdays_follow_calendar_day() {
        let samples = [
            (28, 1, Season::Spring, 28, Weekday::Sunday),
            (29, 1, Season::Summer, 1, Weekday::Monday),
            (56, 1, Season::Summer, 28, Weekday::Sunday),
            (57, 1, Season::Fall, 1, Weekday::Monday),
            (84, 1, Season::Fall, 28, Weekday::Sunday),
            (85, 1, Season::Winter, 1, Weekday::Monday),
            (112, 1, Season::Winter, 28, Weekday::Sunday),
            (113, 2, Season::Spring, 1, Weekday::Monday),
            (224, 2, Season::Winter, 28, Weekday::Sunday),
            (225, 3, Season::Spring, 1, Weekday::Monday),
        ];

        for (calendar_day, year, season, day_of_season, weekday) in samples {
            let date = CalendarDate::from_calendar_day(calendar_day);
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
    fn weekday_keeps_counting_across_seven_day_steps() {
        let names = [
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
            Weekday::Sunday,
        ];
        for day in 1..=21 {
            assert_eq!(
                CalendarDate::from_calendar_day(day).weekday,
                names[(day as usize - 1) % names.len()]
            );
        }
    }

    #[test]
    fn sleep_inside_a_season_keeps_spring() {
        let mut clock = DayClock::default();
        let mut energy = ToolEnergy::default();
        let mut season = Season::Spring;

        let outcome = perform_sleep(&mut clock, &mut energy, &mut season);

        assert_eq!(outcome.calendar_day, 2);
        assert_eq!(season, Season::Spring);
        assert_eq!(outcome.season_rollover, None);
    }

    #[test]
    fn sleep_on_each_season_boundary_reports_the_next_season() {
        let boundaries = [
            (28, Season::Spring, Season::Summer, 1),
            (56, Season::Summer, Season::Fall, 1),
            (84, Season::Fall, Season::Winter, 1),
            (112, Season::Winter, Season::Spring, 2),
        ];

        for (last_day, ended, started, year) in boundaries {
            let mut clock =
                DayClock::from_saved(last_day, crate::core::day_cycle::DayPhase::Evening);
            let mut energy = ToolEnergy::default();
            let mut season = CalendarDate::from_calendar_day(last_day).season;

            let outcome = perform_sleep(&mut clock, &mut energy, &mut season);

            assert_eq!(season, started, "season after day {last_day}");
            assert_eq!(
                outcome.season_rollover,
                Some(SeasonRollover {
                    ended,
                    started,
                    year,
                    day_of_season: 1,
                    weekday: Weekday::Monday,
                    calendar_day: last_day + 1,
                }),
                "rollover leaving day {last_day}"
            );
        }
    }

    #[test]
    fn a_distant_calendar_day_is_year_two_winter() {
        let date = CalendarDate::from_calendar_day(200);
        assert_eq!(date.season, Season::Winter);
        assert_eq!(date.year, 2);
        assert_eq!(date.day_of_season, 4);
        assert_eq!(date.weekday, Weekday::Thursday);
    }
}
