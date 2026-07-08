//! Use selected homestead hotbar entry on a nearby crop plot.

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
use super::tools::HomesteadTool;

/// Max distance (world pixels) to auto-target a plot when facing misses.
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

/// Ensure older saves still have seeds so planting is possible.
pub fn ensure_starter_seeds(mut inventory: ResMut<Inventory>, mut dirty: ResMut<ProfileDirty>) {
    let has_any_seed = inventory.count(MaterialId::TurnipSeed) > 0
        || inventory.count(MaterialId::PotatoSeed) > 0;
    if has_any_seed {
        return;
    }
    inventory.try_add(MaterialId::TurnipSeed, 8);
    inventory.try_add(MaterialId::PotatoSeed, 4);
    dirty.mark();
    info!("Granted starter seeds (turnip ×8, potato ×4).");
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
    if matches!(entry, HotbarEntry::Empty) {
        info!("Hotbar slot empty — pick a tool or seed (1–5).");
        return;
    }

    let Ok((transform, facing)) = player.get_single() else {
        return;
    };

    let player_pos = transform.translation.truncate();
    let Some(plot_index) = find_target_plot_index(player_pos, facing.dir, &plots) else {
        info!(
            "No crop plot in range (stand on the field). Selected: {}.",
            entry.label()
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

    let cost = entry.energy_cost();
    if cost > 0.0 && !energy.try_spend(cost) {
        info!("Not enough energy for {}.", entry.label());
        return;
    }

    let result = apply_hotbar_entry(entry, plot.stage, &mut inventory);
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

fn apply_hotbar_entry(
    entry: HotbarEntry,
    stage: PlotStage,
    inventory: &mut Inventory,
) -> Result<(PlotStage, FarmActionResult), &'static str> {
    match entry {
        HotbarEntry::Empty => Err("empty hotbar slot"),
        HotbarEntry::Tool(HomesteadTool::Hoe) => {
            let (next, result) = till_plot(stage);
            match result {
                FarmActionResult::Failed(msg) => Err(msg),
                other => Ok((next, other)),
            }
        }
        HotbarEntry::Tool(HomesteadTool::WateringCan) => {
            let (next, result) = water_plot(stage);
            match result {
                FarmActionResult::Failed(msg) => Err(msg),
                other => Ok((next, other)),
            }
        }
        HotbarEntry::Tool(HomesteadTool::Seeds) => {
            let crop = super::tools::first_available_seed_crop(|m| inventory.count(m) > 0)
                .ok_or("no seeds in inventory")?;
            plant_with_crop(stage, crop, inventory)
        }
        HotbarEntry::Tool(HomesteadTool::Hand) => harvest_with_hand(stage, inventory),
        HotbarEntry::Item(material) if material.is_seed() => {
            let crop = CropKind::from_seed(material).ok_or("not a plantable seed")?;
            if inventory.count(material) == 0 {
                return Err("out of that seed");
            }
            plant_with_crop(stage, crop, inventory)
        }
        HotbarEntry::Item(_) => Err("that item is not usable on plots"),
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
        let err = apply_hotbar_entry(
            HotbarEntry::Item(MaterialId::TurnipSeed),
            PlotStage::Soil,
            &mut inv,
        )
        .unwrap_err();
        assert!(err.contains("till"));
        assert_eq!(inv.count(MaterialId::TurnipSeed), 8);
    }

    #[test]
    fn plant_specific_seed_from_hotbar() {
        let mut inv = Inventory::with_starter_seeds();
        let (stage, result) = apply_hotbar_entry(
            HotbarEntry::Item(MaterialId::PotatoSeed),
            PlotStage::Tilled,
            &mut inv,
        )
        .unwrap();
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
}
