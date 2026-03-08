use crate::models::TrackId;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrobbleEntry {
	pub id: String,
	pub timestamp: u64,
}

pub struct ScrobbleManager {
	file_path: PathBuf,
	queue: Vec<ScrobbleEntry>,
}

impl ScrobbleManager {
	pub fn new() -> Self {
		let file_path = if let Some(proj_dirs) = ProjectDirs::from("com", "musicbirb", "musicbirb")
		{
			let dir = proj_dirs.data_dir();
			let _ = fs::create_dir_all(dir);
			dir.join("scrobbles.json")
		} else {
			PathBuf::from("scrobbles.json")
		};

		let queue = if let Ok(data) = fs::read_to_string(&file_path) {
			serde_json::from_str(&data).unwrap_or_default()
		} else {
			Vec::new()
		};

		Self { file_path, queue }
	}

	pub fn push(&mut self, track_id: &TrackId, timestamp: u64) {
		self.queue.push(ScrobbleEntry {
			id: track_id.0.clone(),
			timestamp,
		});
		self.save();
	}

	pub fn get_all(&self) -> Vec<(TrackId, u64)> {
		self.queue
			.iter()
			.map(|e| (TrackId(e.id.clone()), e.timestamp))
			.collect()
	}

	pub fn remove_flushed(&mut self, count: usize) {
		if count <= self.queue.len() {
			self.queue.drain(0..count);
			self.save();
		}
	}

	fn save(&self) {
		if let Ok(data) = serde_json::to_string(&self.queue) {
			let _ = fs::write(&self.file_path, data);
		}
	}
}

impl Default for ScrobbleManager {
	fn default() -> Self {
		Self::new()
	}
}
