use super::{JellyfinContext, dto::*};
use crate::error::MusicbirbError;
use crate::models::{Track, TrackId};
use crate::providers::TrackProvider;
use std::sync::Arc;

pub struct JellyfinTrack {
	pub ctx: Arc<JellyfinContext>,
}

#[macros::async_ffi]
impl TrackProvider for JellyfinTrack {
	async fn get_track(&self, track_id: &TrackId) -> Result<Track, MusicbirbError> {
		let item: BaseItemDto = self
			.ctx
			.fetch(&format!("/Users/{}/Items/{}", self.ctx.get_user_id()?, track_id.0))
			.await?;
		Ok(Track::from(item))
	}

	async fn star_track(&self, _track_id: &TrackId) -> Result<(), MusicbirbError> {
		todo!()
	}

	async fn unstar_track(&self, _track_id: &TrackId) -> Result<(), MusicbirbError> {
		todo!()
	}
}
