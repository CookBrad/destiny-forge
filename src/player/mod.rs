mod loadout;
mod progress;

#[cfg(test)]
pub use loadout::ArmorSlots;
pub use loadout::{weapon_kind_label, ArmorKind, Loadout};
pub use progress::WorldProgress;
