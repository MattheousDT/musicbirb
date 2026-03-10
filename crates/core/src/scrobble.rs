use crate::models::TrackId;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

	pub fn save(&self) {
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

pub struct ScrobbleTracker {
	pub accumulated_time: f64,
	pub last_pos: f64,
	pub has_scrobbled: bool,
	pub start_time: u64,
}

impl ScrobbleTracker {
	pub fn new() -> Self {
		Self {
			accumulated_time: 0.0,
			last_pos: 0.0,
			has_scrobbled: false,
			start_time: 0,
		}
	}

	pub fn reset(&mut self) {
		self.accumulated_time = 0.0;
		self.last_pos = 0.0;
		self.has_scrobbled = false;
		self.start_time = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64;
	}

	pub fn get_remaining_duration(&self, track_duration_secs: u32) -> Option<Duration> {
		if track_duration_secs < 30 {
			return None;
		}

		let threshold = (track_duration_secs as f64 / 2.0).min(240.0);
		let remaining = threshold - self.accumulated_time;

		if remaining > 0.0 {
			Some(Duration::from_secs_f64(remaining))
		} else {
			None
		}
	}

	pub fn commit_played_time(&mut self, duration: Duration) {
		self.accumulated_time += duration.as_secs_f64();
	}

	pub fn sync_position(&mut self, current_pos: f64) {
		self.last_pos = current_pos;
	}

	pub fn get_mark_pos(&self, track_duration_secs: u32) -> Option<f64> {
		let duration = track_duration_secs as f64;
		if duration < 30.0 {
			return None;
		}
		let threshold = (duration / 2.0).min(240.0);
		let remaining = threshold - self.accumulated_time;
		if remaining > 0.0 {
			let m = self.last_pos + remaining;
			if m <= duration + 1.0 { Some(m) } else { None }
		} else {
			None
		}
	}
}
