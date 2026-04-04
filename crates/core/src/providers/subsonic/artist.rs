use super::{ServerType, SubsonicContext, navidrome_dto::*};
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
		// --- Navidrome Branch ---
		if self.ctx.server_type == ServerType::Navidrome {
			let nd_artist: NavidromeArtist = self.ctx.get_nd_api(&format!("artist/{}", artist_id.0), &[]).await?;

			let nd_albums: Vec<NavidromeAlbum> = self
				.ctx
				.get_nd_api(
					"album",
					&[
						("artist_id", &artist_id.0),
						("_start", "0"),
						("_end", "-1"),
						("_sort", "min_year"),
						("_order", "DESC"),
					],
				)
				.await?;

			let top_songs = self
				.ctx
				.get_rest_response("getTopSongs", &[("artist", &nd_artist.name), ("count", "10")])
				.await
				.ok()
				.and_then(|r| r.top_songs)
				.map(|ts| ts.song.into_iter().map(Track::from).collect())
				.unwrap_or_default();

			let info = self
				.ctx
				.get_rest_response("getArtistInfo2", &[("id", &artist_id.0)])
				.await
				.ok()
				.and_then(|r| r.artist_info2);
			let biography = info.as_ref().and_then(|i| i.biography.clone());
			let similar_artists = info
				.map(|i| i.similar_artist.into_iter().map(Artist::from).collect())
				.unwrap_or_default();

			return Ok(ArtistDetails {
				id: ArtistId(nd_artist.id.clone()),
				name: nd_artist.name,
				cover_art: Some(CoverArtId(nd_artist.id)),
				album_count: nd_artist.album_count.unwrap_or(0) as u32,
				song_count: nd_artist.song_count.unwrap_or(0) as u32,
				albums: nd_albums.into_iter().map(Album::from).collect(),
				biography,
				similar_artists,
				top_songs,
				starred: nd_artist.starred,
				musicbrainz_id: nd_artist.mbz_artist_id,
				lastfm_url: nd_artist.external_url,
			});
		}

		// --- Standard OpenSubsonic Branch ---
		let mut artist = self
			.ctx
			.get_rest_response("getArtist", &[("id", &artist_id.0)])
			.await?
			.artist
			.ok_or_else(|| MusicbirbError::Api("Artist not found".into()))?;

		let info = self
			.ctx
			.get_rest_response("getArtistInfo2", &[("id", &artist_id.0)])
			.await
			.ok()
			.and_then(|r| r.artist_info2);

		let top_songs = self
			.ctx
			.get_rest_response("getTopSongs", &[("artist", &artist.name), ("count", "20")])
			.await
			.ok()
			.and_then(|r| r.top_songs)
			.map(|ts| ts.song.into_iter().map(Track::from).collect())
			.unwrap_or_default();

		// OpenSubsonic getArtist usually returns albums in ASC order, reverse for UI (newest first)
		artist.album.reverse();

		Ok(ArtistDetails {
			id: ArtistId(artist.id),
			name: artist.name,
			cover_art: artist.cover_art.map(CoverArtId),
			album_count: artist.album_count.unwrap_or(0) as u32,
			song_count: artist.album.iter().fold(0, |acc, e| acc + e.track.unwrap_or(0) as u32),
			albums: artist.album.into_iter().map(Album::from).collect(),
			biography: info.as_ref().and_then(|i| i.biography.clone()),
			similar_artists: info
				.map(|i| i.similar_artist.into_iter().map(Artist::from).collect())
				.unwrap_or_default(),
			top_songs,
			starred: artist.starred.map(|_| true),
			musicbrainz_id: artist.music_brainz_id,
			lastfm_url: artist.last_fm_url,
		})
	}
}
