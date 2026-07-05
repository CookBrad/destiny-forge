use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::profile::{PlayerProfile, PROFILE_COUNT};
use super::settings::GameSettings;

const APP_QUALIFIER: &str = "";
const APP_ORG: &str = "";
const APP_NAME: &str = "destiny_forge";

pub fn save_root() -> PathBuf {
    directories::ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
        .expect("valid application directories")
        .data_local_dir()
        .to_path_buf()
}

pub fn settings_path() -> PathBuf {
    save_root().join("settings.ron")
}

pub fn profile_path(index: u8) -> PathBuf {
    save_root()
        .join("profiles")
        .join(format!("profile_{}.ron", index.min(PROFILE_COUNT - 1)))
}

pub fn load_settings() -> GameSettings {
    load_ron::<GameSettings>(settings_path())
        .unwrap_or_default()
        .migrate()
}

pub fn save_settings(settings: &GameSettings) -> io::Result<()> {
    write_ron(settings_path(), settings)
}

pub fn load_profile(index: u8) -> PlayerProfile {
    load_ron::<PlayerProfile>(profile_path(index))
        .unwrap_or_default()
        .migrate()
}

pub fn save_profile(index: u8, profile: &PlayerProfile) -> io::Result<()> {
    write_ron(profile_path(index), profile)
}

fn load_ron<T>(path: PathBuf) -> Option<T>
where
    T: serde::de::DeserializeOwned + Default,
{
    let contents = fs::read_to_string(&path).ok()?;
    let value: T = ron::from_str(&contents).ok()?;
    Some(value)
}

fn write_ron<T>(path: PathBuf, value: &T) -> io::Result<()>
where
    T: serde::Serialize,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let pretty = ron::ser::PrettyConfig::new().depth_limit(4);
    let contents = ron::ser::to_string_pretty(value, pretty)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let temp_path = path.with_extension("ron.tmp");
    fs::write(&temp_path, contents)?;
    fs::rename(temp_path, path)?;
    Ok(())
}

pub fn save_root_display() -> String {
    save_root().display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{Inventory, MaterialId};
    use crate::player::Loadout;

    #[test]
    fn profile_round_trip_in_temp_dir() {
        let dir = std::env::temp_dir().join("destiny_forge_test_saves");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = dir.join("profile_0.ron");
        let mut profile = PlayerProfile::default();
        profile.inventory.try_add(MaterialId::SlimeGel, 3);

        write_ron(path.clone(), &profile).unwrap();
        let loaded: PlayerProfile = load_ron(path).unwrap();
        assert_eq!(loaded.inventory.count(MaterialId::SlimeGel), 3);
        assert_eq!(loaded.loadout, Loadout::default());
        let _ = fs::remove_dir_all(&dir);
    }
}