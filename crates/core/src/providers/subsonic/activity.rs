use super::SubsonicContext;
use crate::error::MusicbirbError;
use crate::models::{TrackId, TrackScrobble};
use crate::providers::ActivityProvider;
use async_trait::async_trait;
use std::sync::Arc;

pub struct SubsonicActivity {
	pub ctx: Arc<SubsonicContext>,
}

#[async_trait]
impl ActivityProvider for SubsonicActivity {
	async fn now_playing(&self, track_id: &TrackId) -> Result<(), MusicbirbError> {
		self.ctx
			.client
			.scrobble(vec![(track_id.0.clone(), None::<usize>)], Some(false))
			.await
			.map_err(|e| MusicbirbError::Api(format!("Now playing failed: {}", e)))?;
		Ok(())
	}

	async fn scrobble(&self, tracks: Vec<TrackScrobble>) -> Result<(), MusicbirbError> {
		let id_at_time: Vec<(String, Option<usize>)> = tracks
			.into_iter()
			.map(|t| (t.id.0, Some(t.timestamp as usize)))
			.collect();

		self.ctx
			.client
			.scrobble(id_at_time, Some(true))
			.await
			.map_err(|e| MusicbirbError::Api(format!("Scrobble failed: {}", e)))?;
		Ok(())
	}
}
