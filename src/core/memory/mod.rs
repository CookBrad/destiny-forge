mod autosave;
mod autosave_block;
mod crop_save;
mod migration;
mod plugin;
mod profile;
mod ron_value;
mod settings;
mod storage;
mod sync;

pub use autosave::ProfileDirty;
pub use autosave_block::{load_profile_for_slot, ProfileAutosaveBlock, ProfileNeedsAttention};
pub use crop_save::{SavedCropKind, SavedCropPlot, SavedPlotStage};
pub use plugin::MemoryPlugin;
pub use profile::{
    rename_profile_on_disk, sanitize_profile_name, ActiveProfile, PlayerProfile, PROFILE_COUNT,
};
pub use settings::GameSettings;
pub use storage::save_root_display;
pub use sync::{activate_profile, apply_profile_to_runtime, ProfileApply};
