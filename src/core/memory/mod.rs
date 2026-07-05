mod autosave;
mod plugin;
mod profile;
mod settings;
mod storage;
mod sync;

pub use autosave::ProfileDirty;
pub use plugin::MemoryPlugin;
pub use profile::{ActiveProfile, PlayerProfile, PROFILE_COUNT};
pub use settings::{GameSettings, ProfileSettings};
pub use storage::{load_profile, save_root, save_root_display, settings_path};
pub use sync::{activate_profile, apply_profile_to_runtime, snapshot_profile};