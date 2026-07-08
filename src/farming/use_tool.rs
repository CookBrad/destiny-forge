//! Use equipped homestead tool on the facing crop plot.

use bevy::prelude::*;

use crate::core::{ToolEnergy, ProfileDirty};
use crate::items::Inventory;
use crate::overworld::movement::{OverworldPlayer, OverworldVelocity};
use crate::ui::forge_window::ForgeWindowOpen;
use crate::ui::inventory_window::InventoryWindowOpen;

use super::crops::{
    harvest_plot, plant_plot, till_plot, water_plot, CropKind, FarmActionResult, PlotStage,
};
use super::plots::{facing_tile, CropPlot, PlayerFacing};
use super::tools::{first_available_seed_crop, EquippedTool, HomesteadTool};

pub fn update_player_facing(
    mut player: Query<(&OverworldVelocity, &mut PlayerFacing), With<OverworldPlayer>>,
) {
    let Ok((velocity, mut facing)) = player.get_single_mut() else {
        return;
    };
    let v = Vec2::new(velocity.x, velocity.y);
    if v.length_squared() > 1.0 {
        facing.dir = v.normalize();
    }
}

pub fn select_homestead_tool(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory_open: Res<InventoryWindowOpen>,
    forge_open: Res<ForgeWindowOpen>,
    mut equipped: ResMut<EquippedTool>,
) {
    if inventory_open.0 || forge_open.0 {
        return;
    }

    if keyboard.just_pressed(KeyCode::Digit1) {
        equipped.0 = HomesteadTool::Hoe;
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        equipped.0 = HomesteadTool::WateringCan;
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        equipped.0 = HomesteadTool::Seeds;
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        equipped.0 = HomesteadTool::Hand;
    }
}

pub fn use_homestead_tool(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    inventory_open: Res<InventoryWindowOpen>,
    forge_open: Res<ForgeWindowOpen>,
    equipped: Res<EquippedTool>,
    mut energy: ResMut<ToolEnergy>,
    mut inventory: ResMut<Inventory>,
    mut dirty: ResMut<ProfileDirty>,
    player: Query<(&Transform, &PlayerFacing), With<OverworldPlayer>>,
    mut plots: Query<&mut CropPlot>,
) {
    if inventory_open.0 || forge_open.0 {
        return;
    }

    let use_pressed =
        keyboard.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left);
    if !use_pressed {
        return;
    }

    let Ok((transform, facing)) = player.get_single() else {
        return;
    };

    let (tx, ty) = facing_tile(transform.translation.truncate(), facing.dir);
    let Some(mut plot) = plots
        .iter_mut()
        .find(|p| p.tile_x == tx && p.tile_y == ty)
    else {
        return;
    };

    let cost = equipped.0.energy_cost();
    if cost > 0.0 && !energy.try_spend(cost) {
        info!("Not enough energy for {}.", equipped.0.label());
        return;
    }

    let result = apply_tool(equipped.0, plot.stage, &mut inventory);
    match result {
        Ok((stage, action)) => {
            plot.stage = stage;
            dirty.mark();
            log_action(action);
        }
        Err(message) => {
            // Refund energy on failed meaningful actions.
            if cost > 0.0 {
                energy.current = (energy.current + cost).min(energy.max);
            }
            info!("{message}");
        }
    }
}

fn apply_tool(
    tool: HomesteadTool,
    stage: PlotStage,
    inventory: &mut Inventory,
) -> Result<(PlotStage, FarmActionResult), &'static str> {
    match tool {
        HomesteadTool::Hoe => {
            let (next, result) = till_plot(stage);
            match result {
                FarmActionResult::Failed(msg) => Err(msg),
                other => Ok((next, other)),
            }
        }
        HomesteadTool::WateringCan => {
            let (next, result) = water_plot(stage);
            match result {
                FarmActionResult::Failed(msg) => Err(msg),
                other => Ok((next, other)),
            }
        }
        HomesteadTool::Seeds => {
            let crop = first_available_seed_crop(|m| inventory.count(m) > 0)
                .ok_or("no seeds in inventory")?;
            if !inventory.try_remove(crop.seed_material(), 1) {
                return Err("no seeds in inventory");
            }
            let (next, result) = plant_plot(stage, crop);
            if matches!(result, FarmActionResult::Failed(_)) {
                // refund seed
                inventory.try_add(crop.seed_material(), 1);
                return Err("till before planting");
            }
            Ok((next, result))
        }
        HomesteadTool::Hand => {
            let (next, result) = harvest_plot(stage);
            match result {
                FarmActionResult::Harvested { crop, amount } => {
                    let leftover = inventory.try_add(crop.harvest_material(), amount);
                    if leftover > 0 {
                        warn!("Inventory full — lost harvest remainder");
                    }
                    Ok((next, result))
                }
                FarmActionResult::Failed(msg) => Err(msg),
                other => Ok((next, other)),
            }
        }
    }
}

fn log_action(action: FarmActionResult) {
    match action {
        FarmActionResult::Tilled => info!("Tilled soil."),
        FarmActionResult::Planted(crop) => info!("Planted {}.", crop.label()),
        FarmActionResult::Watered => info!("Watered crop."),
        FarmActionResult::Harvested { crop, amount } => {
            info!("Harvested {amount}× {}.", crop.label());
        }
        FarmActionResult::Failed(msg) => info!("{msg}"),
    }
}

/// Prefer crop under seed tool for planting preference when cycling seeds later.
#[allow(dead_code)]
fn seed_crop_order() -> [CropKind; 2] {
    [CropKind::Turnip, CropKind::Potato]
}
