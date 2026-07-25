//! Mine ore nodes with the pickaxe on the hotbar.

use bevy::prelude::*;

use crate::core::{ProfileDirty, ToolEnergy};
use crate::farming::HomesteadHotbar;
use crate::graphics::TILE;
use crate::items::{Inventory, MaterialId};
use crate::overworld::movement::OverworldPlayer;
use crate::ui::forge_window::ForgeWindowOpen;
use crate::ui::inventory_window::InventoryWindowOpen;

use super::logic::{try_mine_node, MineResult};
use super::nodes::OreNode;

const NODE_RANGE: f32 = TILE * 1.5;

/// Bevy system: Space/LMB with pickaxe selected mines nearby nodes.
pub fn use_pickaxe_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    inventory_open: Res<InventoryWindowOpen>,
    forge_open: Res<ForgeWindowOpen>,
    hotbar: Res<HomesteadHotbar>,
    mut energy: ResMut<ToolEnergy>,
    mut inventory: ResMut<Inventory>,
    mut dirty: ResMut<ProfileDirty>,
    player: Query<&Transform, With<OverworldPlayer>>,
    mut nodes: Query<(&mut OreNode, &mut Sprite, &Transform), Without<OverworldPlayer>>,
) {
    if inventory_open.0 || forge_open.0 {
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
    if material != MaterialId::Pickaxe {
        return;
    }
    let Ok(transform) = player.get_single() else {
        return;
    };
    if inventory.count(MaterialId::Pickaxe) == 0 {
        info!("Pickaxe not in inventory.");
        return;
    }

    let player_pos = transform.translation.truncate();
    let power = MaterialId::Pickaxe.pickaxe_power();
    let cost = MaterialId::Pickaxe.energy_cost();

    // Closest intact node within range (by distance).
    let mut best_dist = f32::MAX;
    for (node, _, node_tf) in nodes.iter() {
        if !node.intact {
            continue;
        }
        let dist = player_pos.distance(node_tf.translation.truncate());
        if dist <= NODE_RANGE && dist < best_dist {
            best_dist = dist;
        }
    }
    if best_dist == f32::MAX {
        info!("No ore node in range. Face the rocks by the mine entrance.");
        return;
    }

    for (mut node, mut sprite, node_tf) in &mut nodes {
        if !node.intact {
            continue;
        }
        let dist = player_pos.distance(node_tf.translation.truncate());
        if (dist - best_dist).abs() > 0.01 || dist > NODE_RANGE {
            continue;
        }

        if cost > 0.0 && !energy.try_spend(cost) {
            info!("Not enough energy to mine.");
            return;
        }

        match try_mine_node(power, node.hardness) {
            MineResult::Broke { drops } => {
                node.intact = false;
                sprite.color = Color::srgb(0.22, 0.22, 0.24);
                for (mat, amount) in drops {
                    let left = inventory.try_add(mat, amount);
                    if left > 0 {
                        warn!("Inventory full — lost {left}× {}", mat.display_name());
                    } else {
                        info!("Mined {amount}× {}.", mat.display_name());
                    }
                }
                dirty.mark();
            }
            MineResult::TooHard { required, power } => {
                energy.current = (energy.current + cost).min(energy.max);
                info!("Node too hard (need power {required}, have {power}).");
            }
            MineResult::NoPickaxe => {
                energy.current = (energy.current + cost).min(energy.max);
                info!("Need a pickaxe to mine.");
            }
        }
        return;
    }

    info!("No ore node in range. Face the rocks by the mine entrance.");
}
