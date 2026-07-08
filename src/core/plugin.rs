use bevy::prelude::*;

use super::day_cycle::{tick_day_clock, DayClock, ToolEnergy};
use super::memory::MemoryPlugin;
use super::{DungeonPlayState, GameState};

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MemoryPlugin)
            .init_resource::<DayClock>()
            .init_resource::<ToolEnergy>()
            .init_state::<GameState>()
            .add_sub_state::<DungeonPlayState>()
            .add_systems(
                Update,
                tick_day_clock.run_if(
                    in_state(GameState::Overworld).or(in_state(GameState::Forest)),
                ),
            );
    }
}