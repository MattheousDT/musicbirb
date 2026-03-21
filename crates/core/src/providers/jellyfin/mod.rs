use crate::error::MusicbirbError;
use crate::models::*;
use crate::providers::Provider;
use crate::providers::jellyfin::dto::*;
use async_trait::async_trait;
use reqwest::Client;

pub mod dto;
pub mod models;

#[derive(Clone)]
pub struct JellyfinClient {
	pub base_url: String,
	pub token: Option<String>,
	pub user_id: Option<String>,
	pub http: Client,
}

impl JellyfinClient {
	pub fn new(base_url: &str) -> Self {
		Self {
			base_url: base_url.trim_end_matches('/').to_string(),
			token: None,
			user_id: None,
			http: Client::new(),
		}
	}

	async fn fetch<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, MusicbirbError> {
		let url = format!("{}{}", self.base_url, path);
		let resp = self
			.http
			.get(&url)
			.header("X-Emby-Authorization", self.auth_header())
			.send()
			.await
			.map_err(|e| MusicbirbError::Network(e.to_string()))?;

		if !resp.status().is_success() {
			return Err(MusicbirbError::Api(format!("Jellyfin API error: {}", resp.status())));
		}

		resp.json::<T>().await.map_err(|e| MusicbirbError::Api(e.to_string()))
	}

	pub fn auth_header(&self) -> String {
		format!(
			"MediaBrowser Client=\"Musicbirb\", Device=\"Musicbirb\", DeviceId=\"musicbirb-app\", Version=\"0.1.0\"{}",
			self.token
				.as_ref()
				.map(|t| format!(", Token=\"{}\"", t))
				.unwrap_or_default()
		)
	}

	pub async fn login(&mut self, username: &str, pw: &str) -> Result<AuthResponse, MusicbirbError> {
		let url = format!("{}/Users/AuthenticateByName", self.base_url);
		let resp = self
			.http
			.post(&url)
			.header("X-Emby-Authorization", self.auth_header())
			.json(&AuthRequest { username, pw })
			.send()
			.await
			.map_err(|e| MusicbirbError::Network(e.to_string()))?;

		if !resp.status().is_success() {
			return Err(MusicbirbError::Auth(format!("Jellyfin auth failed: {}", resp.status())));
		}

		let auth_res: AuthResponse = resp.json().await.map_err(|e| MusicbirbError::Api(e.to_string()))?;
		self.token = Some(auth_res.access_token.clone());
		self.user_id = Some(auth_res.user.id.clone());
		Ok(auth_res)
	}

	pub fn set_token(&mut self, token: String) {
		self.token = Some(token);
	}

	pub async fn fetch_me(&mut self) -> Result<(), MusicbirbError> {
		let user: UserDto = self.fetch("/Users/Me").await?;
		self.user_id = Some(user.id);
		Ok(())
	}
}

pub struct JellyfinProvider {
	client: JellyfinClient,
}

impl JellyfinProvider {
	pub fn new(client: JellyfinClient) -> Self {
		Self { client }
	}

	fn user_id(&self) -> Result<&str, MusicbirbError> {
		self.client
			.user_id
			.as_deref()
			.ok_or_else(|| MusicbirbError::Internal("No user ID. Missing call to fetch_me?".into()))
	}
}

#[async_trait]
impl Provider for JellyfinProvider {
	async fn get_stream_url(&self, track_id: &TrackId) -> Result<String, MusicbirbError> {
		let mut url = format!("{}/Audio/{}/stream?static=true", self.client.base_url, track_id.0);
		if let Some(token) = &self.client.token {
			url.push_str(&format!("&api_key={}", token));
		}
		Ok(url)
	}

	async fn get_track(&self, track_id: &TrackId) -> Result<Track, MusicbirbError> {
		let item: BaseItemDto = self
			.client
			.fetch(&format!("/Users/{}/Items/{}", self.user_id()?, track_id.0))
			.await?;
		Ok(Track::from(item))
	}

	async fn get_album_tracks(&self, album_id: &AlbumId) -> Result<Vec<Track>, MusicbirbError> {
		let res: QueryResult<BaseItemDto> = self
			.client
			.fetch(&format!(
				"/Users/{}/Items?ParentId={}&SortBy=ParentIndexNumber,IndexNumber",
				self.user_id()?,
				album_id.0
			))
			.await?;
		Ok(res.items.into_iter().map(Track::from).collect())
	}

	async fn get_album_details(&self, album_id: &AlbumId) -> Result<AlbumDetails, MusicbirbError> {
		let user_id = self.user_id()?;
		let album_dto: BaseItemDto = self
			.client
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

	async fn get_artist_details(&self, artist_id: &ArtistId) -> Result<ArtistDetails, MusicbirbError> {
		let user_id = self.user_id()?;
		let artist_dto: BaseItemDto = self
			.client
			.fetch(&format!("/Users/{}/Items/{}", user_id, artist_id.0))
			.await?;

		let albums_res: QueryResult<BaseItemDto> = self.client.fetch(
			&format!("/Users/{}/Items?ArtistIds={}&IncludeItemTypes=MusicAlbum&SortBy=ProductionYear&SortOrder=Descending&Recursive=true&EnableImages=true", user_id, artist_id.0)
		).await?;
		let albums: Vec<Album> = albums_res.items.into_iter().map(Album::from).collect();

		let top_songs_res: QueryResult<BaseItemDto> = self.client.fetch(
			&format!("/Users/{}/Items?ArtistIds={}&IncludeItemTypes=Audio&SortBy=PlayCount&SortOrder=Descending&Limit=10&Recursive=true&EnableImages=true", user_id, artist_id.0)
		).await?;

		Ok(ArtistDetails {
			id: ArtistId(artist_dto.id.clone()),
			name: artist_dto.name.unwrap_or_else(|| "Unknown".to_string()),
			cover_art: Some(CoverArtId(artist_dto.id)),
			album_count: albums.len() as u32,
			albums,
			biography: artist_dto.overview,
			similar_artists: vec![], // Optional: /Artists/{Id}/Similar
			top_songs: top_songs_res.items.into_iter().map(Track::from).collect(),
		})
	}

	async fn get_playlist_tracks(&self, playlist_id: &PlaylistId) -> Result<Vec<Track>, MusicbirbError> {
		let user_id = self.user_id()?;
		let res: QueryResult<BaseItemDto> = self
			.client
			.fetch(&format!(
				"/Users/{}/Items?ParentId={}&Fields=ItemCounts",
				user_id, playlist_id.0
			))
			.await?;
		Ok(res.items.into_iter().map(Track::from).collect())
	}

	async fn get_playlist_details(&self, playlist_id: &PlaylistId) -> Result<PlaylistDetails, MusicbirbError> {
		let user_id = self.user_id()?;
		let pl_dto: BaseItemDto = self
			.client
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

	fn get_cover_art_url(&self, cover_id: &CoverArtId, size: Option<u32>) -> Result<String, MusicbirbError> {
		let mut url = format!("{}/Items/{}/Images/Primary", self.client.base_url, cover_id.0);
		if let Some(s) = size {
			url.push_str(&format!("?maxWidth={}", s));
		}
		Ok(url)
	}

	async fn get_cover_art_bytes(&self, cover_id: &CoverArtId) -> Result<Vec<u8>, MusicbirbError> {
		let url = self.get_cover_art_url(cover_id, Some(600))?;
		let mut req = self.client.http.get(&url);
		if self.client.token.is_some() {
			req = req.header("X-Emby-Authorization", self.client.auth_header());
		}

		let resp = req.send().await.map_err(|e| MusicbirbError::Network(e.to_string()))?;
		if !resp.status().is_success() {
			return Err(MusicbirbError::Api(format!("Image download failed: {}", resp.status())));
		}
		Ok(resp
			.bytes()
			.await
			.map_err(|e| MusicbirbError::Network(e.to_string()))?
			.to_vec())
	}

	async fn now_playing(&self, track_id: &TrackId) -> Result<(), MusicbirbError> {
		let url = format!("{}/Sessions/Playing", self.client.base_url);
		// Jellyfin expects a POST to report playback start for the active session
		let _ = self
			.client
			.http
			.post(&url)
			.header("X-Emby-Authorization", self.client.auth_header())
			.json(&serde_json::json!({
				"ItemId": track_id.0,
			}))
			.send()
			.await;
		Ok(())
	}

	async fn scrobble(&self, tracks: Vec<TrackScrobble>) -> Result<(), MusicbirbError> {
		let user_id = self.user_id()?;
		for track in tracks {
			let url = format!("{}/Users/{}/PlayedItems/{}", self.client.base_url, user_id, track.id.0);
			// Marking as played in Jellyfin is a POST to the PlayedItems endpoint
			let _ = self
				.client
				.http
				.post(&url)
				.header("X-Emby-Authorization", self.client.auth_header())
				.send()
				.await;
		}
		Ok(())
	}

	async fn get_last_played_albums(&self) -> Result<Vec<Album>, MusicbirbError> {
		let user_id = self.user_id()?;
		let res: QueryResult<BaseItemDto> = self.client.fetch(
			&format!("/Users/{}/Items?IncludeItemTypes=MusicAlbum&SortBy=DatePlayed&SortOrder=Descending&Limit=20&Recursive=true&EnableImages=true", user_id)
		).await?;

		Ok(res.items.into_iter().map(Album::from).collect())
	}

	async fn get_recently_added_albums(&self) -> Result<Vec<Album>, MusicbirbError> {
		// Note: Jellyfin /Items/Latest is recursive by default
		let items: Vec<BaseItemDto> = self
			.client
			.fetch(&format!(
				"/Users/{}/Items/Latest?IncludeItemTypes=MusicAlbum&Limit=20&EnableImages=true",
				self.user_id()?
			))
			.await?;
		Ok(items.into_iter().map(Album::from).collect())
	}

	async fn get_newly_released_albums(&self) -> Result<Vec<Album>, MusicbirbError> {
		let user_id = self.user_id()?;
		let res: QueryResult<BaseItemDto> = self
			.client
			.fetch(&format!(
				"/Users/{}/Items?IncludeItemTypes=MusicAlbum&SortBy=ProductionYear&SortOrder=Descending&Limit=20&Recursive=true",
				user_id
			))
			.await?;
		Ok(res.items.into_iter().map(Album::from).collect())
	}

	async fn get_playlists(&self) -> Result<Vec<Playlist>, MusicbirbError> {
		let user_id = self.user_id()?;
		let res: QueryResult<BaseItemDto> = self
			.client
			.fetch(&format!(
				"/Users/{}/Items?IncludeItemTypes=Playlist&Recursive=true",
				user_id
			))
			.await?;
		Ok(res.items.into_iter().map(Playlist::from).collect())
	}

	async fn ping(&self) -> Result<(), MusicbirbError> {
		let _: () = self.client.fetch("/System/Ping").await?;
		Ok(())
	}
}
