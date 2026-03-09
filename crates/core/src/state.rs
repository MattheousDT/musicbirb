use crate::models::{CoverArtId, Track};
use crate::player::PlayerStatus;
use image::DynamicImage;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CoreState {
	pub queue: Vec<Track>,
	pub queue_position: usize,
	pub time: f64,
	pub status: PlayerStatus,
	pub current_art: Option<Arc<DynamicImage>>,
	pub scrobble_mark_pos: Option<f64>,
}

impl Default for CoreState {
	fn default() -> Self {
		Self {
			queue: Vec::new(),
			queue_position: 0,
			time: 0.0,
			status: PlayerStatus::Stopped,
			current_art: None,
			scrobble_mark_pos: None,
		}
	}
}

pub enum CoreMessage {
	AddTracks(Vec<Track>),
	Next,
	Prev,
	SeekRelative(f64),
	TogglePause,
	UrlReady {
		url: String,
		index: usize,
		is_preload: bool,
	},
	ArtReady {
		art: Arc<DynamicImage>,
		id: CoverArtId,
	},
	ArtDownloaded {
		id: CoverArtId,
		bytes: Vec<u8>,
	},
}
