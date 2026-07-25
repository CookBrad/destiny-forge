use bevy::prelude::*;

use crate::core::GameState;
use crate::ui::forge_window::forge_closed;
use crate::ui::inventory_window::inventory_closed;

use super::buffs::ActiveFoodBuff;
use super::eat::{clear_food_buff_on_hunt_end, eat_food_from_hotbar};
use super::station::{cook_station_input, CookSelectedRecipe};

pub struct CookingPlugin;

impl Plugin for CookingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveFoodBuff>()
            .init_resource::<CookSelectedRecipe>()
            .add_systems(
                Update,
                (cook_station_input, eat_food_from_hotbar)
                    .run_if(in_state(GameState::Overworld))
                    .run_if(inventory_closed)
                    .run_if(forge_closed),
            )
            .add_systems(OnExit(GameState::Dungeon), clear_food_buff_on_hunt_end);
    }
}
