use crate::error::MusicbirbError;
use crate::models::{
	Album, AlbumDetails, AlbumId, Artist, ArtistDetails, ArtistId, CoverArtId, Playlist,
	PlaylistDetails, PlaylistId, Track, TrackId, TrackScrobble,
};
use crate::providers::Provider;
use async_trait::async_trait;
use reqwest::StatusCode;
use std::sync::Arc;
use submarine::api::get_album_list::Order;
use submarine::{Client, auth::AuthBuilder};

pub mod models;

pub struct SubsonicProvider {
	client: Client,
	http_client: reqwest::Client,
	username: String,
}

impl SubsonicProvider {
	pub fn new(url: &str, username: &str, password: &str) -> Result<Self, MusicbirbError> {
		let auth = AuthBuilder::new(username, env!("CARGO_PKG_VERSION"))
			.client_name("musicbirb")
			.hashed(password);

		let client = Client::new(url, auth);

		let http_client = reqwest::ClientBuilder::new()
			.build()
			.map_err(|e| MusicbirbError::Network(format!("HTTP Client Init Error: {}", e)))?;

		Ok(Self {
			client,
			http_client,
			username: username.to_string(),
		})
	}
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn create_subsonic_provider(
	url: String,
	username: String,
	password: String,
) -> Result<Arc<dyn Provider>, MusicbirbError> {
	let provider = SubsonicProvider::new(&url, &username, &password)?;
	Ok(Arc::new(provider))
}

#[async_trait]
impl Provider for SubsonicProvider {
	async fn get_stream_url(&self, track_id: &TrackId) -> Result<String, MusicbirbError> {
		let url = self
			.client
			.stream_url(
				&track_id.0,
				None,
				None::<String>,
				None,
				None::<String>,
				None,
				None,
			)
			.map_err(|e| MusicbirbError::Api(format!("Failed to build stream URL: {}", e)))?;
		Ok(url.to_string())
	}

	async fn get_track(&self, track_id: &TrackId) -> Result<Track, MusicbirbError> {
		let data = self
			.client
			.get_song(&track_id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to fetch track: {}", e)))?;

		Ok(Track::from(data))
	}

	async fn get_album_tracks(&self, album_id: &AlbumId) -> Result<Vec<Track>, MusicbirbError> {
		let album = self
			.client
			.get_album(&album_id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed: {}", e)))?;

		Ok(album.song.into_iter().map(Track::from).collect())
	}

	async fn get_album_details(&self, album_id: &AlbumId) -> Result<AlbumDetails, MusicbirbError> {
		let album = self
			.client
			.get_album(&album_id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to get album details: {}", e)))?;

		Ok(AlbumDetails::from(album))
	}

	async fn get_artist_details(
		&self,
		artist_id: &ArtistId,
	) -> Result<ArtistDetails, MusicbirbError> {
		let mut artist = self
			.client
			.get_artist(&artist_id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to get artist details: {}", e)))?;

		let info = self
			.client
			.get_artist_info2(&artist_id.0, None, None)
			.await
			.ok();

		let top_songs = self
			.client
			.get_top_songs(&artist.base.name, Some(10))
			.await
			.unwrap_or_default();

		let biography = info
			.as_ref()
			.and_then(|i| i.base.biography.clone().into_iter().next());

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

	async fn get_playlist_tracks(
		&self,
		playlist_id: &PlaylistId,
	) -> Result<Vec<Track>, MusicbirbError> {
		let playlist = self
			.client
			.get_playlist(&playlist_id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed: {}", e)))?;

		Ok(playlist.entry.into_iter().map(Track::from).collect())
	}

	async fn get_playlist_details(
		&self,
		playlist_id: &PlaylistId,
	) -> Result<PlaylistDetails, MusicbirbError> {
		let pl_data = self
			.client
			.get_playlist(&playlist_id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed: {}", e)))?;

		Ok(PlaylistDetails::from(pl_data))
	}

	async fn get_cover_art_bytes(&self, cover_id: &CoverArtId) -> Result<Vec<u8>, MusicbirbError> {
		let url = self
			.client
			.get_cover_art_url(&cover_id.0, Some(600))
			.map_err(|e| MusicbirbError::Api(e.to_string()))?;

		let resp = self
			.http_client
			.get(url.clone())
			.send()
			.await
			.map_err(|e| MusicbirbError::Network(e.to_string()))?;

		if resp.status() != StatusCode::OK {
			return Err(MusicbirbError::Api(format!(
				"Image download failed: {}",
				resp.status()
			)));
		}

		let bytes = resp
			.bytes()
			.await
			.map_err(|e| MusicbirbError::Network(e.to_string()))?;
		Ok(bytes.to_vec())
	}

	async fn now_playing(&self, track_id: &TrackId) -> Result<(), MusicbirbError> {
		self.client
			.scrobble(vec![(track_id.0.clone(), None::<usize>)], Some(false))
			.await
			.map_err(|e| MusicbirbError::Api(format!("Now playing failed: {}", e)))?;
		Ok(())
	}

	async fn scrobble(&self, tracks: Vec<TrackScrobble>) -> Result<(), MusicbirbError> {
		let id_at_time: Vec<(String, Option<usize>)> = tracks
			.into_iter()
			.map(|t| (t.id.0, Some(t.timestamp as usize)))
			.collect();

		self.client
			.scrobble(id_at_time, Some(true))
			.await
			.map_err(|e| MusicbirbError::Api(format!("Scrobble failed: {}", e)))?;
		Ok(())
	}

	async fn get_last_played_albums(&self) -> Result<Vec<Album>, MusicbirbError> {
		let list = self
			.client
			.get_album_list2(Order::Recent, Some(20), None, None::<String>)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to get recent albums: {}", e)))?;

		Ok(list.into_iter().map(Album::from).collect())
	}

	async fn get_recently_added_albums(&self) -> Result<Vec<Album>, MusicbirbError> {
		let list = self
			.client
			.get_album_list2(Order::Newest, Some(20), None, None::<String>)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to get newest albums: {}", e)))?;
		Ok(list.into_iter().map(Album::from).collect())
	}

	async fn get_newly_released_albums(&self) -> Result<Vec<Album>, MusicbirbError> {
		let list = self
			.client
			.get_album_list2_by_year(Some(9999), Some(0), Some(20), None, None::<String>)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to get recent albums: {}", e)))?;
		Ok(list.into_iter().map(Album::from).collect())
	}

	async fn get_playlists(&self) -> Result<Vec<Playlist>, MusicbirbError> {
		let list = self
			.client
			.get_playlists(Some(&self.username))
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to get playlists: {}", e)))?;
		Ok(list.into_iter().map(Playlist::from).collect())
	}
}
