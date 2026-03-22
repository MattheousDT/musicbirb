use super::{JellyfinContext, dto::*};
use crate::error::MusicbirbError;
use crate::models::{CoverArtId, Playlist, PlaylistDetails, PlaylistId, Track};
use crate::providers::PlaylistProvider;
use std::sync::Arc;

pub struct JellyfinPlaylist {
	pub ctx: Arc<JellyfinContext>,
}

#[macros::async_ffi]
impl PlaylistProvider for JellyfinPlaylist {
	async fn get_playlists(&self) -> Result<Vec<Playlist>, MusicbirbError> {
		let user_id = self.ctx.get_user_id()?;
		let res: QueryResult<BaseItemDto> = self
			.ctx
			.fetch(&format!(
				"/Users/{}/Items?IncludeItemTypes=Playlist&Recursive=true",
				user_id
			))
			.await?;
		Ok(res.items.into_iter().map(Playlist::from).collect())
	}

	async fn get_playlist_tracks(&self, playlist_id: &PlaylistId) -> Result<Vec<Track>, MusicbirbError> {
		let user_id = self.ctx.get_user_id()?;
		let res: QueryResult<BaseItemDto> = self
			.ctx
			.fetch(&format!(
				"/Users/{}/Items?ParentId={}&Fields=ItemCounts",
				user_id, playlist_id.0
			))
			.await?;
		Ok(res.items.into_iter().map(Track::from).collect())
	}

	async fn get_playlist_details(&self, playlist_id: &PlaylistId) -> Result<PlaylistDetails, MusicbirbError> {
		let user_id = self.ctx.get_user_id()?;
		let pl_dto: BaseItemDto = self
			.ctx
			.fetch(&format!("/Users/{}/Items/{}", user_id, playlist_id.0))
			.await?;

		let songs = self.get_playlist_tracks(playlist_id).await?;

		Ok(PlaylistDetails {
			id: PlaylistId(pl_dto.id.clone()),
			name: pl_dto.name.unwrap_or_else(|| "Unknown".to_string()),
			song_count: songs.len() as u32,
			duration_secs: songs.iter().map(|t| t.duration_secs).sum(),
			cover_art: Some(CoverArtId(pl_dto.id)),
			owner: None,
			public: None,
			created_timestamp: 0,
			changed_timestamp: 0,
			comment: pl_dto.overview,
			songs,
		})
	}
}
