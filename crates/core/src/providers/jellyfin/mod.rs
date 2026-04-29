use crate::error::MusicbirbError;
use crate::providers::*;
use dto::*;
use moka_query::QueryClient;
use reqwest::Client;
use std::sync::Arc;

use activity::*;
use album::*;
use artist::*;
use media::*;
use playlist::*;
use search::*;
use track::*;

pub mod dto;
pub mod models;

pub mod activity;
pub mod album;
pub mod artist;
pub mod media;
pub mod playlist;
pub mod search;
pub mod track;

use std::sync::RwLock;

pub struct JellyfinContext {
	pub base_url: String,
	pub token: RwLock<Option<String>>,
	pub user_id: RwLock<Option<String>>,
	pub credentials: RwLock<Option<(String, String)>>,
	pub http: Client,
}

#[macros::async_ffi]
impl JellyfinContext {
	pub fn new(base_url: &str) -> Self {
		Self {
			base_url: base_url.trim_end_matches('/').to_string(),
			token: RwLock::new(None),
			user_id: RwLock::new(None),
			credentials: RwLock::new(None),
			http: Client::new(),
		}
	}

	pub async fn fetch<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, MusicbirbError> {
		let url = format!("{}{}", self.base_url, path);
		let mut resp = self
			.http
			.get(&url)
			.header("X-Emby-Authorization", self.auth_header())
			.send()
			.await
			.map_err(|e| MusicbirbError::Network(e.to_string()))?;

		if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
			let _ = self.refresh_login().await;
			resp = self
				.http
				.get(&url)
				.header("X-Emby-Authorization", self.auth_header())
				.send()
				.await
				.map_err(|e| MusicbirbError::Network(e.to_string()))?;
		}

		if !resp.status().is_success() {
			return Err(MusicbirbError::Api(format!("Jellyfin API error: {}", resp.status())));
		}

		resp.json::<T>().await.map_err(|e| MusicbirbError::Api(e.to_string()))
	}

	pub fn auth_header(&self) -> String {
		let token = self.token.read().unwrap().clone();
		format!(
			"MediaBrowser Client=\"Musicbirb\", Device=\"Musicbirb\", DeviceId=\"musicbirb-app\", Version=\"0.1.0\"{}",
			token.as_ref().map(|t| format!(", Token=\"{}\"", t)).unwrap_or_default()
		)
	}

	pub async fn refresh_login(&self) -> Result<(), MusicbirbError> {
		let creds = self.credentials.read().unwrap().clone();
		if let Some((username, pw)) = creds {
			let url = format!("{}/Users/AuthenticateByName", self.base_url);
			let resp = self
				.http
				.post(&url)
				.header(
					"X-Emby-Authorization",
					"MediaBrowser Client=\"Musicbirb\", Device=\"Musicbirb\", DeviceId=\"musicbirb-app\", Version=\"0.1.0\"",
				)
				.json(&AuthRequest {
					username: &username,
					pw: &pw,
				})
				.send()
				.await
				.map_err(|e| MusicbirbError::Network(e.to_string()))?;

			if resp.status().is_success() {
				if let Ok(auth_res) = resp.json::<AuthResponse>().await {
					*self.token.write().unwrap() = Some(auth_res.access_token);
					*self.user_id.write().unwrap() = Some(auth_res.user.id);
					return Ok(());
				}
			}
			Err(MusicbirbError::Auth("Failed to re-authenticate".into()))
		} else {
			Err(MusicbirbError::Auth(
				"No credentials available for re-authentication".into(),
			))
		}
	}

	pub async fn login(&self, username: &str, pw: &str) -> Result<AuthResponse, MusicbirbError> {
		*self.credentials.write().unwrap() = Some((username.to_string(), pw.to_string()));
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
		*self.token.write().unwrap() = Some(auth_res.access_token.clone());
		*self.user_id.write().unwrap() = Some(auth_res.user.id.clone());
		Ok(auth_res)
	}

	pub fn set_token(&self, token: String) {
		*self.token.write().unwrap() = Some(token);
	}

	pub async fn fetch_me(&self) -> Result<(), MusicbirbError> {
		let user: UserDto = self.fetch("/Users/Me").await?;
		*self.user_id.write().unwrap() = Some(user.id);
		Ok(())
	}

	pub fn get_user_id(&self) -> Result<String, MusicbirbError> {
		self.user_id
			.read()
			.unwrap()
			.clone()
			.ok_or_else(|| MusicbirbError::Internal("No user ID. Missing call to fetch_me?".into()))
	}
}

pub struct JellyfinProvider {
	ctx: Arc<JellyfinContext>,
	query_client: Arc<QueryClient>,
}

impl JellyfinProvider {
	pub fn new(ctx: JellyfinContext) -> Self {
		Self {
			ctx: Arc::new(ctx),
			query_client: Arc::new(QueryClient::new()),
		}
	}
}

#[macros::async_ffi]
impl Provider for JellyfinProvider {
	async fn ping(&self) -> Result<(), MusicbirbError> {
		let _: () = self.ctx.fetch("/System/Ping").await?;
		Ok(())
	}

	fn media(&self) -> Arc<dyn MediaProvider> {
		Arc::new(JellyfinMedia {
			ctx: Arc::clone(&self.ctx),
		})
	}

	fn track(&self) -> Arc<CachedTrackProvider> {
		Arc::new(crate::providers::CachedTrackProvider::new(
			Arc::new(JellyfinTrack {
				ctx: Arc::clone(&self.ctx),
			}),
			Arc::clone(&self.query_client),
		))
	}

	fn album(&self) -> Arc<CachedAlbumProvider> {
		Arc::new(crate::providers::CachedAlbumProvider::new(
			Arc::new(JellyfinAlbum {
				ctx: Arc::clone(&self.ctx),
			}),
			Arc::clone(&self.query_client),
		))
	}

	fn artist(&self) -> Arc<CachedArtistProvider> {
		Arc::new(crate::providers::CachedArtistProvider::new(
			Arc::new(JellyfinArtist {
				ctx: Arc::clone(&self.ctx),
			}),
			Arc::clone(&self.query_client),
		))
	}

	fn playlist(&self) -> Arc<CachedPlaylistProvider> {
		Arc::new(crate::providers::CachedPlaylistProvider::new(
			Arc::new(JellyfinPlaylist {
				ctx: Arc::clone(&self.ctx),
			}),
			Arc::clone(&self.query_client),
		))
	}

	fn activity(&self) -> Arc<CachedActivityProvider> {
		Arc::new(crate::providers::CachedActivityProvider::new(
			Arc::new(JellyfinActivity {
				ctx: Arc::clone(&self.ctx),
			}),
			Arc::clone(&self.query_client),
		))
	}

	fn search(&self) -> Arc<CachedSearchProvider> {
		Arc::new(crate::providers::CachedSearchProvider::new(
			Arc::new(JellyfinSearch {
				ctx: Arc::clone(&self.ctx),
			}),
			Arc::clone(&self.query_client),
		))
	}
}
