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

			let (mut albums, mut appears_on) = (Vec::new(), Vec::new());
			for alb in nd_albums {
				let model_alb = Album::from(alb.clone());
				if alb.artist_id.as_ref() == Some(&artist_id.0) || alb.album_artist_id.as_ref() == Some(&artist_id.0) {
					albums.push(model_alb);
				} else {
					appears_on.push(model_alb);
				}
			}

			let info = self
				.ctx
				.get_rest_response("getArtistInfo2", &[("id", &artist_id.0)])
				.await
				.ok()
				.and_then(|r| r.artist_info2);

			let starred_songs = self
				.ctx
				.get_nd_api::<Vec<NavidromeSong>>("song", &[("artist_id", &artist_id.0), ("starred", "true")])
				.await
				.unwrap_or_default()
				.into_iter()
				.map(Track::from)
				.collect();

			return Ok(ArtistDetails {
				id: ArtistId(nd_artist.id.clone()),
				name: nd_artist.name,
				cover_art: Some(CoverArtId(nd_artist.id)),
				album_count: albums.len() as u32,
				appears_on_count: appears_on.len() as u32,
				song_count: nd_artist.song_count.unwrap_or(0) as u32,
				albums,
				appears_on,
				biography: info.as_ref().and_then(|i| i.biography.clone()),
				similar_artists: info
					.map(|i| i.similar_artist.into_iter().map(Artist::from).collect())
					.unwrap_or_default(),
				top_songs: Vec::new(),
				starred_songs,
				starred: nd_artist.starred,
				musicbrainz_id: nd_artist.mbz_artist_id,
				lastfm_url: nd_artist.external_url,
			});
		}

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

		artist.album.reverse();

		let (mut albums, mut appears_on) = (Vec::new(), Vec::new());
		for alb_dto in artist.album.clone() {
			let model_alb = Album::from(alb_dto.clone());
			if alb_dto.artist_id.as_ref() == Some(&artist_id.0) {
				albums.push(model_alb);
			} else {
				appears_on.push(model_alb);
			}
		}

		Ok(ArtistDetails {
			id: ArtistId(artist.id),
			name: artist.name,
			cover_art: artist.cover_art.map(CoverArtId),
			album_count: albums.len() as u32,
			appears_on_count: appears_on.len() as u32,
			song_count: artist.album.iter().fold(0, |acc, e| acc + e.track.unwrap_or(0) as u32),
			albums,
			appears_on,
			biography: info.as_ref().and_then(|i| i.biography.clone()),
			similar_artists: info
				.map(|i| i.similar_artist.into_iter().map(Artist::from).collect())
				.unwrap_or_default(),
			top_songs: Vec::new(),
			starred_songs: Vec::new(),
			starred: artist.starred.map(|_| true),
			musicbrainz_id: artist.music_brainz_id,
			lastfm_url: artist.last_fm_url,
		})
	}

	async fn get_top_songs(&self, artist_id: &ArtistId) -> Result<Vec<Track>, MusicbirbError> {
		let name = if self.ctx.server_type == ServerType::Navidrome {
			self.ctx
				.get_nd_api::<NavidromeArtist>(&format!("artist/{}", artist_id.0), &[])
				.await?
				.name
		} else {
			self.ctx
				.get_rest_response("getArtist", &[("id", &artist_id.0)])
				.await?
				.artist
				.ok_or_else(|| MusicbirbError::Api("Artist not found".into()))?
				.name
		};

		Ok(self
			.ctx
			.get_rest_response("getTopSongs", &[("artist", &name), ("count", "20")])
			.await?
			.top_songs
			.map(|ts| ts.song.into_iter().map(Track::from).collect())
			.unwrap_or_default())
	}

	async fn get_personal_top_songs(&self, artist_id: &ArtistId) -> Result<Vec<Track>, MusicbirbError> {
		if self.ctx.server_type == ServerType::Navidrome {
			let songs: Vec<NavidromeSong> = self
				.ctx
				.get_nd_api(
					"song",
					&[
						("artist_id", &artist_id.0),
						("_sort", "playCount"),
						("_order", "DESC"),
						("_start", "0"),
						("_end", "20"),
					],
				)
				.await?;
			return Ok(songs.into_iter().map(Track::from).collect());
		}

		self.get_top_songs(artist_id).await
	}
}
