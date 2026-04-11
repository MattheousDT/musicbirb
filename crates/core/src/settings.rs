use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct AppSettings {
	pub active_account_id: Option<String>,
	pub accounts: Vec<AccountConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct AccountConfig {
	pub id: String,
	pub provider: String,
	pub url: String,
	pub username: String,
}

impl AppSettings {
	pub fn load(data_dir: Option<PathBuf>) -> Self {
		let path = Self::get_path(data_dir);
		if let Ok(data) = fs::read_to_string(&path) {
			serde_json::from_str(&data).unwrap_or_default()
		} else {
			Self::default()
		}
	}

	pub fn save(&self, data_dir: Option<PathBuf>) -> Result<(), crate::MusicbirbError> {
		let path = Self::get_path(data_dir);
		if let Some(parent) = path.parent() {
			let _ = fs::create_dir_all(parent);
		}
		let data = serde_json::to_string_pretty(self).map_err(|e| crate::MusicbirbError::Internal(e.to_string()))?;
		fs::write(path, data).map_err(|e| crate::MusicbirbError::Internal(e.to_string()))
	}

	fn get_path(data_dir: Option<PathBuf>) -> PathBuf {
		if let Some(dir) = data_dir {
			dir.join("settings.json")
		} else if let Some(proj_dirs) = ProjectDirs::from("com", "musicbirb", "musicbirb") {
			let dir = proj_dirs.data_dir();
			let _ = fs::create_dir_all(dir);
			dir.join("settings.json")
		} else {
			PathBuf::from("settings.json")
		}
	}
}
