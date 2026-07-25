mod animation;
mod cast;
mod logic;
mod plugin;
mod spot;
mod ui;

pub use animation::{cleanup_fishing_animation, sync_fishing_animation, FishingBobber};
pub use cast::{clear_cast_on_zone_exit, tick_active_cast, use_fishing_rod_system, ActiveCast};
pub use logic::{
    apply_bar_input, can_afford_cast, cancel_cast, catch_yield, fish_inside_bar, force_idle,
    rod_energy_cost, start_cast, tick_cast, tick_fight, tick_fish_motion, tick_progress, CastPhase,
    CastState, FightSim, FishOutcome, FishingAnimKind, BAR_HEIGHT, BAR_RAISE_SPEED,
    INITIAL_PROGRESS, PROGRESS_DRAIN_RATE, PROGRESS_FILL_RATE, RESULT_DISPLAY_SECS,
};
pub use plugin::FishingPlugin;
pub use spot::{nearest_spot_distance, spawn_fishing_spot, FishingSpot, DOCK_TILE, POND_TILES};
pub use ui::{cleanup_fishing_bar, sync_fishing_bar_ui, FishingBarRoot};
