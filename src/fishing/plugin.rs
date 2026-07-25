use bevy::prelude::*;

use crate::core::GameState;

use super::animation::{cleanup_fishing_animation, sync_fishing_animation};
use super::cast::{clear_cast_on_zone_exit, tick_active_cast, use_fishing_rod_system, ActiveCast};
use super::ui::{cleanup_fishing_bar, sync_fishing_bar_ui};

fn fishing_zone() -> impl Condition<()> {
    in_state(GameState::Overworld).or(in_state(GameState::Lake))
}

pub struct FishingPlugin;

impl Plugin for FishingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveCast>()
            .add_systems(
                Update,
                (
                    tick_active_cast,
                    use_fishing_rod_system,
                    sync_fishing_bar_ui,
                    sync_fishing_animation,
                )
                    .chain()
                    .run_if(fishing_zone()),
            )
            .add_systems(
                OnExit(GameState::Overworld),
                (
                    clear_cast_on_zone_exit,
                    cleanup_fishing_bar,
                    cleanup_fishing_animation,
                ),
            )
            .add_systems(
                OnExit(GameState::Lake),
                (
                    clear_cast_on_zone_exit,
                    cleanup_fishing_bar,
                    cleanup_fishing_animation,
                ),
            );
    }
}
