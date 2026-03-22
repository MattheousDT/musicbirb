use super::JellyfinContext;
use crate::error::MusicbirbError;
use crate::models::{TrackId, TrackScrobble};
use crate::providers::ActivityProvider;
use std::sync::Arc;

pub struct JellyfinActivity {
	pub ctx: Arc<JellyfinContext>,
}

#[macros::async_ffi]
impl ActivityProvider for JellyfinActivity {
	async fn now_playing(&self, track_id: &TrackId) -> Result<(), MusicbirbError> {
		let url = format!("{}/Sessions/Playing", self.ctx.base_url);
		let _ = self
			.ctx
			.http
			.post(&url)
			.header("X-Emby-Authorization", self.ctx.auth_header())
			.json(&serde_json::json!({
				"ItemId": track_id.0,
			}))
			.send()
			.await;
		Ok(())
	}

	async fn scrobble(&self, tracks: Vec<TrackScrobble>) -> Result<(), MusicbirbError> {
		let user_id = self.ctx.get_user_id()?;
		for track in tracks {
			let url = format!("{}/Users/{}/PlayedItems/{}", self.ctx.base_url, user_id, track.id.0);
			let _ = self
				.ctx
				.http
				.post(&url)
				.header("X-Emby-Authorization", self.ctx.auth_header())
				.send()
				.await;
		}
		Ok(())
	}
}
