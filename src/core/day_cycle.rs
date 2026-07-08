//! Soft day cycle: morning → afternoon → evening → sleep.
//! Real-time advances phases on the homestead; sleep advances the calendar day.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Real seconds spent in each phase before advancing (evening holds until sleep).
pub const PHASE_DURATION_SECS: f32 = 90.0;

/// How many soft-day phase steps a dungeon hunt consumes (large share of the day).
/// Morning → Evening (2 steps). Afternoon → Evening (1 step used of 2). Evening stays.
pub const HUNT_DAY_COST_STEPS: u8 = 2;

/// Default max for the homestead tool energy pool.
pub const TOOL_ENERGY_MAX: f32 = 100.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DayPhase {
    #[default]
    Morning,
    Afternoon,
    Evening,
}

impl DayPhase {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Morning => "Morning",
            Self::Afternoon => "Afternoon",
            Self::Evening => "Evening",
        }
    }

    /// Next phase, or `None` when already evening (player must sleep).
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Morning => Some(Self::Afternoon),
            Self::Afternoon => Some(Self::Evening),
            Self::Evening => None,
        }
    }

    /// Soft sky / clear-color tint for the phase.
    pub fn ambient_clear_color(self) -> Color {
        match self {
            Self::Morning => Color::srgb(0.28, 0.38, 0.42),
            Self::Afternoon => Color::srgb(0.22, 0.28, 0.18),
            Self::Evening => Color::srgb(0.14, 0.12, 0.2),
        }
    }
}

/// Runtime day clock. Calendar day + phase also mirror into `WorldProgress` for saves.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct DayClock {
    pub calendar_day: u32,
    pub phase: DayPhase,
    pub phase_elapsed_secs: f32,
}

impl Default for DayClock {
    fn default() -> Self {
        Self {
            calendar_day: 1,
            phase: DayPhase::Morning,
            phase_elapsed_secs: 0.0,
        }
    }
}

impl DayClock {
    pub fn from_saved(calendar_day: u32, phase: DayPhase) -> Self {
        Self {
            calendar_day: calendar_day.max(1),
            phase,
            phase_elapsed_secs: 0.0,
        }
    }

    pub fn hud_label(&self) -> String {
        format!("Day {} · {}", self.calendar_day, self.phase.display_name())
    }

    /// Advance soft phases on real time. Evening waits for sleep.
    pub fn tick(&mut self, delta_secs: f32) -> bool {
        if delta_secs <= 0.0 {
            return false;
        }

        if self.phase == DayPhase::Evening {
            self.phase_elapsed_secs =
                (self.phase_elapsed_secs + delta_secs).min(PHASE_DURATION_SECS);
            return false;
        }

        self.phase_elapsed_secs += delta_secs;
        if self.phase_elapsed_secs < PHASE_DURATION_SECS {
            return false;
        }

        self.phase_elapsed_secs = 0.0;
        if let Some(next) = self.phase.next() {
            self.phase = next;
            true
        } else {
            false
        }
    }

    /// End the day: next calendar morning. Returns the new day number.
    pub fn sleep(&mut self) -> u32 {
        self.calendar_day = self.calendar_day.saturating_add(1).max(1);
        self.phase = DayPhase::Morning;
        self.phase_elapsed_secs = 0.0;
        self.calendar_day
    }

    pub fn phase_progress(&self) -> f32 {
        (self.phase_elapsed_secs / PHASE_DURATION_SECS).clamp(0.0, 1.0)
    }

    /// Entering a dungeon hunt costs a large share of the day.
    /// Returns true when the phase actually changed.
    pub fn apply_hunt_day_cost(&mut self) -> bool {
        let before = self.phase;
        for _ in 0..HUNT_DAY_COST_STEPS {
            match self.phase.next() {
                Some(next) => {
                    self.phase = next;
                    self.phase_elapsed_secs = 0.0;
                }
                None => break,
            }
        }
        self.phase != before
    }
}

/// Homestead tool energy pool (hoe / water / pick / rod). Combat never drains this.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct ToolEnergy {
    pub current: f32,
    pub max: f32,
}

impl Default for ToolEnergy {
    fn default() -> Self {
        Self {
            current: TOOL_ENERGY_MAX,
            max: TOOL_ENERGY_MAX,
        }
    }
}

impl ToolEnergy {
    pub fn from_saved(current: f32, max: f32) -> Self {
        let max = if max <= 0.0 { TOOL_ENERGY_MAX } else { max };
        Self {
            current: current.clamp(0.0, max),
            max,
        }
    }

    pub fn restore_full(&mut self) {
        self.current = self.max;
    }

    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    pub fn is_empty(&self) -> bool {
        self.current <= 0.0
    }

    pub fn fraction(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.current / self.max).clamp(0.0, 1.0)
        }
    }

    /// Spend tool energy. Returns false when the pool cannot cover the cost (no spend).
    pub fn try_spend(&mut self, amount: f32) -> bool {
        if amount <= 0.0 {
            return true;
        }
        if self.current + f32::EPSILON < amount {
            return false;
        }
        self.current = (self.current - amount).max(0.0);
        true
    }
}

/// Sleep at a bed: advance day, reset phase, restore tool energy.
pub fn perform_sleep(clock: &mut DayClock, energy: &mut ToolEnergy) -> u32 {
    let day = clock.sleep();
    energy.restore_full();
    day
}

/// Tick day clock while exploring the homestead / forest.
pub fn tick_day_clock(
    time: Res<Time>,
    mut clock: ResMut<DayClock>,
    mut profile: ResMut<super::PlayerProfile>,
    mut dirty: ResMut<super::ProfileDirty>,
) {
    if !clock.tick(time.delta_secs()) {
        return;
    }
    profile.calendar_day = clock.calendar_day;
    profile.day_phase = clock.phase;
    dirty.mark();
    info!(
        "Day phase → {} (day {})",
        clock.phase.display_name(),
        clock.calendar_day
    );
}

/// Keep clear color aligned with day phase on the homestead.
pub fn sync_overworld_ambient(
    clock: Res<DayClock>,
    mut clear: ResMut<ClearColor>,
) {
    if clock.is_changed() {
        clear.0 = clock.phase.ambient_clear_color();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phases_advance_morning_to_evening() {
        let mut clock = DayClock::default();
        assert_eq!(clock.phase, DayPhase::Morning);

        assert!(clock.tick(PHASE_DURATION_SECS));
        assert_eq!(clock.phase, DayPhase::Afternoon);

        assert!(clock.tick(PHASE_DURATION_SECS));
        assert_eq!(clock.phase, DayPhase::Evening);

        assert!(!clock.tick(PHASE_DURATION_SECS * 2.0));
        assert_eq!(clock.phase, DayPhase::Evening);
    }

    #[test]
    fn sleep_advances_day_and_resets_morning() {
        let mut clock = DayClock {
            calendar_day: 3,
            phase: DayPhase::Evening,
            phase_elapsed_secs: 40.0,
        };
        let mut energy = ToolEnergy {
            current: 12.0,
            max: 100.0,
        };

        let day = perform_sleep(&mut clock, &mut energy);
        assert_eq!(day, 4);
        assert_eq!(clock.phase, DayPhase::Morning);
        assert_eq!(clock.phase_elapsed_secs, 0.0);
        assert!(energy.is_full());
    }

    #[test]
    fn hud_label_includes_day_and_phase() {
        let clock = DayClock::from_saved(2, DayPhase::Afternoon);
        assert_eq!(clock.hud_label(), "Day 2 · Afternoon");
    }

    #[test]
    fn hunt_day_cost_skips_most_of_morning_day() {
        let mut clock = DayClock::default();
        assert!(clock.apply_hunt_day_cost());
        assert_eq!(clock.phase, DayPhase::Evening);
    }

    #[test]
    fn hunt_from_afternoon_ends_in_evening() {
        let mut clock = DayClock {
            calendar_day: 1,
            phase: DayPhase::Afternoon,
            phase_elapsed_secs: 10.0,
        };
        assert!(clock.apply_hunt_day_cost());
        assert_eq!(clock.phase, DayPhase::Evening);
    }

    #[test]
    fn hunt_from_evening_does_not_change_phase() {
        let mut clock = DayClock {
            calendar_day: 1,
            phase: DayPhase::Evening,
            phase_elapsed_secs: 5.0,
        };
        assert!(!clock.apply_hunt_day_cost());
        assert_eq!(clock.phase, DayPhase::Evening);
    }

    #[test]
    fn tool_energy_spend_and_refuse() {
        let mut energy = ToolEnergy::default();
        assert!(energy.try_spend(40.0));
        assert!((energy.current - 60.0).abs() < 0.01);
        assert!(!energy.try_spend(70.0));
        assert!((energy.current - 60.0).abs() < 0.01);
        assert!(energy.try_spend(60.0));
        assert!(energy.is_empty());
        assert!(!energy.try_spend(1.0));
    }
}
