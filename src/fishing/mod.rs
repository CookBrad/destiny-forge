mod cast;
mod logic;
mod plugin;
mod spot;

pub use cast::{tick_active_cast, use_fishing_rod_system, ActiveCast};
pub use logic::{
    resolve_catch, resolve_catch_default, rod_energy_cost, timing_cursor, CatchQuality, CatchResult,
    DEFAULT_ZONE_CENTER, GOOD_ZONE_HALF, PERFECT_ZONE_HALF, TIMING_PERIOD_SECS,
};
pub use plugin::FishingPlugin;
pub use spot::{nearest_spot_distance, spawn_fishing_spot, FishingSpot, DOCK_TILE, POND_TILES};
