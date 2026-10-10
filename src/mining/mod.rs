//! One-layer ore mine entered from the homestead. Copper + iron nodes, pickaxe tiers,
//! respawn on sleep. Raw ore feeds the forge directly (no smelting).

mod interaction;
mod layout;
mod ore;
mod plugin;
mod setup;
#[cfg(test)]
mod transition_tests;

pub use ore::respawn_ore_nodes_on_sleep;
pub use plugin::MiningPlugin;
