use super::SubsonicContext;
use crate::error::MusicbirbError;
use crate::models::{Playlist, PlaylistDetails, PlaylistId, Track};
use crate::providers::PlaylistProvider;
use std::sync::Arc;

pub struct SubsonicPlaylist {
	pub ctx: Arc<SubsonicContext>,
}

#[macros::async_ffi]
impl PlaylistProvider for SubsonicPlaylist {
	async fn get_playlists(&self) -> Result<Vec<Playlist>, MusicbirbError> {
		let list = self
			.ctx
			.client
			.get_playlists(Some(&self.ctx.username))
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to get playlists: {}", e)))?;
		Ok(list.into_iter().map(Playlist::from).collect())
	}

	async fn get_playlist_tracks(&self, playlist_id: &PlaylistId) -> Result<Vec<Track>, MusicbirbError> {
		let playlist = self
			.ctx
			.client
			.get_playlist(&playlist_id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed: {}", e)))?;

		Ok(playlist.entry.into_iter().map(Track::from).collect())
	}

	async fn get_playlist_details(&self, playlist_id: &PlaylistId) -> Result<PlaylistDetails, MusicbirbError> {
		let pl_data = self
			.ctx
			.client
			.get_playlist(&playlist_id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed: {}", e)))?;

		Ok(PlaylistDetails::from(pl_data))
	}
}
