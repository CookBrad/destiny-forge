//! Pure fishing rules: cast timing bar → catch resolution.

use crate::items::MaterialId;

/// Center of the perfect catch zone on the 0..=1 timing bar.
pub const DEFAULT_ZONE_CENTER: f32 = 0.55;
/// Half-width of the perfect (green) zone.
pub const PERFECT_ZONE_HALF: f32 = 0.08;
/// Half-width of the good (yellow) zone.
pub const GOOD_ZONE_HALF: f32 = 0.18;
/// How long one full timing sweep takes (seconds).
pub const TIMING_PERIOD_SECS: f32 = 1.4;
/// Energy spent when casting (mirrors FishingRod).
pub fn rod_energy_cost() -> f32 {
    MaterialId::FishingRod.energy_cost()
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
}
