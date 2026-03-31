use super::SubsonicContext;
use crate::error::MusicbirbError;
use crate::models::{Playlist, PlaylistDetails, PlaylistId, Track, TrackId};
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

	async fn create_playlist(
		&self,
		name: &str,
		description: Option<String>,
		public: bool,
	) -> Result<Playlist, MusicbirbError> {
		// Subsonic handles creation and setting public/comment separately in most clients
		let pl_data = self
			.ctx
			.client
			.create_playlist(name, vec![] as Vec<String>)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to create playlist: {}", e)))?;

		let playlist = Playlist::from(pl_data.base);

		if description.is_some() || public {
			self.update_playlist(&playlist.id, None, description, Some(public))
				.await?;
		}

		Ok(playlist)
	}

	async fn update_playlist(
		&self,
		id: &PlaylistId,
		name: Option<String>,
		description: Option<String>,
		public: Option<bool>,
	) -> Result<(), MusicbirbError> {
		self.ctx
			.client
			.update_playlist(
				&id.0,
				name,
				description,
				public,
				vec![] as Vec<String>,
				vec![] as Vec<i64>,
			)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to update playlist: {}", e)))?;
		Ok(())
	}

	async fn delete_playlist(&self, id: &PlaylistId) -> Result<(), MusicbirbError> {
		self.ctx
			.client
			.delete_playlist(&id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to delete playlist: {}", e)))?;
		Ok(())
	}

	async fn add_to_playlist(&self, id: &PlaylistId, track_ids: Vec<TrackId>) -> Result<(), MusicbirbError> {
		let ids_to_add: Vec<String> = track_ids.into_iter().map(|tid| tid.0).collect();

		self.ctx
			.client
			.update_playlist(
				&id.0,
				None::<String>,
				None::<String>,
				None,
				ids_to_add,
				vec![] as Vec<i64>,
			)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to add to playlist: {}", e)))?;
		Ok(())
	}

	async fn remove_from_playlist(&self, id: &PlaylistId, track_indices: Vec<u32>) -> Result<(), MusicbirbError> {
		let indices: Vec<i64> = track_indices.into_iter().map(|i| i as i64).collect();
		self.ctx
			.client
			.update_playlist(
				&id.0,
				None::<String>,
				None::<String>,
				None,
				vec![] as Vec<String>,
				indices,
			)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to remove from playlist: {}", e)))?;
		Ok(())
	}

	async fn replace_playlist_tracks(&self, id: &PlaylistId, track_ids: Vec<TrackId>) -> Result<(), MusicbirbError> {
		let current_tracks = self.get_playlist_tracks(id).await?;
		let indices_to_remove: Vec<i64> = (0..current_tracks.len() as i64).collect();

		// Split into two atomic API calls since mutating indices while appending at the same time is prone to race conditions in some servers
		if !indices_to_remove.is_empty() {
			self.ctx
				.client
				.update_playlist(
					&id.0,
					None::<String>,
					None::<String>,
					None,
					vec![] as Vec<String>,
					indices_to_remove,
				)
				.await
				.map_err(|e| MusicbirbError::Api(format!("Failed to clear playlist: {}", e)))?;
		}

		if !track_ids.is_empty() {
			self.add_to_playlist(id, track_ids).await?;
		}

		Ok(())
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
