use super::SubsonicContext;
use crate::error::MusicbirbError;
use crate::models::{Track, TrackId};
use crate::providers::TrackProvider;
use std::sync::Arc;

pub struct SubsonicTrack {
	pub ctx: Arc<SubsonicContext>,
}

#[macros::async_ffi]
impl TrackProvider for SubsonicTrack {
	async fn get_track(&self, track_id: &TrackId) -> Result<Track, MusicbirbError> {
		let data = self
			.ctx
			.client
			.get_song(&track_id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to fetch track: {}", e)))?;

		Ok(Track::from(data))
	}
}
