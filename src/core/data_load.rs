//! Shared RON content loading: prefer on-disk assets, fall back to embedded strings.

use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;

/// Load RON content for game data files.
///
/// Order:
/// 1. `assets/...` on disk (content iteration without rebuild when cwd is repo root)
/// 2. Embedded string baked via `include_str!`
///
/// Invalid RON logs an error and returns `None` so callers can fall back safely.
pub fn load_ron_from_assets_or_embedded<T>(
    relative_path: &str,
    embedded: &str,
    label: &str,
) -> Option<T>
where
    T: DeserializeOwned,
{
    if Path::new(relative_path).is_file() {
        match fs::read_to_string(relative_path) {
            Ok(text) => match parse_ron::<T>(&text) {
                Ok(value) => return Some(value),
                Err(err) => {
                    bevy::log::error!(
                        "Invalid {label} at {relative_path}: {err}; trying embedded fallback"
                    );
                }
            },
            Err(err) => {
                bevy::log::warn!(
                    "Could not read {relative_path}: {err}; using embedded {label}"
                );
            }
        }
    }

    match parse_ron::<T>(embedded) {
        Ok(value) => Some(value),
        Err(err) => {
            bevy::log::error!("Invalid embedded {label}: {err}");
            None
        }
    }
}

fn parse_ron<T: DeserializeOwned>(source: &str) -> Result<T, String> {
    ron::from_str(source).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Sample {
        value: u32,
    }

    #[test]
    fn parses_valid_ron() {
        let parsed: Sample = parse_ron("(value: 7)").unwrap();
        assert_eq!(parsed, Sample { value: 7 });
    }

    #[test]
    fn rejects_invalid_ron() {
        assert!(parse_ron::<Sample>("not ron").is_err());
    }
}
