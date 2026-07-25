//! Use selected hotbar inventory item on a nearby crop plot.

use bevy::prelude::*;

use crate::core::{ProfileDirty, ToolEnergy};
use crate::graphics::TILE;
use crate::items::{Inventory, MaterialId};
use crate::overworld::movement::{OverworldPlayer, OverworldVelocity};
use crate::ui::forge_window::ForgeWindowOpen;
use crate::ui::inventory_window::InventoryWindowOpen;

use super::crops::{
    harvest_plot, plant_plot, till_plot, water_plot, CropKind, FarmActionResult, PlotStage,
};
use super::hotbar::{HomesteadHotbar, HotbarEntry};
use super::plots::{facing_tile, tile_coords_from_world, CropPlot, PlayerFacing};

const PLOT_TARGET_RANGE: f32 = TILE * 1.35;

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

/// Ensure tools + seeds exist for older saves.
pub fn ensure_starter_seeds(mut inventory: ResMut<Inventory>, mut dirty: ResMut<ProfileDirty>) {
    let mut changed = false;
    if inventory.count(MaterialId::Hoe) == 0 {
        inventory.try_add(MaterialId::Hoe, 1);
        changed = true;
    }
    if inventory.count(MaterialId::WateringCan) == 0 {
        inventory.try_add(MaterialId::WateringCan, 1);
        changed = true;
    }
    if inventory.count(MaterialId::Pickaxe) == 0 {
        inventory.try_add(MaterialId::Pickaxe, 1);
        changed = true;
    }
    if inventory.count(MaterialId::FishingRod) == 0 {
        inventory.try_add(MaterialId::FishingRod, 1);
        changed = true;
    }
    let has_seed = inventory.count(MaterialId::TurnipSeed) > 0
        || inventory.count(MaterialId::PotatoSeed) > 0;
    if !has_seed {
        inventory.try_add(MaterialId::TurnipSeed, 8);
        inventory.try_add(MaterialId::PotatoSeed, 4);
        changed = true;
    }
    if changed {
        dirty.mark();
        info!("Restocked homestead tools/seeds in inventory — drag them onto the hotbar.");
    }
}

pub fn use_homestead_tool(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    inventory_open: Res<InventoryWindowOpen>,
    forge_open: Res<ForgeWindowOpen>,
    hotbar: Res<HomesteadHotbar>,
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

    let entry = hotbar.selected_entry();
    let Some(material) = entry.material() else {
        // Empty highlighted slot: harvest only.
        try_harvest_only(
            &player,
            &mut plots,
            &mut inventory,
            &mut dirty,
        );
        return;
    };

    // Pickaxe / rod / food are handled by mining, fishing, cooking systems.
    if matches!(
        material,
        MaterialId::Pickaxe | MaterialId::FishingRod
    ) || material.is_food()
    {
        return;
    }

    if inventory.count(material) == 0 && !material.is_tool() {
        info!("No {} left — restock from inventory.", material.display_name());
        return;
    }
    if material.is_tool() && inventory.count(material) == 0 {
        info!("{} not in inventory.", material.display_name());
        return;
    }

    let Ok((transform, facing)) = player.get_single() else {
        return;
    };

    let player_pos = transform.translation.truncate();
    let Some(plot_index) = find_target_plot_index(player_pos, facing.dir, &plots) else {
        info!(
            "No crop plot in range. Selected: {}.",
            material.display_name()
        );
        return;
    };

    let target = {
        let plot = plots.iter().nth(plot_index).expect("plot index valid");
        (plot.tile_x, plot.tile_y)
    };

    let Some(mut plot) = plots
        .iter_mut()
        .find(|p| p.tile_x == target.0 && p.tile_y == target.1)
    else {
        return;
    };

    // Ready crops: harvest with empty slot only (see try_harvest_only). With an item
    // selected, try normal use first; seeds fail on ready unless we harvest.
    if matches!(plot.stage, PlotStage::Ready { .. }) {
        if let Ok((stage, action)) = harvest_with_hand(plot.stage, &mut inventory) {
            plot.stage = stage;
            dirty.mark();
            log_action(action);
            return;
        }
    }

    let cost = material.energy_cost();
    if cost > 0.0 && !energy.try_spend(cost) {
        info!("Not enough energy for {}.", material.display_name());
        return;
    }

    let result = apply_material(material, plot.stage, &mut inventory);
    match result {
        Ok((stage, action)) => {
            plot.stage = stage;
            dirty.mark();
            log_action(action);
        }
        Err(message) => {
            if cost > 0.0 {
                energy.current = (energy.current + cost).min(energy.max);
            }
            info!("{message}");
        }
    }
}

fn try_harvest_only(
    player: &Query<(&Transform, &PlayerFacing), With<OverworldPlayer>>,
    plots: &mut Query<&mut CropPlot>,
    inventory: &mut Inventory,
    dirty: &mut ProfileDirty,
) {
    let Ok((transform, facing)) = player.get_single() else {
        info!("Hotbar slot empty — drag a tool or seed from inventory (I). Empty slot harvests ready crops.");
        return;
    };
    let player_pos = transform.translation.truncate();
    let Some(plot_index) = find_target_plot_index(player_pos, facing.dir, plots) else {
        info!("Hotbar empty — drag items from inventory onto the bar. Empty slot can harvest.");
        return;
    };
    let target = {
        let plot = plots.iter().nth(plot_index).expect("plot");
        (plot.tile_x, plot.tile_y)
    };
    let Some(mut plot) = plots
        .iter_mut()
        .find(|p| p.tile_x == target.0 && p.tile_y == target.1)
    else {
        return;
    };
    match harvest_with_hand(plot.stage, inventory) {
        Ok((stage, action)) => {
            plot.stage = stage;
            dirty.mark();
            log_action(action);
        }
        Err(msg) => info!("{msg} (empty hotbar harvests ready crops only)"),
    }
}

fn find_target_plot_index(
    player_pos: Vec2,
    facing: Vec2,
    plots: &Query<&mut CropPlot>,
) -> Option<usize> {
    let (face_tx, face_ty) = facing_tile(player_pos, facing);
    let (stand_tx, stand_ty) = tile_coords_from_world(player_pos);

    let mut best_facing: Option<usize> = None;
    let mut best_standing: Option<usize> = None;
    let mut best_near: Option<(usize, f32)> = None;

    for (index, plot) in plots.iter().enumerate() {
        if plot.tile_x == face_tx && plot.tile_y == face_ty {
            best_facing = Some(index);
        }
        if plot.tile_x == stand_tx && plot.tile_y == stand_ty {
            best_standing = Some(index);
        }

        let center = plot_center(plot.tile_x, plot.tile_y);
        let dist = player_pos.distance(center);
        if dist <= PLOT_TARGET_RANGE {
            match best_near {
                Some((_, best_d)) if dist >= best_d => {}
                _ => best_near = Some((index, dist)),
            }
        }
    }

    best_facing.or(best_standing).or(best_near.map(|(i, _)| i))
}

fn plot_center(tile_x: u32, tile_y: u32) -> Vec2 {
    Vec2::new(
        tile_x as f32 * TILE + TILE * 0.5,
        tile_y as f32 * TILE + TILE * 0.5,
    )
}

fn apply_material(
    material: MaterialId,
    stage: PlotStage,
    inventory: &mut Inventory,
) -> Result<(PlotStage, FarmActionResult), &'static str> {
    match material {
        MaterialId::Hoe => {
            let (next, result) = till_plot(stage);
            match result {
                FarmActionResult::Failed(msg) => Err(msg),
                other => Ok((next, other)),
            }
        }
        MaterialId::WateringCan => {
            let (next, result) = water_plot(stage);
            match result {
                FarmActionResult::Failed(msg) => Err(msg),
                other => Ok((next, other)),
            }
        }
        m if m.is_seed() => {
            let crop = CropKind::from_seed(m).ok_or("not a plantable seed")?;
            plant_with_crop(stage, crop, inventory)
        }
        _ => Err("that item is not usable on plots"),
    }
}

fn plant_with_crop(
    stage: PlotStage,
    crop: CropKind,
    inventory: &mut Inventory,
) -> Result<(PlotStage, FarmActionResult), &'static str> {
    if !inventory.try_remove(crop.seed_material(), 1) {
        return Err("no seeds in inventory");
    }
    let (next, result) = plant_plot(stage, crop);
    match result {
        FarmActionResult::Failed(msg) => {
            inventory.try_add(crop.seed_material(), 1);
            Err(msg)
        }
        other => Ok((next, other)),
    }
}

fn harvest_with_hand(
    stage: PlotStage,
    inventory: &mut Inventory,
) -> Result<(PlotStage, FarmActionResult), &'static str> {
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

fn log_action(action: FarmActionResult) {
    match action {
        FarmActionResult::Tilled => info!("Tilled soil."),
        FarmActionResult::Planted(crop) => info!("Planted {} seeds.", crop.label()),
        FarmActionResult::Watered => info!("Watered crop."),
        FarmActionResult::Harvested { crop, amount } => {
            info!("Harvested {amount}× {}.", crop.label());
        }
        FarmActionResult::Failed(msg) => info!("{msg}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plant_requires_tilled_soil() {
        let mut inv = Inventory::with_starter_seeds();
        let err = apply_material(MaterialId::TurnipSeed, PlotStage::Soil, &mut inv).unwrap_err();
        assert!(err.contains("till"));
        assert_eq!(inv.count(MaterialId::TurnipSeed), 8);
    }

    #[test]
    fn plant_specific_seed() {
        let mut inv = Inventory::with_starter_seeds();
        let (stage, result) =
            apply_material(MaterialId::PotatoSeed, PlotStage::Tilled, &mut inv).unwrap();
        assert!(matches!(
            stage,
            PlotStage::Growing {
                crop: CropKind::Potato,
                days: 0,
                ..
            }
        ));
        assert!(matches!(result, FarmActionResult::Planted(CropKind::Potato)));
        assert_eq!(inv.count(MaterialId::PotatoSeed), 3);
    }

    #[test]
    fn hoe_tills_soil() {
        let mut inv = Inventory::with_starter_seeds();
        let (stage, result) = apply_material(MaterialId::Hoe, PlotStage::Soil, &mut inv).unwrap();
        assert_eq!(stage, PlotStage::Tilled);
        assert_eq!(result, FarmActionResult::Tilled);
        assert_eq!(inv.count(MaterialId::Hoe), 1);
    }
}
