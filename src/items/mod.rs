mod inventory;
mod material;
mod plugin;

pub use inventory::{Inventory, MaterialStack, INVENTORY_SLOT_COUNT, MAX_STACK};
pub use material::MaterialId;
pub use plugin::ItemsPlugin;