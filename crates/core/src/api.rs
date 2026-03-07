use crate::error::CoreError;
use crate::models::{AlbumId, CoverArtId, PlaylistId, Track, TrackId};
use reqwest::StatusCode;
use submarine::{Client, auth::AuthBuilder};

pub struct SubsonicClient {
	client: Client,
	http_client: reqwest::Client,
}

impl SubsonicClient {
	pub fn new(url: &str, username: &str, password: &str) -> Result<Self, CoreError> {
		let auth = AuthBuilder::new(username, env!("CARGO_PKG_VERSION"))
			.client_name("musicbirb")
			.hashed(password);

		let client = Client::new(url, auth);
		let http_client = reqwest::Client::new();

		Ok(Self {
			client,
			http_client,
		})
	}

	pub async fn get_stream_url(&self, track_id: &TrackId) -> Result<String, CoreError> {
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
			.map_err(|e| CoreError::Api(format!("Failed to build stream URL: {}", e)))?;
		Ok(url.to_string())
	}

	pub async fn get_track(&self, track_id: &TrackId) -> Result<Track, CoreError> {
		let data = self
			.client
			.get_song(&track_id.0)
			.await
			.map_err(|e| CoreError::Api(format!("Failed to fetch track: {}", e)))?;

		Ok(Track::from(data))
	}

	pub async fn get_album_tracks(&self, album_id: &AlbumId) -> Result<Vec<Track>, CoreError> {
		let album = self
			.client
			.get_album(&album_id.0)
			.await
			.map_err(|e| CoreError::Api(format!("Failed: {}", e)))?;

		Ok(album.song.into_iter().map(Track::from).collect())
	}

	pub async fn get_playlist_tracks(
		&self,
		playlist_id: &PlaylistId,
	) -> Result<Vec<Track>, CoreError> {
		let playlist = self
			.client
			.get_playlist(&playlist_id.0)
			.await
			.map_err(|e| CoreError::Api(format!("Failed: {}", e)))?;

		Ok(playlist.entry.into_iter().map(Track::from).collect())
	}

	pub async fn get_cover_art_bytes(&self, cover_id: &CoverArtId) -> Result<Vec<u8>, CoreError> {
		let url = self
			.client
			.get_cover_art_url(&cover_id.0, Some(600))
			.map_err(|e| CoreError::Api(e.to_string()))?;

		let resp = self.http_client.get(url.clone()).send().await?;

		if resp.status() != StatusCode::OK {
			return Err(CoreError::Api(format!(
				"Image download failed: {}",
				resp.status()
			)));
		}

		let bytes = resp.bytes().await?;
		Ok(bytes.to_vec())
	}
}
