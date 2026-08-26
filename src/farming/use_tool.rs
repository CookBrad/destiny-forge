//! Use the equipped homestead tool (keys 1–4) on a nearby crop plot.

use bevy::prelude::*;

use crate::core::{PlayerProfile, ProfileDirty, ToolEnergy};
use crate::graphics::TILE;
use crate::items::{Inventory, MaterialId};
use crate::overworld::movement::{OverworldPlayer, OverworldVelocity};
use crate::ui::forge_window::ForgeWindowOpen;
use crate::ui::inventory_window::InventoryWindowOpen;

use super::actions::{apply_tool, log_action};
use super::persist::capture_plots;
use super::plots::{facing_tile, tile_coords_from_world, CropPlot, PlayerFacing};
use super::tools::EquippedTool;

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

/// Ensure hoe, can, and starter seeds exist for older saves. Never grants pickaxe/rod.
pub fn ensure_starter_seeds(mut inventory: ResMut<Inventory>, mut dirty: ResMut<ProfileDirty>) {
    let mut changed = false;
    changed |= grant_if_missing(&mut inventory, MaterialId::Hoe, 1);
    changed |= grant_if_missing(&mut inventory, MaterialId::WateringCan, 1);
    let has_seed = inventory.count(MaterialId::TurnipSeed) > 0
        || inventory.count(MaterialId::PotatoSeed) > 0;
    if !has_seed {
        inventory.try_add(MaterialId::TurnipSeed, 8);
        inventory.try_add(MaterialId::PotatoSeed, 4);
        changed = true;
    }
    if changed {
        dirty.mark();
        info!("Restocked homestead hoe, watering can, and starter seeds.");
    }
}

fn grant_if_missing(inventory: &mut Inventory, material: MaterialId, amount: u32) -> bool {
    if inventory.count(material) == 0 {
        inventory.try_add(material, amount);
        true
    } else {
        false
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
    mut profile: ResMut<PlayerProfile>,
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
    let player_pos = transform.translation.truncate();
    let Some((tile_x, tile_y)) = find_target_tiles(player_pos, facing.dir, &plots) else {
        info!(
            "No crop plot in range. Equipped: {}.",
            equipped.0.label()
        );
        return;
    };

    let tool = equipped.0;
    let cost = tool.energy_cost();
    if cost > 0.0 && !energy.try_spend(cost) {
        info!("Not enough energy for {}.", tool.label());
        return;
    }

    let outcome = apply_to_target(&mut plots, tile_x, tile_y, tool, &mut inventory);
    match outcome {
        Ok(()) => {
            persist_plots(&plots, &mut profile, &mut dirty);
        }
        Err(message) => {
            if cost > 0.0 {
                energy.current = (energy.current + cost).min(energy.max);
            }
            info!("{message}");
        }
    }
}

fn apply_to_target(
    plots: &mut Query<&mut CropPlot>,
    tile_x: u32,
    tile_y: u32,
    tool: super::tools::HomesteadTool,
    inventory: &mut Inventory,
) -> Result<(), &'static str> {
    let Some(mut plot) = plots
        .iter_mut()
        .find(|plot| plot.tile_x == tile_x && plot.tile_y == tile_y)
    else {
        return Err("plot gone");
    };

    let (stage, action) = apply_tool(tool, plot.stage, inventory)?;
    plot.stage = stage;
    log_action(&action);
    Ok(())
}

pub fn persist_plots(
    plots: &Query<&mut CropPlot>,
    profile: &mut PlayerProfile,
    dirty: &mut ProfileDirty,
) {
    profile.crop_plots = capture_plots(plots.iter());
    dirty.mark();
}

pub fn capture_crop_plots_on_exit(
    plots: Query<&CropPlot>,
    mut profile: ResMut<PlayerProfile>,
    mut dirty: ResMut<ProfileDirty>,
) {
    profile.crop_plots = capture_plots(plots.iter());
    dirty.mark();
}

fn find_target_tiles(
    player_pos: Vec2,
    facing: Vec2,
    plots: &Query<&mut CropPlot>,
) -> Option<(u32, u32)> {
    let (face_tx, face_ty) = facing_tile(player_pos, facing);
    let (stand_tx, stand_ty) = tile_coords_from_world(player_pos);

    let mut best_facing = None;
    let mut best_standing = None;
    let mut best_near: Option<(u32, u32, f32)> = None;

    for plot in plots.iter() {
        if plot.tile_x == face_tx && plot.tile_y == face_ty {
            best_facing = Some((plot.tile_x, plot.tile_y));
        }
        if plot.tile_x == stand_tx && plot.tile_y == stand_ty {
            best_standing = Some((plot.tile_x, plot.tile_y));
        }
        let center = plot_center(plot.tile_x, plot.tile_y);
        let dist = player_pos.distance(center);
        if dist <= PLOT_TARGET_RANGE {
            let better = best_near.map(|(_, _, best_d)| dist < best_d).unwrap_or(true);
            if better {
                best_near = Some((plot.tile_x, plot.tile_y, dist));
            }
        }
    }

    best_facing
        .or(best_standing)
        .or(best_near.map(|(x, y, _)| (x, y)))
}

fn plot_center(tile_x: u32, tile_y: u32) -> Vec2 {
    Vec2::new(
        tile_x as f32 * TILE + TILE * 0.5,
        tile_y as f32 * TILE + TILE * 0.5,
    )
}
