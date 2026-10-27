use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::profile::{PlayerProfile, PROFILE_COUNT};
use super::settings::GameSettings;

const APP_QUALIFIER: &str = "";
const APP_ORG: &str = "";
const APP_NAME: &str = "destiny_forge";

#[derive(Debug)]
pub enum ProfileLoadError {
    Missing,
    Unreadable(io::Error),
    Corrupt(String),
}

impl ProfileLoadError {
    pub(crate) fn detail(&self) -> String {
        match self {
            Self::Missing => "profile file is missing".to_string(),
            Self::Unreadable(error) => error.to_string(),
            Self::Corrupt(detail) => detail.clone(),
        }
    }
}

#[cfg(test)]
static SAVE_ROOT_GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
static SAVE_ROOT_OVERRIDE: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

pub fn save_root() -> PathBuf {
    #[cfg(test)]
    if let Some(root) = SAVE_ROOT_OVERRIDE.lock().unwrap().clone() {
        return root;
    }
    directories::ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
        .expect("valid application directories")
        .data_local_dir()
        .to_path_buf()
}

#[cfg(test)]
pub(crate) fn set_save_root_for_test(root: Option<PathBuf>) {
    *SAVE_ROOT_OVERRIDE.lock().unwrap() = root;
}

#[cfg(test)]
pub(crate) fn lock_save_root_for_test() -> std::sync::MutexGuard<'static, ()> {
    SAVE_ROOT_GATE.lock().unwrap()
}

pub fn settings_path() -> PathBuf {
    save_root().join("settings.ron")
}

pub fn profile_path(index: u8) -> PathBuf {
    profile_path_in(&save_root(), index)
}

pub fn load_settings() -> GameSettings {
    load_ron::<GameSettings>(&settings_path())
        .unwrap_or_default()
        .migrate()
}

pub fn save_settings(settings: &GameSettings) -> io::Result<()> {
    write_ron(settings_path(), settings)
}

pub fn load_profile(index: u8) -> Result<PlayerProfile, ProfileLoadError> {
    load_profile_at(&profile_path(index))
}

pub fn profile_or_missing_default(
    result: Result<PlayerProfile, ProfileLoadError>,
) -> Result<PlayerProfile, ProfileLoadError> {
    match result {
        Err(ProfileLoadError::Missing) => Ok(PlayerProfile::default()),
        other => other,
    }
}

pub fn save_profile(index: u8, profile: &PlayerProfile) -> io::Result<()> {
    save_profile_at(&profile_path(index), profile)
}

fn load_profile_at(path: &Path) -> Result<PlayerProfile, ProfileLoadError> {
    let loaded = read_profile_file(path);
    if matches!(
        loaded,
        Err(ProfileLoadError::Corrupt(_) | ProfileLoadError::Unreadable(_))
    ) {
        let _ = backup_bad_save(path);
    }
    loaded
}

fn save_profile_at(path: &Path, profile: &PlayerProfile) -> io::Result<()> {
    if matches!(
        read_profile_file(path),
        Err(ProfileLoadError::Corrupt(_) | ProfileLoadError::Unreadable(_))
    ) {
        let _ = backup_bad_save(path);
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "autosave is blocked until the player chooses how to handle this profile",
        ));
    }
    write_ron(path.to_path_buf(), profile)
}

fn read_profile_file(path: &Path) -> Result<PlayerProfile, ProfileLoadError> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(ProfileLoadError::Missing);
        }
        Err(error) => return Err(ProfileLoadError::Unreadable(error)),
    };
    super::migration::profile_from_ron(&contents).map_err(ProfileLoadError::Corrupt)
}

fn backup_bad_save(path: &Path) -> io::Result<PathBuf> {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("profile");
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let destination = unique_corrupt_backup(parent, stem);
    fs::copy(path, &destination)?;
    Ok(destination)
}

fn unique_corrupt_backup(parent: &Path, stem: &str) -> PathBuf {
    let stamp = unix_millis();
    let mut destination = parent.join(format!("{stem}.corrupt-{stamp}.ron"));
    let mut duplicate = 1u32;
    while destination.exists() {
        destination = parent.join(format!("{stem}.corrupt-{stamp}-{duplicate}.ron"));
        duplicate += 1;
    }
    destination
}

fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn profile_path_in(root: &Path, index: u8) -> PathBuf {
    root.join("profiles")
        .join(format!("profile_{}.ron", index.min(PROFILE_COUNT - 1)))
}

fn load_ron<T>(path: &Path) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    let contents = fs::read_to_string(path).ok()?;
    ron::from_str(&contents).ok()
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
    use super::super::profile::PROFILE_VERSION;
    use super::*;
    use crate::items::{ItemQuality, MaterialId};
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
        let loaded: PlayerProfile = load_ron(&path).unwrap();
        assert_eq!(loaded.inventory.count(MaterialId::SlimeGel), 3);
        assert_eq!(loaded.loadout, Loadout::default());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_ron_keeps_original_bytes_and_writes_a_corrupt_backup() {
        let root = temporary_save_root("corrupt");
        let primary = profile_path_in(&root, 0);
        fs::create_dir_all(primary.parent().unwrap()).unwrap();
        let original = b"this is not a profile {{{{".to_vec();
        fs::write(&primary, &original).unwrap();

        let loaded = load_profile_at(&primary);
        assert!(matches!(loaded, Err(ProfileLoadError::Corrupt(_))));
        assert!(matches!(
            profile_or_missing_default(load_profile_at(&primary)),
            Err(ProfileLoadError::Corrupt(_))
        ));
        assert_eq!(fs::read(&primary).unwrap(), original);

        let backups = corrupt_backups(primary.parent().unwrap(), "profile_0");
        assert!(!backups.is_empty());
        assert!(backups
            .iter()
            .all(|backup| fs::read(backup).unwrap() == original));
        assert!(backups.iter().all(|backup| {
            let name = backup.file_name().unwrap().to_string_lossy();
            name.starts_with("profile_0.corrupt-") && name.ends_with(".ron")
        }));

        let save_error = save_profile_at(&primary, &PlayerProfile::default()).unwrap_err();
        assert!(!save_error.to_string().is_empty());
        assert_eq!(fs::read(&primary).unwrap(), original);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_profile_produces_a_default_profile() {
        let root = temporary_save_root("missing");
        let primary = profile_path_in(&root, 0);
        let loaded = profile_or_missing_default(load_profile_at(&primary)).unwrap();
        let expected = PlayerProfile::default();
        assert_eq!(loaded.version, expected.version);
        assert_eq!(loaded.name, expected.name);
        assert_eq!(
            loaded.inventory.total_items(),
            expected.inventory.total_items()
        );
        assert_eq!(loaded.calendar_day, expected.calendar_day);
        assert!(corrupt_backups(primary.parent().unwrap(), "profile_0").is_empty());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn valid_v6_profile_loads_and_migrates_to_current_version() {
        let root = temporary_save_root("v6");
        let primary = profile_path_in(&root, 0);
        let mut profile = PlayerProfile {
            version: 6,
            ..PlayerProfile::default()
        };
        profile.inventory.try_add(MaterialId::SlimeGel, 3);

        save_profile_at(&primary, &profile).unwrap();
        let loaded = load_profile_at(&primary).unwrap();
        assert_eq!(loaded.version, PROFILE_VERSION);
        assert_eq!(loaded.inventory.count(MaterialId::SlimeGel), 3);
        assert!(loaded
            .inventory
            .slots
            .iter()
            .all(|slot| slot.quality == ItemQuality::Normal));
        assert_eq!(loaded.loadout, Loadout::default());
        assert!(corrupt_backups(primary.parent().unwrap(), "profile_0").is_empty());
        let _ = fs::remove_dir_all(&root);
    }

    fn temporary_save_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "destiny_forge_sav02_{}_{label}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("profiles")).unwrap();
        root
    }

    fn corrupt_backups(directory: &Path, stem: &str) -> Vec<PathBuf> {
        let prefix = format!("{stem}.corrupt-");
        fs::read_dir(directory)
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                path.is_file() && name.starts_with(&prefix) && name.ends_with(".ron")
            })
            .collect()
    }
}
