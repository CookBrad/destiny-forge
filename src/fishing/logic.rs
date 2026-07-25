//! Stardew Valley–style fishing: vertical green bar vs moving fish + catch progress.
//! Pure simulation only — no Bevy types.

use crate::items::MaterialId;

/// Default height of the player's green catch bar (0..=1 axis).
pub const BAR_HEIGHT: f32 = 0.22;
/// How fast the bar rises while holding (units per second).
pub const BAR_RAISE_SPEED: f32 = 0.95;
/// How fast the bar falls when not holding.
pub const BAR_FALL_SPEED: f32 = 0.75;
/// Progress fill rate while fish is inside the bar.
pub const PROGRESS_FILL_RATE: f32 = 0.28;
/// Progress drain rate while fish is outside the bar.
pub const PROGRESS_DRAIN_RATE: f32 = 0.22;
/// Starting catch progress when the fight begins (room for one mistake).
pub const INITIAL_PROGRESS: f32 = 0.28;
/// Cast animation / cast phase duration.
pub const CAST_PHASE_SECS: f32 = 0.55;
/// Wait-for-bite phase duration before the fight starts.
pub const BITE_WAIT_SECS: f32 = 0.9;
/// Result banner duration.
pub const RESULT_DISPLAY_SECS: f32 = 1.4;
/// Base fish swim speed magnitude.
pub const FISH_SPEED: f32 = 0.55;

/// Energy spent when casting (mirrors FishingRod).
pub fn rod_energy_cost() -> f32 {
    MaterialId::FishingRod.energy_cost()
}

pub fn can_afford_cast(current_energy: f32, cost: f32) -> bool {
    cost <= 0.0 || current_energy + f32::EPSILON >= cost
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FishOutcome {
    Caught,
    Escaped,
    Cancelled,
}

impl FishOutcome {
    pub fn feedback_label(self) -> &'static str {
        match self {
            Self::Caught => "Caught! River Fish",
            Self::Escaped => "The fish got away…",
            Self::Cancelled => "Cast cancelled",
        }
    }

    pub fn is_success(self) -> bool {
        matches!(self, Self::Caught)
    }
}

/// Live contest numbers (axis: 0 = bottom, 1 = top).
#[derive(Clone, Debug, PartialEq)]
pub struct FightSim {
    /// Bottom edge of the green bar.
    pub bar_bottom: f32,
    pub bar_height: f32,
    /// Fish center Y on the axis.
    pub fish_y: f32,
    pub fish_vel: f32,
    /// Catch meter 0..=1.
    pub progress: f32,
    pub elapsed: f32,
}

impl FightSim {
    pub fn new() -> Self {
        Self {
            bar_bottom: 0.35,
            bar_height: BAR_HEIGHT,
            fish_y: 0.5,
            fish_vel: FISH_SPEED,
            progress: INITIAL_PROGRESS,
            elapsed: 0.0,
        }
    }

    pub fn bar_top(&self) -> f32 {
        (self.bar_bottom + self.bar_height).min(1.0)
    }

    pub fn fish_in_bar(&self) -> bool {
        fish_inside_bar(self.fish_y, self.bar_bottom, self.bar_height)
    }
}

impl Default for FightSim {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CastPhase {
    Idle,
    /// Line going out / cast animation.
    Casting { remaining: f32 },
    /// Bobber in water, waiting for bite.
    WaitingBite { remaining: f32 },
    /// Active Stardew-style contest.
    Fighting(FightSim),
    ShowingResult {
        outcome: FishOutcome,
        remaining: f32,
    },
}

impl Default for CastPhase {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct CastState {
    pub phase: CastPhase,
}

impl CastState {
    pub fn is_idle(&self) -> bool {
        matches!(self.phase, CastPhase::Idle)
    }

    pub fn is_fighting(&self) -> bool {
        matches!(self.phase, CastPhase::Fighting(_))
    }

    pub fn is_busy(&self) -> bool {
        !self.is_idle()
    }

    /// True while minigame UI should show (fight or result; cast/bite use lighter UI).
    pub fn bar_visible(&self) -> bool {
        matches!(
            self.phase,
            CastPhase::Fighting(_) | CastPhase::ShowingResult { .. }
        )
    }

    /// Any active cast (including cast/bite wait) — blocks title Esc, etc.
    pub fn minigame_active(&self) -> bool {
        self.is_busy()
    }

    pub fn fight(&self) -> Option<&FightSim> {
        match &self.phase {
            CastPhase::Fighting(sim) => Some(sim),
            _ => None,
        }
    }

    pub fn result_label(&self) -> Option<&'static str> {
        match &self.phase {
            CastPhase::Casting { .. } => Some("Casting…"),
            CastPhase::WaitingBite { .. } => Some("Waiting for a bite…"),
            CastPhase::Fighting(_) => Some("Hold Space — raise bar · release — lower"),
            CastPhase::ShowingResult { outcome, .. } => Some(outcome.feedback_label()),
            CastPhase::Idle => None,
        }
    }

    pub fn anim_kind(&self) -> FishingAnimKind {
        match self.phase {
            CastPhase::Idle => FishingAnimKind::None,
            CastPhase::Casting { .. } => FishingAnimKind::Cast,
            CastPhase::WaitingBite { .. } => FishingAnimKind::Waiting,
            CastPhase::Fighting(_) => FishingAnimKind::Fighting,
            CastPhase::ShowingResult {
                outcome: FishOutcome::Caught,
                ..
            } => FishingAnimKind::Success,
            CastPhase::ShowingResult { .. } => FishingAnimKind::Fail,
        }
    }
}

/// Visual phase for cast/bite animation hooks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FishingAnimKind {
    None,
    Cast,
    Waiting,
    Fighting,
    Success,
    Fail,
}

// --- pure geometry / motion -------------------------------------------------

pub fn fish_inside_bar(fish_y: f32, bar_bottom: f32, bar_height: f32) -> bool {
    let y = fish_y.clamp(0.0, 1.0);
    let top = (bar_bottom + bar_height).min(1.0);
    y >= bar_bottom && y <= top
}

/// Raise (hold) or lower (release) the green bar; keeps it fully on the 0..=1 axis.
pub fn apply_bar_input(sim: &mut FightSim, holding: bool, dt: f32) {
    if dt <= 0.0 {
        return;
    }
    if holding {
        sim.bar_bottom += BAR_RAISE_SPEED * dt;
    } else {
        sim.bar_bottom -= BAR_FALL_SPEED * dt;
    }
    let max_bottom = (1.0 - sim.bar_height).max(0.0);
    sim.bar_bottom = sim.bar_bottom.clamp(0.0, max_bottom);
}

/// Deterministic fish motion: constant velocity with periodic velocity flips.
pub fn tick_fish_motion(sim: &mut FightSim, dt: f32) {
    if dt <= 0.0 {
        return;
    }
    sim.elapsed += dt;

    // Flip direction on a predictable schedule so tests are stable.
    let flip_period = 0.85;
    let flips = (sim.elapsed / flip_period).floor() as i32;
    let sign = if flips % 2 == 0 { 1.0 } else { -1.0 };
    // Slight speed pulse for readability.
    let pulse = 1.0 + 0.15 * (sim.elapsed * 3.0).sin();
    sim.fish_vel = FISH_SPEED * sign * pulse;

    sim.fish_y += sim.fish_vel * dt;
    if sim.fish_y < 0.0 {
        sim.fish_y = 0.0;
        sim.fish_vel = sim.fish_vel.abs();
    } else if sim.fish_y > 1.0 {
        sim.fish_y = 1.0;
        sim.fish_vel = -sim.fish_vel.abs();
    }
}

/// Update progress from fish-in-bar; returns Some(outcome) when fight ends.
pub fn tick_progress(sim: &mut FightSim, dt: f32) -> Option<FishOutcome> {
    if dt <= 0.0 {
        return None;
    }
    if sim.fish_in_bar() {
        sim.progress += PROGRESS_FILL_RATE * dt;
    } else {
        sim.progress -= PROGRESS_DRAIN_RATE * dt;
    }
    if sim.progress >= 1.0 {
        sim.progress = 1.0;
        return Some(FishOutcome::Caught);
    }
    if sim.progress <= 0.0 {
        sim.progress = 0.0;
        return Some(FishOutcome::Escaped);
    }
    None
}

/// One fight frame: bar input + fish + progress.
pub fn tick_fight(sim: &mut FightSim, holding: bool, dt: f32) -> Option<FishOutcome> {
    apply_bar_input(sim, holding, dt);
    tick_fish_motion(sim, dt);
    tick_progress(sim, dt)
}

// --- state machine ----------------------------------------------------------

/// Begin cast from idle. Returns false if already mid-minigame.
pub fn start_cast(state: &mut CastState) -> bool {
    if !state.is_idle() {
        return false;
    }
    state.phase = CastPhase::Casting {
        remaining: CAST_PHASE_SECS,
    };
    true
}

/// Cancel any non-idle phase into a cancelled result (or idle if already result).
pub fn cancel_cast(state: &mut CastState) -> bool {
    match state.phase {
        CastPhase::Idle | CastPhase::ShowingResult { .. } => false,
        _ => {
            state.phase = CastPhase::ShowingResult {
                outcome: FishOutcome::Cancelled,
                remaining: RESULT_DISPLAY_SECS,
            };
            true
        }
    }
}

pub fn force_idle(state: &mut CastState) {
    state.phase = CastPhase::Idle;
}

/// Advance timers / fight. `holding` only matters during Fighting.
/// Returns Some when a catch/escape just resolved this tick (for inventory grant).
pub fn tick_cast(state: &mut CastState, holding: bool, dt: f32) -> Option<FishOutcome> {
    if dt <= 0.0 {
        return None;
    }

    match &mut state.phase {
        CastPhase::Idle => None,
        CastPhase::Casting { remaining } => {
            *remaining -= dt;
            if *remaining <= 0.0 {
                state.phase = CastPhase::WaitingBite {
                    remaining: BITE_WAIT_SECS,
                };
            }
            None
        }
        CastPhase::WaitingBite { remaining } => {
            *remaining -= dt;
            if *remaining <= 0.0 {
                state.phase = CastPhase::Fighting(FightSim::new());
            }
            None
        }
        CastPhase::Fighting(sim) => {
            if let Some(outcome) = tick_fight(sim, holding, dt) {
                state.phase = CastPhase::ShowingResult {
                    outcome,
                    remaining: RESULT_DISPLAY_SECS,
                };
                Some(outcome)
            } else {
                None
            }
        }
        CastPhase::ShowingResult { remaining, .. } => {
            *remaining -= dt;
            if *remaining <= 0.0 {
                state.phase = CastPhase::Idle;
            }
            None
        }
    }
}

/// Inventory yield for a successful catch.
pub fn catch_yield(outcome: FishOutcome) -> Option<(MaterialId, u32)> {
    match outcome {
        FishOutcome::Caught => Some((MaterialId::RiverFish, 1)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hold_raises_bar_release_lowers() {
        let mut sim = FightSim::new();
        let start = sim.bar_bottom;
        apply_bar_input(&mut sim, true, 0.2);
        assert!(sim.bar_bottom > start);
        let raised = sim.bar_bottom;
        apply_bar_input(&mut sim, false, 0.2);
        assert!(sim.bar_bottom < raised);
    }

    #[test]
    fn bar_stays_within_axis() {
        let mut sim = FightSim::new();
        apply_bar_input(&mut sim, true, 10.0);
        assert!(sim.bar_bottom + sim.bar_height <= 1.0 + 1e-4);
        apply_bar_input(&mut sim, false, 10.0);
        assert!(sim.bar_bottom >= -1e-4);
    }

    #[test]
    fn fish_position_advances_over_ticks() {
        let mut sim = FightSim::new();
        let y0 = sim.fish_y;
        tick_fish_motion(&mut sim, 0.1);
        assert!((sim.fish_y - y0).abs() > 1e-4);
        // Multiple ticks stay in range
        for _ in 0..40 {
            tick_fish_motion(&mut sim, 0.05);
            assert!((0.0..=1.0).contains(&sim.fish_y));
        }
    }

    #[test]
    fn progress_fills_when_fish_inside_drains_outside() {
        let mut sim = FightSim::new();
        // Place fish inside bar
        sim.fish_y = sim.bar_bottom + sim.bar_height * 0.5;
        sim.fish_vel = 0.0;
        let p0 = sim.progress;
        // Only progress tick (no fish motion)
        assert!(tick_progress(&mut sim, 0.3).is_none());
        assert!(sim.progress > p0);

        // Move fish outside
        sim.fish_y = 0.99;
        let p1 = sim.progress;
        assert!(tick_progress(&mut sim, 0.3).is_none());
        assert!(sim.progress < p1);
    }

    #[test]
    fn full_progress_is_catch() {
        let mut sim = FightSim::new();
        sim.fish_y = sim.bar_bottom + 0.05;
        sim.progress = 0.95;
        let out = tick_progress(&mut sim, 1.0);
        assert_eq!(out, Some(FishOutcome::Caught));
        assert!((sim.progress - 1.0).abs() < 1e-4);
    }

    #[test]
    fn empty_progress_is_escape() {
        let mut sim = FightSim::new();
        sim.fish_y = 0.99;
        sim.progress = 0.05;
        let out = tick_progress(&mut sim, 1.0);
        assert_eq!(out, Some(FishOutcome::Escaped));
        assert!(sim.progress <= 0.0);
    }

    #[test]
    fn cancel_and_force_idle_leave_no_stuck_active() {
        let mut state = CastState::default();
        assert!(start_cast(&mut state));
        assert!(state.minigame_active());
        assert!(cancel_cast(&mut state));
        assert!(matches!(
            state.phase,
            CastPhase::ShowingResult {
                outcome: FishOutcome::Cancelled,
                ..
            }
        ));
        tick_cast(&mut state, false, RESULT_DISPLAY_SECS + 0.1);
        assert!(state.is_idle());
        assert!(!cancel_cast(&mut state));

        start_cast(&mut state);
        force_idle(&mut state);
        assert!(state.is_idle());
        assert!(!state.minigame_active());
    }

    #[test]
    fn state_machine_cast_to_fight_to_catch() {
        let mut state = CastState::default();
        assert!(start_cast(&mut state));
        assert!(matches!(state.phase, CastPhase::Casting { .. }));
        assert_eq!(state.anim_kind(), FishingAnimKind::Cast);

        tick_cast(&mut state, false, CAST_PHASE_SECS + 0.01);
        assert!(matches!(state.phase, CastPhase::WaitingBite { .. }));
        assert_eq!(state.anim_kind(), FishingAnimKind::Waiting);

        tick_cast(&mut state, false, BITE_WAIT_SECS + 0.01);
        assert!(state.is_fighting());
        assert!(state.bar_visible());

        // Force catch: keep fish centered in bar each step while progress fills.
        let mut outcome = None;
        for _ in 0..40 {
            if let CastPhase::Fighting(sim) = &mut state.phase {
                sim.fish_y = sim.bar_bottom + sim.bar_height * 0.5;
            }
            outcome = tick_cast(&mut state, true, 0.15);
            if outcome.is_some() {
                break;
            }
        }
        assert_eq!(outcome, Some(FishOutcome::Caught));
        assert!(matches!(
            state.phase,
            CastPhase::ShowingResult {
                outcome: FishOutcome::Caught,
                ..
            }
        ));
        assert_eq!(
            catch_yield(FishOutcome::Caught),
            Some((MaterialId::RiverFish, 1))
        );
    }

    #[test]
    fn energy_gate_and_cannot_double_start() {
        let cost = rod_energy_cost();
        assert!(can_afford_cast(cost, cost));
        assert!(!can_afford_cast(0.0, cost));
        let mut state = CastState::default();
        assert!(start_cast(&mut state));
        assert!(!start_cast(&mut state));
    }

    #[test]
    fn fish_inside_bar_geometry() {
        assert!(fish_inside_bar(0.5, 0.4, 0.2));
        assert!(!fish_inside_bar(0.3, 0.4, 0.2));
        assert!(!fish_inside_bar(0.7, 0.4, 0.2));
    }
}
