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
		let res = self.ctx.get_rest_response("getSong", &[("id", &track_id.0)]).await?;
		let song = res.song.ok_or_else(|| MusicbirbError::Api("Song not found".into()))?;

		Ok(Track::from(song))
	}

	async fn star_track(&self, track_id: &TrackId) -> Result<(), MusicbirbError> {
		let _ = self.ctx.get_rest_response("star", &[("id", &track_id.0)]).await?;

		Ok(())
	}

	async fn unstar_track(&self, track_id: &TrackId) -> Result<(), MusicbirbError> {
		let _ = self.ctx.get_rest_response("unstar", &[("id", &track_id.0)]).await?;

		Ok(())
	}
}
