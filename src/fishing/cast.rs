//! Cast + reel minigame driven from the hotbar fishing rod.

use bevy::prelude::*;

use crate::core::{ProfileDirty, ToolEnergy};
use crate::farming::HomesteadHotbar;
use crate::graphics::TILE;
use crate::items::{Inventory, MaterialId};
use crate::overworld::movement::OverworldPlayer;
use crate::ui::forge_window::ForgeWindowOpen;
use crate::ui::inventory_window::InventoryWindowOpen;

use super::logic::{
    resolve_catch_default, rod_energy_cost, timing_cursor, CatchResult, TIMING_PERIOD_SECS,
};
use super::spot::FishingSpot;

const SPOT_RANGE: f32 = TILE * 2.0;

/// Active cast while the timing bar is running.
#[derive(Resource, Clone, Debug, Default)]
pub struct ActiveCast {
    pub active: bool,
    pub elapsed: f32,
    /// Energy already spent for this cast.
    pub energy_spent: bool,
}

impl ActiveCast {
    pub fn clear(&mut self) {
        self.active = false;
        self.elapsed = 0.0;
        self.energy_spent = false;
    }

    pub fn begin(&mut self) {
        self.active = true;
        self.elapsed = 0.0;
        self.energy_spent = true;
    }

    pub fn cursor(&self) -> f32 {
        timing_cursor(self.elapsed, TIMING_PERIOD_SECS)
    }
}

pub fn tick_active_cast(time: Res<Time>, mut cast: ResMut<ActiveCast>) {
    if cast.active {
        cast.elapsed += time.delta_secs();
    }
}

pub fn use_fishing_rod_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    inventory_open: Res<InventoryWindowOpen>,
    forge_open: Res<ForgeWindowOpen>,
    hotbar: Res<HomesteadHotbar>,
    mut energy: ResMut<ToolEnergy>,
    mut inventory: ResMut<Inventory>,
    mut dirty: ResMut<ProfileDirty>,
    mut cast: ResMut<ActiveCast>,
    player: Query<&Transform, With<OverworldPlayer>>,
    spots: Query<&Transform, With<FishingSpot>>,
) {
    if inventory_open.0 || forge_open.0 {
        return;
    }

    let use_pressed =
        keyboard.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left);
    if !use_pressed {
        return;
    }

    // If a cast is active, any Space reels — even if hotbar changed mid-cast.
    if cast.active {
        reel(&mut cast, &mut inventory, &mut dirty);
        return;
    }

    let Some(material) = hotbar.selected_entry().material() else {
        return;
    };
    if material != MaterialId::FishingRod {
        return;
    }

    if inventory.count(MaterialId::FishingRod) == 0 {
        info!("Fishing rod not in inventory.");
        return;
    }

    let Ok(transform) = player.get_single() else {
        return;
    };
    let player_pos = transform.translation.truncate();

    let near = spots.iter().any(|t| {
        player_pos.distance(t.translation.truncate()) <= SPOT_RANGE
    });
    if !near {
        info!("Move closer to the fishing dock to cast.");
        return;
    }

    let cost = rod_energy_cost();
    if !energy.try_spend(cost) {
        info!("Not enough energy to fish.");
        return;
    }

    cast.begin();
    info!("Cast! Press Space again when the bite hits the green zone.");
}

fn reel(cast: &mut ActiveCast, inventory: &mut Inventory, dirty: &mut ProfileDirty) {
    let cursor = cast.cursor();
    cast.clear();
    match resolve_catch_default(cursor) {
        CatchResult::Caught {
            fish,
            amount,
            quality,
        } => {
            let left = inventory.try_add(fish, amount);
            if left > 0 {
                warn!("Inventory full — lost fish.");
            }
            dirty.mark();
            let q = match quality {
                super::logic::CatchQuality::Perfect => "Perfect",
                super::logic::CatchQuality::Good => "Good",
                super::logic::CatchQuality::Miss => "Miss",
            };
            info!("{q} catch! {amount}× {}.", fish.display_name());
        }
        CatchResult::Miss => {
            info!("The fish got away (cursor {:.0}%).", cursor * 100.0);
        }
    }
}
