mod food;
mod inventory;
mod material;
mod quality;

// Tests in other modules need this. A non-test `pub use` is `unused_imports`
// under `-D warnings`; `-A dead_code` does not cover that lint.
#[cfg(test)]
pub use inventory::MAX_STACK;
#[cfg(test)]
pub use quality::ItemQuality;
pub use food::FoodBuff;
pub use inventory::{Inventory, INVENTORY_SLOT_COUNT};
pub use material::MaterialId;
