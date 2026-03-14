use crate::error::MusicbirbError;
use crate::models::{Album, AlbumId, CoverArtId, Playlist, PlaylistId, Track, TrackId};
use reqwest::StatusCode;
use submarine::api::get_album_list::Order;
use submarine::{Client, auth::AuthBuilder};

pub struct SubsonicClient {
	client: Client,
	http_client: reqwest::Client,
	username: String,
}

impl SubsonicClient {
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

	pub async fn get_stream_url(&self, track_id: &TrackId) -> Result<String, MusicbirbError> {
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

	pub async fn get_track(&self, track_id: &TrackId) -> Result<Track, MusicbirbError> {
		let data = self
			.client
			.get_song(&track_id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to fetch track: {}", e)))?;

		Ok(Track::from(data))
	}

	pub async fn get_album_tracks(&self, album_id: &AlbumId) -> Result<Vec<Track>, MusicbirbError> {
		let album = self
			.client
			.get_album(&album_id.0)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed: {}", e)))?;

		Ok(album.song.into_iter().map(Track::from).collect())
	}

	pub async fn get_playlist_tracks(
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

	pub async fn get_cover_art_bytes(
		&self,
		cover_id: &CoverArtId,
	) -> Result<Vec<u8>, MusicbirbError> {
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

	pub async fn now_playing(&self, track_id: &TrackId) -> Result<(), MusicbirbError> {
		self.client
			.scrobble(vec![(track_id.0.clone(), None::<usize>)], Some(false))
			.await
			.map_err(|e| MusicbirbError::Api(format!("Now playing failed: {}", e)))?;
		Ok(())
	}

	pub async fn scrobble(&self, tracks: &[(TrackId, u64)]) -> Result<(), MusicbirbError> {
		let id_at_time: Vec<(String, Option<usize>)> = tracks
			.iter()
			.map(|(id, time)| (id.0.clone(), Some(*time as usize)))
			.collect();

		self.client
			.scrobble(id_at_time, Some(true))
			.await
			.map_err(|e| MusicbirbError::Api(format!("Scrobble failed: {}", e)))?;
		Ok(())
	}

	pub async fn get_last_played_albums(&self) -> Result<Vec<Album>, MusicbirbError> {
		let list = self
			.client
			.get_album_list2(Order::Recent, Some(20), None, None::<String>)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to get recent albums: {}", e)))?;

		Ok(list.into_iter().map(Album::from).collect())
	}

	pub async fn get_recently_added_albums(&self) -> Result<Vec<Album>, MusicbirbError> {
		let list = self
			.client
			.get_album_list2(Order::Newest, Some(20), None, None::<String>)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to get newest albums: {}", e)))?;
		Ok(list.into_iter().map(Album::from).collect())
	}

	pub async fn get_newly_released_albums(&self) -> Result<Vec<Album>, MusicbirbError> {
		let list = self
			.client
			.get_album_list2_by_year(Some(9999), Some(0), Some(20), None, None::<String>)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to get recent albums: {}", e)))?;
		Ok(list.into_iter().map(Album::from).collect())
	}

	pub async fn get_playlists(&self) -> Result<Vec<Playlist>, MusicbirbError> {
		let list = self
			.client
			.get_playlists(Some(&self.username))
			.await
			.map_err(|e| MusicbirbError::Api(format!("Failed to get playlists: {}", e)))?;
		Ok(list.into_iter().map(Playlist::from).collect())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::env;

	fn init_client() -> SubsonicClient {
		dotenvy::dotenv().ok();
		let url = env::var("SUBSONIC_URL").expect("SUBSONIC_URL not set");
		let user = env::var("SUBSONIC_USER").expect("SUBSONIC_USER not set");
		let pass = env::var("SUBSONIC_PASS").expect("SUBSONIC_PASS not set");
		SubsonicClient::new(&url, &user, &pass).unwrap()
	}

	#[tokio::test]
	async fn test_get_last_played() {
		let client = init_client();
		let albums = client.get_last_played_albums().await.unwrap();
		println!("Last Played: {:?}", albums.len());
		assert!(!albums.is_empty());
	}

	#[tokio::test]
	async fn test_get_recently_added() {
		let client = init_client();
		let albums = client.get_recently_added_albums().await.unwrap();
		println!("Recently Added: {:?}", albums.len());
		assert!(!albums.is_empty());
	}

	#[tokio::test]
	async fn test_newly_released() {
		let client = init_client();
		let albums = client.get_newly_released_albums().await.unwrap();
		println!("Newly Released: {:?}", albums.len());
		assert!(!albums.is_empty());
	}

	#[tokio::test]
	async fn test_get_playlists() {
		let client = init_client();
		let playlists = client.get_playlists().await.unwrap();
		println!("Playlists: {:?}", playlists.len());
	}
}
