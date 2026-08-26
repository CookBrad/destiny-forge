//! Pure farm actions applied to a plot stage + inventory.

use crate::items::Inventory;

use super::crops::{
    harvest_plot, plant_plot, till_plot, water_plot, CropKind, FarmActionResult, PlotStage,
};
use super::tools::{first_available_seed_crop, HomesteadTool};

pub fn apply_tool(
    tool: HomesteadTool,
    stage: PlotStage,
    inventory: &mut Inventory,
) -> Result<(PlotStage, FarmActionResult), &'static str> {
    match tool {
        HomesteadTool::Hoe => ok_or_fail(till_plot(stage)),
        HomesteadTool::WateringCan => ok_or_fail(water_plot(stage)),
        HomesteadTool::Seeds => plant_first_seed(stage, inventory),
        HomesteadTool::Hand => harvest_with_hand(stage, inventory),
    }
}

fn ok_or_fail(
    (next, result): (PlotStage, FarmActionResult),
) -> Result<(PlotStage, FarmActionResult), &'static str> {
    match result {
        FarmActionResult::Failed(msg) => Err(msg),
        other => Ok((next, other)),
    }
}

fn plant_first_seed(
    stage: PlotStage,
    inventory: &mut Inventory,
) -> Result<(PlotStage, FarmActionResult), &'static str> {
    let crop = first_available_seed_crop(|id| inventory.count(id) > 0)
        .ok_or("no seeds in inventory")?;
    plant_with_crop(stage, crop, inventory)
}

fn plant_with_crop(
    stage: PlotStage,
    crop: CropKind,
    inventory: &mut Inventory,
) -> Result<(PlotStage, FarmActionResult), &'static str> {
    if !inventory.try_remove(crop.seed_material(), 1) {
        return Err("no seeds in inventory");
    }
    match plant_plot(stage, crop) {
        (next, FarmActionResult::Failed(msg)) => {
            inventory.try_add(crop.seed_material(), 1);
            Err(msg)
        }
        (next, other) => Ok((next, other)),
    }
}

fn harvest_with_hand(
    stage: PlotStage,
    inventory: &mut Inventory,
) -> Result<(PlotStage, FarmActionResult), &'static str> {
    match harvest_plot(stage) {
        (next, FarmActionResult::Harvested { crop, amount }) => {
            let leftover = inventory.try_add(crop.harvest_material(), amount);
            if leftover > 0 {
                bevy::log::warn!("Inventory full — lost harvest remainder");
            }
            Ok((
                next,
                FarmActionResult::Harvested { crop, amount },
            ))
        }
        (_, FarmActionResult::Failed(msg)) => Err(msg),
        (next, other) => Ok((next, other)),
    }
}

pub fn log_action(action: &FarmActionResult) {
    match action {
        FarmActionResult::Tilled => bevy::log::info!("Tilled soil."),
        FarmActionResult::Planted(crop) => {
            bevy::log::info!("Planted {} seeds.", crop.label())
        }
        FarmActionResult::Watered => bevy::log::info!("Watered crop."),
        FarmActionResult::Harvested { crop, amount } => {
            bevy::log::info!("Harvested {amount}× {}.", crop.label());
        }
        FarmActionResult::Failed(msg) => bevy::log::info!("{msg}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::farming::crops::PlotStage;
    use crate::items::{Inventory, MaterialId};

    #[test]
    fn plant_requires_tilled_soil() {
        let mut inv = Inventory::with_starter_seeds();
        let err = apply_tool(HomesteadTool::Seeds, PlotStage::Soil, &mut inv).unwrap_err();
        assert!(err.contains("till"));
        assert_eq!(inv.count(MaterialId::TurnipSeed), 8);
    }

    #[test]
    fn plant_uses_turnip_then_potato() {
        let mut inv = Inventory::with_starter_seeds();
        let (stage, result) =
            apply_tool(HomesteadTool::Seeds, PlotStage::Tilled, &mut inv).unwrap();
        assert!(matches!(
            stage,
            PlotStage::Growing {
                crop: CropKind::Turnip,
                days: 0,
                ..
            }
        ));
        assert!(matches!(result, FarmActionResult::Planted(CropKind::Turnip)));
        assert_eq!(inv.count(MaterialId::TurnipSeed), 7);
        assert_eq!(inv.count(MaterialId::PotatoSeed), 4);
    }

    #[test]
    fn hoe_tills_soil() {
        let mut inv = Inventory::with_starter_seeds();
        let (stage, result) = apply_tool(HomesteadTool::Hoe, PlotStage::Soil, &mut inv).unwrap();
        assert_eq!(stage, PlotStage::Tilled);
        assert_eq!(result, FarmActionResult::Tilled);
        assert_eq!(inv.count(MaterialId::Hoe), 1);
        assert_eq!(inv.count(MaterialId::WateringCan), 1);
        assert_eq!(inv.count(MaterialId::TurnipSeed), 8);
        assert_eq!(inv.count(MaterialId::PotatoSeed), 4);
    }
}
