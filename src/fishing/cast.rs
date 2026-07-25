//! Cast / fight / cancel wiring for Stardew-style fishing.

use bevy::prelude::*;

use crate::core::{ProfileDirty, ToolEnergy};
use crate::farming::HomesteadHotbar;
use crate::graphics::TILE;
use crate::items::{Inventory, MaterialId};
use crate::overworld::movement::OverworldPlayer;
use crate::ui::forge_window::ForgeWindowOpen;
use crate::ui::inventory_window::InventoryWindowOpen;

use super::logic::{
    can_afford_cast, cancel_cast, catch_yield, force_idle, rod_energy_cost, start_cast, tick_cast,
    CastState, FishOutcome,
};
use super::spot::FishingSpot;

const SPOT_RANGE: f32 = TILE * 2.2;

#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct ActiveCast {
    pub state: CastState,
    /// Hold Space / LMB to raise the green bar during a fight.
    pub holding: bool,
}

impl ActiveCast {
    pub fn minigame_active(&self) -> bool {
        self.state.minigame_active()
    }

    pub fn is_fighting(&self) -> bool {
        self.state.is_fighting()
    }

    pub fn bar_visible(&self) -> bool {
        self.state.bar_visible()
    }

    pub fn clear(&mut self) {
        force_idle(&mut self.state);
        self.holding = false;
    }
}

pub fn tick_active_cast(time: Res<Time>, mut cast: ResMut<ActiveCast>, mut inventory: ResMut<Inventory>, mut dirty: ResMut<ProfileDirty>) {
    let holding = cast.holding;
    if let Some(outcome) = tick_cast(&mut cast.state, holding, time.delta_secs()) {
        apply_outcome(outcome, &mut inventory, &mut dirty);
    }
    if cast.state.is_idle() {
        cast.holding = false;
    }
}

pub fn use_fishing_rod_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    inventory_open: Res<InventoryWindowOpen>,
    forge_open: Res<ForgeWindowOpen>,
    hotbar: Res<HomesteadHotbar>,
    mut energy: ResMut<ToolEnergy>,
    inventory: Res<Inventory>,
    mut cast: ResMut<ActiveCast>,
    player: Query<&Transform, With<OverworldPlayer>>,
    spots: Query<&Transform, With<FishingSpot>>,
) {
    if inventory_open.0 || forge_open.0 {
        if cast.minigame_active() {
            if cancel_cast(&mut cast.state) {
                cast.holding = false;
            }
        }
        return;
    }

    // Hold state for the fight (Space or LMB held).
    cast.holding = keyboard.pressed(KeyCode::Space) || mouse.pressed(MouseButton::Left);

    let cancel_pressed = keyboard.just_pressed(KeyCode::Escape)
        || keyboard.just_pressed(KeyCode::KeyQ)
        || mouse.just_pressed(MouseButton::Right);

    if cast.minigame_active() && cancel_pressed {
        if cancel_cast(&mut cast.state) {
            cast.holding = false;
            info!("Cast cancelled.");
        }
        return;
    }

    // During fight / cast / bite: no new cast starts.
    if cast.minigame_active() {
        return;
    }

    let use_pressed =
        keyboard.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left);
    if !use_pressed {
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
        info!("Move to a fishing dock to cast (try the lake path east).");
        return;
    }

    let cost = rod_energy_cost();
    if !can_afford_cast(energy.current, cost) || !energy.try_spend(cost) {
        info!("Not enough energy to fish.");
        return;
    }

    if start_cast(&mut cast.state) {
        cast.holding = false;
        info!("Casting… hold Space when the fish bites to keep it in the green bar.");
    }
}

fn apply_outcome(outcome: FishOutcome, inventory: &mut Inventory, dirty: &mut ProfileDirty) {
    if let Some((fish, amount)) = catch_yield(outcome) {
        let left = inventory.try_add(fish, amount);
        if left > 0 {
            warn!("Inventory full — lost fish.");
        } else {
            dirty.mark();
            info!("Caught {amount}× {}!", fish.display_name());
        }
    } else {
        info!("{}", outcome.feedback_label());
    }
}

pub fn clear_cast_on_zone_exit(mut cast: ResMut<ActiveCast>) {
    cast.clear();
}
