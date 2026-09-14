//! Persistent settings: only the chosen MIDI output.

use std::{
	io,
	path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

pub const CONFIG_FILE: &str = "config.toml";
const APP_DIR: &str = "Rumpus";
const CONFIG_DIR_ENV: &str = "RUMPUS_CONFIG_DIR";

/// A MIDI output port: an endpoint plus the group index used as its MIDI 1.0 port.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OutputSelection {
	pub endpoint_id: String,
	pub group_index: u8,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct AppConfig {
	pub output: Option<OutputSelection>,
}

impl AppConfig {
	/// The directory holding the config file: `RUMPUS_CONFIG_DIR` when set, else the
	/// per-user config directory.
	#[must_use]
	pub fn dir() -> Option<PathBuf> {
		std::env::var_os(CONFIG_DIR_ENV).map(PathBuf::from).or_else(|| dirs::config_dir().map(|d| d.join(APP_DIR)))
	}

	/// Loads the config, falling back to defaults with a warning when the file is unreadable.
	#[must_use]
	pub fn load() -> (Self, Option<String>) {
		Self::dir().map_or_else(|| (Self::default(), None), |dir| Self::load_from(&dir))
	}

	#[must_use]
	pub fn load_from(dir: &Path) -> (Self, Option<String>) {
		let path = dir.join(CONFIG_FILE);
		let text = match std::fs::read_to_string(&path) {
			Ok(text) => text,
			Err(e) if e.kind() == io::ErrorKind::NotFound => return (Self::default(), None),
			Err(e) => return (Self::default(), Some(format!("Could not read {}: {e}", path.display()))),
		};
		match toml::from_str(&text) {
			Ok(config) => (config, None),
			Err(e) => (Self::default(), Some(format!("Ignoring invalid {}: {e}", path.display()))),
		}
	}

	pub fn save(&self) -> io::Result<()> {
		let dir = Self::dir().ok_or_else(|| io::Error::other("no config directory"))?;
		self.save_to(&dir)
	}

	pub fn save_to(&self, dir: &Path) -> io::Result<()> {
		std::fs::create_dir_all(dir)?;
		let text = toml::to_string_pretty(self).map_err(io::Error::other)?;
		std::fs::write(dir.join(CONFIG_FILE), text)
	}
}

#[cfg(test)]
mod tests {
	use std::fs;

	use super::*;

	#[test]
	fn missing_file_yields_defaults_without_warning() {
		let dir = tempfile::tempdir().unwrap();
		let (config, warning) = AppConfig::load_from(dir.path());
		assert_eq!(config, AppConfig::default());
		assert_eq!(warning, None);
	}

	#[test]
	fn output_selection_round_trips() {
		let dir = tempfile::tempdir().unwrap();
		let config = AppConfig {
			output: Some(OutputSelection { endpoint_id: r"\\?\SWD#MIDISRV#...".to_owned(), group_index: 3 }),
		};
		config.save_to(dir.path()).unwrap();
		let (loaded, warning) = AppConfig::load_from(dir.path());
		assert_eq!(loaded, config);
		assert_eq!(warning, None);
	}

	#[test]
	fn corrupt_file_yields_defaults_with_a_warning() {
		let dir = tempfile::tempdir().unwrap();
		fs::write(dir.path().join(CONFIG_FILE), "output = 5").unwrap();
		let (config, warning) = AppConfig::load_from(dir.path());
		assert_eq!(config, AppConfig::default());
		assert!(warning.is_some_and(|w| w.contains(CONFIG_FILE)));
	}

	#[test]
	fn unknown_keys_are_tolerated() {
		let dir = tempfile::tempdir().unwrap();
		fs::write(dir.path().join(CONFIG_FILE), "future_setting = true\n").unwrap();
		let (config, warning) = AppConfig::load_from(dir.path());
		assert_eq!(config, AppConfig::default());
		assert_eq!(warning, None);
	}

	#[test]
	fn save_creates_the_directory() {
		let dir = tempfile::tempdir().unwrap();
		let nested = dir.path().join("Rumpus");
		AppConfig::default().save_to(&nested).unwrap();
		assert!(nested.join(CONFIG_FILE).is_file());
	}
}
