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
    can_afford_cast, cancel_cast, force_idle, reel_cast, rod_energy_cost, start_cast, tick_cast,
    CastState, CatchQuality, CatchResult, DEFAULT_ZONE_CENTER,
};
use super::spot::FishingSpot;

const SPOT_RANGE: f32 = TILE * 2.0;

/// Runtime cast resource wrapping the pure state machine.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct ActiveCast {
    pub state: CastState,
}

impl ActiveCast {
    pub fn is_waiting(&self) -> bool {
        self.state.is_waiting()
    }

    pub fn bar_visible(&self) -> bool {
        self.state.bar_visible()
    }

    pub fn clear(&mut self) {
        force_idle(&mut self.state);
    }
}

pub fn tick_active_cast(time: Res<Time>, mut cast: ResMut<ActiveCast>) {
    tick_cast(&mut cast.state, time.delta_secs());
}

/// Cast / reel / cancel input for the fishing rod minigame.
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
        // Leaving UI open mid-cast should not soft-lock: cancel wait.
        if cast.is_waiting() {
            cancel_cast(&mut cast.state);
        }
        return;
    }

    // --- Cancel (waiting only): Esc, Q, or right-click ---
    let cancel_pressed = keyboard.just_pressed(KeyCode::Escape)
        || keyboard.just_pressed(KeyCode::KeyQ)
        || mouse.just_pressed(MouseButton::Right);
    if cast.is_waiting() && cancel_pressed {
        cancel_cast(&mut cast.state);
        info!("Cast cancelled.");
        return;
    }

    let use_pressed =
        keyboard.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left);

    // --- Reel while waiting (any hotbar selection) ---
    if cast.is_waiting() {
        if use_pressed {
            if let Some(result) = reel_cast(&mut cast.state) {
                apply_catch_result(result, &mut inventory, &mut dirty);
            }
        }
        return;
    }

    // Showing result or idle: Space only starts a new cast when idle + rod ready.
    if !use_pressed || !cast.state.is_idle() {
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

    let near = spots
        .iter()
        .any(|t| player_pos.distance(t.translation.truncate()) <= SPOT_RANGE);
    if !near {
        info!("Move closer to the fishing dock to cast.");
        return;
    }

    let cost = rod_energy_cost();
    if !can_afford_cast(energy.current, cost) || !energy.try_spend(cost) {
        info!("Not enough energy to fish.");
        return;
    }

    if start_cast(&mut cast.state, DEFAULT_ZONE_CENTER) {
        info!("Cast! Hit Space in the green zone · Esc/Q cancel.");
    }
}

fn apply_catch_result(
    result: CatchResult,
    inventory: &mut Inventory,
    dirty: &mut ProfileDirty,
) {
    match result {
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
                CatchQuality::Perfect => "Perfect",
                CatchQuality::Good => "Good",
                CatchQuality::Miss => "Miss",
            };
            info!("{q} catch! {amount}× {}.", fish.display_name());
        }
        CatchResult::Miss => {
            info!("The fish got away.");
        }
    }
}

/// Clear cast when leaving the overworld so nothing stays stuck.
pub fn clear_cast_on_overworld_exit(mut cast: ResMut<ActiveCast>) {
    cast.clear();
}
