use super::SubsonicContext;
use crate::error::MusicbirbError;
use crate::models::{AlbumDetails, AlbumId, Track};
use crate::providers::AlbumProvider;
use async_trait::async_trait;
use std::sync::Arc;

pub struct SubsonicAlbum {
	pub ctx: Arc<SubsonicContext>,
}

#[async_trait]
impl AlbumProvider for SubsonicAlbum {
	async fn get_album_tracks(&self, album_id: &AlbumId) -> Result<Vec<Track>, MusicbirbError> {
		let album = self
			.ctx
			.client
			.get_album(&album_id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed: {}", e)))?;

		Ok(album.song.into_iter().map(Track::from).collect())
	}

	async fn get_album_details(&self, album_id: &AlbumId) -> Result<AlbumDetails, MusicbirbError> {
		let album = self
			.ctx
			.client
			.get_album(&album_id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to get album details: {}", e)))?;

		Ok(AlbumDetails::from(album))
	}
}
