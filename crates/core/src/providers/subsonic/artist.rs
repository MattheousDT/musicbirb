use super::SubsonicContext;
use crate::error::MusicbirbError;
use crate::models::{Album, Artist, ArtistDetails, ArtistId, CoverArtId, Track};
use crate::providers::ArtistProvider;
use std::sync::Arc;

pub struct SubsonicArtist {
	pub ctx: Arc<SubsonicContext>,
}

#[macros::async_ffi]
impl ArtistProvider for SubsonicArtist {
	async fn get_artist_details(&self, artist_id: &ArtistId) -> Result<ArtistDetails, MusicbirbError> {
		let mut artist = self
			.ctx
			.client
			.get_artist(&artist_id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to get artist details: {}", e)))?;

		let info = self.ctx.client.get_artist_info(&artist_id.0, None, None).await.ok();

		let top_songs = self
			.ctx
			.client
			.get_top_songs(&artist.base.name, Some(10))
			.await
			.unwrap_or_default();

		let biography = info.as_ref().and_then(|i| i.base.biography.clone().into_iter().next());

		let similar_artists = info
			.map(|i| i.similar_artist.into_iter().map(Artist::from).collect())
			.unwrap_or_default();

		artist.album.reverse();

		Ok(ArtistDetails {
			id: ArtistId(artist.base.id),
			name: artist.base.name,
			cover_art: artist.base.cover_art.map(CoverArtId),
			album_count: artist.base.album_count as u32,
			albums: artist.album.into_iter().map(Album::from).collect(),
			biography,
			similar_artists,
			top_songs: top_songs.into_iter().map(Track::from).collect(),
		})
	}
}
