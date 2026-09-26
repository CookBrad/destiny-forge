mod loadout;
mod progress;

// Tests in other modules need this. A non-test `pub use` is `unused_imports`
// under `-D warnings`; `-A dead_code` does not cover that lint.
#[cfg(test)]
pub use loadout::ArmorSlots;
pub use loadout::{weapon_kind_label, ArmorKind, Loadout};
pub use progress::WorldProgress;
