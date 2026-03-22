use super::JellyfinContext;
use crate::error::MusicbirbError;
use crate::models::{AlbumDetails, AlbumId, ArtistId, CoverArtId, Track};
use crate::providers::AlbumProvider;
use crate::providers::jellyfin::dto::{BaseItemDto, QueryResult};
use async_trait::async_trait;
use std::sync::Arc;

pub struct JellyfinAlbum {
	pub ctx: Arc<JellyfinContext>,
}

#[async_trait]
impl AlbumProvider for JellyfinAlbum {
	async fn get_album_tracks(&self, album_id: &AlbumId) -> Result<Vec<Track>, MusicbirbError> {
		let user_id = self.ctx.get_user_id()?;
		let res: QueryResult<BaseItemDto> = self
			.ctx
			.fetch(&format!(
				"/Users/{}/Items?ParentId={}&SortBy=ParentIndexNumber,IndexNumber",
				user_id, album_id.0
			))
			.await?;
		Ok(res.items.into_iter().map(Track::from).collect())
	}

	async fn get_album_details(&self, album_id: &AlbumId) -> Result<AlbumDetails, MusicbirbError> {
		let user_id = self.ctx.get_user_id()?;
		let album_dto: BaseItemDto = self
			.ctx
			.fetch(&format!("/Users/{}/Items/{}", user_id, album_id.0))
			.await?;

		let songs = self.get_album_tracks(album_id).await?;

		Ok(AlbumDetails {
			id: AlbumId(album_dto.id.clone()),
			title: album_dto.name.unwrap_or_else(|| "Unknown".to_string()),
			artist: album_dto
				.artists
				.and_then(|a| a.first().cloned())
				.unwrap_or_else(|| "Unknown".to_string()),
			artist_id: album_dto.artist_items.and_then(|mut a| a.pop().map(|x| ArtistId(x.id))),
			cover_art: Some(CoverArtId(album_dto.id)),
			song_count: songs.len() as u32,
			duration_secs: songs.iter().map(|t| t.duration_secs).sum(),
			year: album_dto.production_year,
			genre: None,
			play_count: None,
			created_timestamp: None,
			starred_timestamp: None,
			songs,
		})
	}
}
