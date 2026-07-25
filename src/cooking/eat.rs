//! Eat cooked food from the homestead hotbar.

use bevy::prelude::*;

use crate::core::ProfileDirty;
use crate::farming::HomesteadHotbar;
use crate::items::{Inventory, MaterialId};
use crate::overworld::movement::OverworldPlayer;
use crate::ui::forge_window::ForgeWindowOpen;
use crate::ui::inventory_window::InventoryWindowOpen;

use super::buffs::{try_eat_food, ActiveFoodBuff};

pub fn eat_food_from_hotbar(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    inventory_open: Res<InventoryWindowOpen>,
    forge_open: Res<ForgeWindowOpen>,
    hotbar: Res<HomesteadHotbar>,
    mut inventory: ResMut<Inventory>,
    mut buff: ResMut<ActiveFoodBuff>,
    mut dirty: ResMut<ProfileDirty>,
    // Ensure we only run when player exists (overworld).
    player: Query<(), With<OverworldPlayer>>,
) {
    if inventory_open.0 || forge_open.0 || player.get_single().is_err() {
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
    if !material.is_food() {
        return;
    }

    match try_eat_food(&mut inventory, &mut buff, material) {
        Ok(name) => {
            dirty.mark();
            info!(
                "Ate {name}. Buff active until {:?}.",
                buff.expiry
            );
        }
        Err(msg) => info!("{msg}"),
    }
}

/// Clear food buffs that expire on sleep.
pub fn clear_food_buff_on_sleep(buff: &mut ActiveFoodBuff) {
    buff.on_sleep();
}

/// Clear OneHunt buffs when leaving the dungeon.
pub fn clear_food_buff_on_hunt_end(mut buff: ResMut<ActiveFoodBuff>) {
    if buff.expiry == super::buffs::BuffExpiry::OneHunt {
        info!("Hunt food buff expired.");
        buff.on_hunt_end();
    }
}

// Silence unused MaterialId if only used via is_food
#[allow(dead_code)]
fn _food_ids() -> [MaterialId; 2] {
    [MaterialId::HeartyStew, MaterialId::SpicySashimi]
}
