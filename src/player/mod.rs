mod loadout;
mod progress;

// Tests in other modules need this. A non-test `pub use` is `unused_imports`
// under `-D warnings`; `-A dead_code` does not cover that lint.
#[cfg(test)]
pub use loadout::ArmorSlots;
pub use loadout::{expire_food_buff_after_hunt, weapon_kind_label, ArmorKind, Loadout};
pub use progress::WorldProgress;
