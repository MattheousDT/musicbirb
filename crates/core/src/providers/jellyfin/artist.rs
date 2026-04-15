use super::{JellyfinContext, dto::*};
use crate::error::MusicbirbError;
use crate::models::{Album, ArtistDetails, ArtistId, CoverArtId, Track};
use crate::providers::ArtistProvider;
use std::sync::Arc;

pub struct JellyfinArtist {
	pub ctx: Arc<JellyfinContext>,
}

#[macros::async_ffi]
impl ArtistProvider for JellyfinArtist {
	async fn get_artist_details(&self, artist_id: &ArtistId) -> Result<ArtistDetails, MusicbirbError> {
		let user_id = self.ctx.get_user_id()?;
		let artist_dto: BaseItemDto = self
			.ctx
			.fetch(&format!("/Users/{}/Items/{}", user_id, artist_id.0))
			.await?;

		let albums_res: QueryResult<BaseItemDto> = self.ctx.fetch(
			&format!("/Users/{}/Items?ArtistIds={}&IncludeItemTypes=MusicAlbum&SortBy=ProductionYear&SortOrder=Descending&Recursive=true&EnableImages=true", user_id, artist_id.0)
		).await?;
		let albums: Vec<Album> = albums_res.items.into_iter().map(Album::from).collect();

		let top_songs_res: QueryResult<BaseItemDto> = self.ctx.fetch(
			&format!("/Users/{}/Items?ArtistIds={}&IncludeItemTypes=Audio&SortBy=PlayCount&SortOrder=Descending&Limit=10&Recursive=true&EnableImages=true", user_id, artist_id.0)
		).await?;

		Ok(ArtistDetails {
			id: ArtistId(artist_dto.id.clone()),
			name: artist_dto.name.unwrap_or_else(|| "Unknown".to_string()),
			cover_art: Some(CoverArtId(artist_dto.id)),
			album_count: albums.len() as u32,
			appears_on_count: 0,
			song_count: albums.iter().fold(0, |acc, e| acc + e.song_count.unwrap_or(0)),
			albums,
			biography: artist_dto.overview,
			similar_artists: vec![],
			top_songs: top_songs_res.items.into_iter().map(Track::from).collect(),
			starred: None,
			musicbrainz_id: None,
			lastfm_url: None,
			appears_on: vec![],
			starred_songs: vec![],
		})
	}

	async fn get_top_songs(&self, _artist_id: &ArtistId) -> Result<Vec<Track>, MusicbirbError> {
		todo!()
	}

	async fn get_personal_top_songs(&self, _artist_id: &ArtistId) -> Result<Vec<Track>, MusicbirbError> {
		todo!()
	}

	async fn star_artist(&self, _artist_id: &ArtistId) -> Result<(), MusicbirbError> {
		todo!()
	}

	async fn unstar_artist(&self, _artist_id: &ArtistId) -> Result<(), MusicbirbError> {
		todo!()
	}
}
