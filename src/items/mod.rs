mod food;
mod inventory;
mod material;

pub use food::FoodBuff;
pub use inventory::{Inventory, INVENTORY_SLOT_COUNT};
pub use material::MaterialId;

// Tests in other modules need these. A non-test `pub use` is `unused_imports`
// under `-D warnings`; `-A dead_code` does not cover that lint.
#[cfg(test)]
pub use inventory::MAX_STACK;
#[cfg(test)]
pub use material::ItemCategory;
