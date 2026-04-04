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
		let res = self
			.ctx
			.get_rest_response("getPlaylists", &[("username", &self.ctx.username)])
			.await?;
		let list = res.playlists.map(|p| p.playlist).unwrap_or_default();
		Ok(list.into_iter().map(Playlist::from).collect())
	}

	async fn get_playlist_tracks(&self, playlist_id: &PlaylistId) -> Result<Vec<Track>, MusicbirbError> {
		let res = self
			.ctx
			.get_rest_response("getPlaylist", &[("id", &playlist_id.0)])
			.await?;
		let playlist = res
			.playlist
			.ok_or_else(|| MusicbirbError::Api("Playlist not found".into()))?;

		Ok(playlist.entry.into_iter().map(Track::from).collect())
	}

	async fn create_playlist(
		&self,
		name: &str,
		description: Option<String>,
		public: bool,
	) -> Result<Playlist, MusicbirbError> {
		let res = self.ctx.get_rest_response("createPlaylist", &[("name", name)]).await?;

		let pl = res
			.playlist
			.ok_or_else(|| MusicbirbError::Api("Playlist created but not returned".into()))?;
		let mut playlist = Playlist::from(pl.base);

		if description.is_some() || public {
			self.update_playlist(&playlist.id, None, description.clone(), Some(public))
				.await?;
			playlist.public = Some(public);
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
		let mut params = vec![("playlistId", id.0.as_str())];

		if let Some(n) = &name {
			params.push(("name", n.as_str()));
		}
		if let Some(d) = &description {
			params.push(("comment", d.as_str()));
		}

		let pub_str = public.map(|p| p.to_string());
		if let Some(p) = &pub_str {
			params.push(("public", p.as_str()));
		}

		self.ctx.get_rest_response("updatePlaylist", &params).await?;
		Ok(())
	}

	async fn delete_playlist(&self, id: &PlaylistId) -> Result<(), MusicbirbError> {
		self.ctx.get_rest_response("deletePlaylist", &[("id", &id.0)]).await?;
		Ok(())
	}

	async fn add_to_playlist(&self, id: &PlaylistId, track_ids: Vec<TrackId>) -> Result<(), MusicbirbError> {
		let mut params = vec![("playlistId", id.0.as_str())];
		for tid in &track_ids {
			params.push(("songIdToAdd", tid.0.as_str()));
		}

		self.ctx.get_rest_response("updatePlaylist", &params).await?;
		Ok(())
	}

	async fn remove_from_playlist(&self, id: &PlaylistId, track_indices: Vec<u32>) -> Result<(), MusicbirbError> {
		let mut params = vec![("playlistId", id.0.as_str())];
		let idx_strs: Vec<String> = track_indices.iter().map(|i| i.to_string()).collect();

		for idx in &idx_strs {
			params.push(("songIndexToRemove", idx.as_str()));
		}

		self.ctx.get_rest_response("updatePlaylist", &params).await?;
		Ok(())
	}

	async fn replace_playlist_tracks(&self, id: &PlaylistId, track_ids: Vec<TrackId>) -> Result<(), MusicbirbError> {
		let current_tracks = self.get_playlist_tracks(id).await?;

		let mut remove_params = vec![("playlistId", id.0.as_str())];
		let idx_strs: Vec<String> = (0..current_tracks.len()).map(|i| i.to_string()).collect();

		for idx in &idx_strs {
			remove_params.push(("songIndexToRemove", idx.as_str()));
		}

		if !idx_strs.is_empty() {
			self.ctx.get_rest_response("updatePlaylist", &remove_params).await?;
		}

		if !track_ids.is_empty() {
			self.add_to_playlist(id, track_ids).await?;
		}

		Ok(())
	}

	async fn get_playlist_details(&self, playlist_id: &PlaylistId) -> Result<PlaylistDetails, MusicbirbError> {
		let res = self
			.ctx
			.get_rest_response("getPlaylist", &[("id", &playlist_id.0)])
			.await?;
		let playlist = res
			.playlist
			.ok_or_else(|| MusicbirbError::Api("Playlist not found".into()))?;

		Ok(PlaylistDetails::from(playlist))
	}
}
