//! Pure fishing rules: cast state machine, timing cursor, catch resolution.

use crate::items::MaterialId;

/// Center of the perfect catch zone on the 0..=1 timing bar.
pub const DEFAULT_ZONE_CENTER: f32 = 0.55;
/// Half-width of the perfect (green) zone.
pub const PERFECT_ZONE_HALF: f32 = 0.08;
/// Half-width of the good (yellow) zone.
pub const GOOD_ZONE_HALF: f32 = 0.18;
/// How long one full timing sweep takes (seconds).
pub const TIMING_PERIOD_SECS: f32 = 1.4;
/// How long the result banner stays before returning to idle.
pub const RESULT_DISPLAY_SECS: f32 = 1.35;

/// Energy spent when casting (mirrors FishingRod).
pub fn rod_energy_cost() -> f32 {
    MaterialId::FishingRod.energy_cost()
}

/// True when the player can spend energy to cast.
pub fn can_afford_cast(current_energy: f32, cost: f32) -> bool {
    cost <= 0.0 || current_energy + f32::EPSILON >= cost
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatchQuality {
    Perfect,
    Good,
    Miss,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatchResult {
    Caught {
        fish: MaterialId,
        amount: u32,
        quality: CatchQuality,
    },
    Miss,
}

impl CatchResult {
    pub fn quality(&self) -> CatchQuality {
        match self {
            Self::Caught { quality, .. } => *quality,
            Self::Miss => CatchQuality::Miss,
        }
    }

    pub fn feedback_label(&self) -> &'static str {
        match self {
            Self::Caught {
                quality: CatchQuality::Perfect,
                ..
            } => "Perfect catch!",
            Self::Caught {
                quality: CatchQuality::Good,
                ..
            } => "Good catch!",
            Self::Miss
            | Self::Caught {
                quality: CatchQuality::Miss,
                ..
            } => "Miss — fish got away",
        }
    }
}

/// Terminal outcome shown on the result banner (includes cancel).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CastOutcome {
    Catch(CatchResult),
    Cancelled,
}

impl CastOutcome {
    pub fn feedback_label(&self) -> &'static str {
        match self {
            Self::Catch(result) => result.feedback_label(),
            Self::Cancelled => "Cast cancelled",
        }
    }
}

/// Explicit minigame phase — prevents stuck "active" flags.
#[derive(Clone, Debug, PartialEq)]
pub enum CastPhase {
    Idle,
    /// Line is out; timing cursor moves until reel or cancel.
    Waiting {
        elapsed: f32,
        zone_center: f32,
    },
    /// Brief result feedback before returning to idle.
    ShowingResult {
        outcome: CastOutcome,
        remaining: f32,
    },
}

impl Default for CastPhase {
    fn default() -> Self {
        Self::Idle
    }
}

/// Pure cast state machine (no Bevy types).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct CastState {
    pub phase: CastPhase,
}

impl CastState {
    pub fn is_idle(&self) -> bool {
        matches!(self.phase, CastPhase::Idle)
    }

    pub fn is_waiting(&self) -> bool {
        matches!(self.phase, CastPhase::Waiting { .. })
    }

    pub fn is_showing_result(&self) -> bool {
        matches!(self.phase, CastPhase::ShowingResult { .. })
    }

    /// True while the timing bar should be on screen (waiting or result).
    pub fn bar_visible(&self) -> bool {
        !self.is_idle()
    }

    /// Cursor 0..=1 while waiting; None otherwise.
    pub fn cursor(&self) -> Option<f32> {
        match self.phase {
            CastPhase::Waiting { elapsed, .. } => {
                Some(timing_cursor(elapsed, TIMING_PERIOD_SECS))
            }
            _ => None,
        }
    }

    pub fn zone_center(&self) -> Option<f32> {
        match self.phase {
            CastPhase::Waiting { zone_center, .. } => Some(zone_center),
            _ => None,
        }
    }

    pub fn result_label(&self) -> Option<&'static str> {
        match &self.phase {
            CastPhase::ShowingResult { outcome, .. } => Some(outcome.feedback_label()),
            CastPhase::Waiting { .. } => Some("Space — Reel · Esc/Q — Cancel"),
            CastPhase::Idle => None,
        }
    }
}

/// Start a cast from idle. Returns false if already mid-minigame.
pub fn start_cast(state: &mut CastState, zone_center: f32) -> bool {
    if !state.is_idle() {
        return false;
    }
    state.phase = CastPhase::Waiting {
        elapsed: 0.0,
        zone_center: zone_center.clamp(0.15, 0.85),
    };
    true
}

/// Cancel an active wait; shows cancelled result briefly. No-op if not waiting.
pub fn cancel_cast(state: &mut CastState) -> bool {
    if !state.is_waiting() {
        return false;
    }
    state.phase = CastPhase::ShowingResult {
        outcome: CastOutcome::Cancelled,
        remaining: RESULT_DISPLAY_SECS,
    };
    true
}

/// Reel during Waiting. Transitions to ShowingResult and returns the catch.
/// Returns None if not waiting (no soft-lock side effects).
pub fn reel_cast(state: &mut CastState) -> Option<CatchResult> {
    let (elapsed, zone_center) = match state.phase {
        CastPhase::Waiting {
            elapsed,
            zone_center,
        } => (elapsed, zone_center),
        _ => return None,
    };
    let cursor = timing_cursor(elapsed, TIMING_PERIOD_SECS);
    let result = resolve_catch(
        cursor,
        zone_center,
        PERFECT_ZONE_HALF,
        GOOD_ZONE_HALF,
    );
    state.phase = CastPhase::ShowingResult {
        outcome: CastOutcome::Catch(result.clone()),
        remaining: RESULT_DISPLAY_SECS,
    };
    Some(result)
}

/// Advance timers. When result duration elapses, return to Idle.
pub fn tick_cast(state: &mut CastState, delta_secs: f32) {
    if delta_secs <= 0.0 {
        return;
    }
    match &mut state.phase {
        CastPhase::Waiting { elapsed, .. } => {
            *elapsed += delta_secs;
        }
        CastPhase::ShowingResult { remaining, .. } => {
            *remaining -= delta_secs;
            if *remaining <= 0.0 {
                state.phase = CastPhase::Idle;
            }
        }
        CastPhase::Idle => {}
    }
}

/// Force clear to idle (e.g. leaving overworld). Never leaves a stuck wait.
pub fn force_idle(state: &mut CastState) {
    state.phase = CastPhase::Idle;
}

/// Oscillating cursor on 0..=1 (ping-pong).
pub fn timing_cursor(elapsed_secs: f32, period_secs: f32) -> f32 {
    let period = period_secs.max(0.01);
    let t = (elapsed_secs / period) % 2.0;
    if t < 1.0 {
        t
    } else {
        2.0 - t
    }
}

/// Resolve a reel attempt at `cursor` (0..=1) against the catch zones.
pub fn resolve_catch(
    cursor: f32,
    zone_center: f32,
    perfect_half: f32,
    good_half: f32,
) -> CatchResult {
    let cursor = cursor.clamp(0.0, 1.0);
    let dist = (cursor - zone_center).abs();
    if dist <= perfect_half {
        CatchResult::Caught {
            fish: MaterialId::RiverFish,
            amount: 2,
            quality: CatchQuality::Perfect,
        }
    } else if dist <= good_half {
        CatchResult::Caught {
            fish: MaterialId::RiverFish,
            amount: 1,
            quality: CatchQuality::Good,
        }
    } else {
        CatchResult::Miss
    }
}

/// Convenience: resolve using default zone sizes.
pub fn resolve_catch_default(cursor: f32) -> CatchResult {
    resolve_catch(
        cursor,
        DEFAULT_ZONE_CENTER,
        PERFECT_ZONE_HALF,
        GOOD_ZONE_HALF,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_center_yields_fish() {
        let result = resolve_catch_default(DEFAULT_ZONE_CENTER);
        assert_eq!(
            result,
            CatchResult::Caught {
                fish: MaterialId::RiverFish,
                amount: 2,
                quality: CatchQuality::Perfect,
            }
        );
    }

    #[test]
    fn edge_of_good_zone_catches_one() {
        let cursor = DEFAULT_ZONE_CENTER + GOOD_ZONE_HALF - 0.001;
        let result = resolve_catch_default(cursor);
        assert!(matches!(
            result,
            CatchResult::Caught {
                amount: 1,
                quality: CatchQuality::Good,
                ..
            }
        ));
    }

    #[test]
    fn far_timing_is_miss() {
        assert_eq!(resolve_catch_default(0.0), CatchResult::Miss);
        assert_eq!(resolve_catch_default(1.0), CatchResult::Miss);
    }

    #[test]
    fn timing_cursor_ping_pongs() {
        assert!((timing_cursor(0.0, 1.0) - 0.0).abs() < 0.01);
        assert!((timing_cursor(0.5, 1.0) - 0.5).abs() < 0.01);
        assert!((timing_cursor(1.0, 1.0) - 1.0).abs() < 0.01);
        assert!((timing_cursor(1.5, 1.0) - 0.5).abs() < 0.01);
        assert!((timing_cursor(2.0, 1.0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn energy_gate_blocks_insufficient_pool() {
        let cost = rod_energy_cost();
        assert!(can_afford_cast(cost, cost));
        assert!(can_afford_cast(cost + 1.0, cost));
        assert!(!can_afford_cast(cost - 0.5, cost));
        assert!(!can_afford_cast(0.0, cost));
    }

    #[test]
    fn state_machine_start_reel_perfect_without_stuck_flag() {
        let mut state = CastState::default();
        assert!(state.is_idle());
        assert!(start_cast(&mut state, DEFAULT_ZONE_CENTER));
        assert!(state.is_waiting());
        assert!(state.bar_visible());

        // At elapsed 0, cursor is 0 → miss if reeled immediately… advance to zone center.
        // cursor = elapsed/period for first half; center 0.55 → elapsed = 0.55 * 1.4
        if let CastPhase::Waiting { elapsed, .. } = &mut state.phase {
            *elapsed = DEFAULT_ZONE_CENTER * TIMING_PERIOD_SECS;
        }
        let result = reel_cast(&mut state).expect("reel while waiting");
        assert!(matches!(
            result,
            CatchResult::Caught {
                quality: CatchQuality::Perfect,
                amount: 2,
                ..
            }
        ));
        assert!(state.is_showing_result());
        assert!(!state.is_waiting());

        // Result expires → idle (no stuck active)
        tick_cast(&mut state, RESULT_DISPLAY_SECS + 0.1);
        assert!(state.is_idle());
        assert!(!state.bar_visible());
    }

    #[test]
    fn cancel_from_waiting_clears_without_stuck_active() {
        let mut state = CastState::default();
        assert!(start_cast(&mut state, DEFAULT_ZONE_CENTER));
        assert!(cancel_cast(&mut state));
        assert!(state.is_showing_result());
        assert_eq!(
            state.result_label(),
            Some("Cast cancelled")
        );
        tick_cast(&mut state, RESULT_DISPLAY_SECS + 0.05);
        assert!(state.is_idle());
        // Cancel again while idle is a no-op
        assert!(!cancel_cast(&mut state));
        assert!(state.is_idle());
    }

    #[test]
    fn cannot_double_start_or_reel_when_idle() {
        let mut state = CastState::default();
        assert!(start_cast(&mut state, DEFAULT_ZONE_CENTER));
        assert!(!start_cast(&mut state, DEFAULT_ZONE_CENTER));
        assert!(reel_cast(&mut state).is_some());
        // Now showing result — reel is no-op
        assert!(reel_cast(&mut state).is_none());
        force_idle(&mut state);
        assert!(reel_cast(&mut state).is_none());
    }

    #[test]
    fn tick_advances_cursor_during_wait() {
        let mut state = CastState::default();
        start_cast(&mut state, DEFAULT_ZONE_CENTER);
        let c0 = state.cursor().unwrap();
        tick_cast(&mut state, 0.2);
        let c1 = state.cursor().unwrap();
        assert!(c1 > c0);
    }

    #[test]
    fn miss_and_good_paths_via_state_machine() {
        let mut state = CastState::default();
        start_cast(&mut state, DEFAULT_ZONE_CENTER);
        // elapsed 0 → cursor 0 → miss
        let miss = reel_cast(&mut state).unwrap();
        assert_eq!(miss, CatchResult::Miss);
        force_idle(&mut state);

        start_cast(&mut state, DEFAULT_ZONE_CENTER);
        // Good zone edge: just outside perfect
        let good_cursor = DEFAULT_ZONE_CENTER + PERFECT_ZONE_HALF + 0.02;
        if let CastPhase::Waiting { elapsed, .. } = &mut state.phase {
            *elapsed = good_cursor * TIMING_PERIOD_SECS;
        }
        let good = reel_cast(&mut state).unwrap();
        assert!(matches!(
            good,
            CatchResult::Caught {
                quality: CatchQuality::Good,
                amount: 1,
                ..
            }
        ));
    }
}
