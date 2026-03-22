use super::{JellyfinContext, dto::*};
use crate::error::MusicbirbError;
use crate::models::{Track, TrackId};
use crate::providers::TrackProvider;
use async_trait::async_trait;
use std::sync::Arc;

pub struct JellyfinTrack {
	pub ctx: Arc<JellyfinContext>,
}

#[async_trait]
impl TrackProvider for JellyfinTrack {
	async fn get_track(&self, track_id: &TrackId) -> Result<Track, MusicbirbError> {
		let item: BaseItemDto = self
			.ctx
			.fetch(&format!("/Users/{}/Items/{}", self.ctx.get_user_id()?, track_id.0))
			.await?;
		Ok(Track::from(item))
	}
}
