use super::SubsonicContext;
use crate::error::MusicbirbError;
use crate::models::{TrackId, TrackScrobble};
use crate::providers::ActivityProvider;
use std::sync::Arc;

pub struct SubsonicActivity {
	pub ctx: Arc<SubsonicContext>,
}

#[macros::async_ffi]
impl ActivityProvider for SubsonicActivity {
	async fn now_playing(&self, track_id: &TrackId) -> Result<(), MusicbirbError> {
		self.ctx
			.get_rest_response("scrobble", &[("id", &track_id.0), ("submission", "false")])
			.await?;
		Ok(())
	}

	async fn scrobble(&self, tracks: Vec<TrackScrobble>) -> Result<(), MusicbirbError> {
		let mut params = vec![("submission", "true")];
		let time_strs: Vec<String> = tracks.iter().map(|t| t.timestamp.to_string()).collect();

		for (i, track) in tracks.iter().enumerate() {
			params.push(("id", track.id.0.as_str()));
			params.push(("time", time_strs[i].as_str()));
		}

		self.ctx.get_rest_response("scrobble", &params).await?;
		Ok(())
	}
}
