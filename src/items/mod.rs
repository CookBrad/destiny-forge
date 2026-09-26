mod inventory;
mod material;

#[cfg(test)]
pub use inventory::MAX_STACK;
pub use inventory::{Inventory, INVENTORY_SLOT_COUNT};
pub use material::MaterialId;
