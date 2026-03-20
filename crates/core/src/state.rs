use crate::backend::PlayerStatus;
use crate::models::{CoverArtId, Track};
use image::DynamicImage;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct PlaybackSync {
	pub position_secs: f64,
	pub timestamp: Instant,
	pub status: PlayerStatus,
}

#[derive(Debug, Clone)]
pub struct CoreState {
	pub queue: Vec<Track>,
	pub queue_position: usize,
	pub sync: PlaybackSync,
	pub current_art: Option<Arc<DynamicImage>>,
	pub scrobble_mark_pos: Option<f64>,
}

impl Default for CoreState {
	fn default() -> Self {
		Self {
			queue: Vec::new(),
			queue_position: 0,
			sync: PlaybackSync {
				position_secs: 0.0,
				timestamp: Instant::now(),
				status: PlayerStatus::Stopped,
			},
			current_art: None,
			scrobble_mark_pos: None,
		}
	}
}

#[cfg(feature = "ffi")]
#[derive(uniffi::Record, Debug, Clone)]
pub struct UiState {
	pub queue: Vec<Track>,
	pub queue_position: u32,
	pub position_secs: f64,
	pub status: PlayerStatus,
	pub scrobble_mark_pos: Option<f64>,
}

pub enum CoreMessage {
	Shutdown,
	ProviderChanged,
	AddTracks(Vec<Track>, bool),
	ReplaceTracks(Vec<Track>, usize),
	ClearQueue,
	RemoveIndex(usize),
	Next,
	Prev,
	PlayIndex(usize),
	SeekRelative(f64),
	TogglePause,
	UrlReady {
		url: String,
		index: usize,
		is_preload: bool,
	},
	ArtDownloaded {
		id: CoverArtId,
		bytes: Vec<u8>,
	},
}
