use super::SubsonicContext;
use crate::error::MusicbirbError;
use crate::models::{AlbumDetails, AlbumId, Track};
use crate::providers::AlbumProvider;
use std::sync::Arc;

pub struct SubsonicAlbum {
	pub ctx: Arc<SubsonicContext>,
}

#[macros::async_ffi]
impl AlbumProvider for SubsonicAlbum {
	async fn get_album_tracks(&self, album_id: &AlbumId) -> Result<Vec<Track>, MusicbirbError> {
		let res = self.ctx.get_rest_response("getAlbum", &[("id", &album_id.0)]).await?;
		let album = res.album.ok_or_else(|| MusicbirbError::Api("Album not found".into()))?;

		Ok(album.song.into_iter().map(Track::from).collect())
	}

	async fn get_album_details(&self, album_id: &AlbumId) -> Result<AlbumDetails, MusicbirbError> {
		let res = self.ctx.get_rest_response("getAlbum", &[("id", &album_id.0)]).await?;
		let album = res.album.ok_or_else(|| MusicbirbError::Api("Album not found".into()))?;

		Ok(AlbumDetails::from(album))
	}

	async fn star_album(&self, album_id: &AlbumId) -> Result<(), MusicbirbError> {
		let _ = self.ctx.get_rest_response("star", &[("albumId", &album_id.0)]).await?;

		Ok(())
	}

	async fn unstar_album(&self, album_id: &AlbumId) -> Result<(), MusicbirbError> {
		let _ = self
			.ctx
			.get_rest_response("unstar", &[("albumId", &album_id.0)])
			.await?;

		Ok(())
	}
}
