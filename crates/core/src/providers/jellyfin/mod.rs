use crate::error::MusicbirbError;
use crate::providers::*;
use async_trait::async_trait;
use dto::*;
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

#[derive(Clone)]
pub struct JellyfinContext {
	pub base_url: String,
	pub token: Option<String>,
	pub user_id: Option<String>,
	pub http: Client,
}

impl JellyfinContext {
	pub fn new(base_url: &str) -> Self {
		Self {
			base_url: base_url.trim_end_matches('/').to_string(),
			token: None,
			user_id: None,
			http: Client::new(),
		}
	}

	pub async fn fetch<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, MusicbirbError> {
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

	pub fn get_user_id(&self) -> Result<&str, MusicbirbError> {
		self.user_id
			.as_deref()
			.ok_or_else(|| MusicbirbError::Internal("No user ID. Missing call to fetch_me?".into()))
	}
}

pub struct JellyfinProvider {
	ctx: Arc<JellyfinContext>,
}

impl JellyfinProvider {
	pub fn new(ctx: JellyfinContext) -> Self {
		Self { ctx: Arc::new(ctx) }
	}
}

#[async_trait]
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

	fn track(&self) -> Arc<dyn TrackProvider> {
		Arc::new(JellyfinTrack {
			ctx: Arc::clone(&self.ctx),
		})
	}

	fn album(&self) -> Arc<dyn AlbumProvider> {
		Arc::new(JellyfinAlbum {
			ctx: Arc::clone(&self.ctx),
		})
	}

	fn artist(&self) -> Arc<dyn ArtistProvider> {
		Arc::new(JellyfinArtist {
			ctx: Arc::clone(&self.ctx),
		})
	}

	fn playlist(&self) -> Arc<dyn PlaylistProvider> {
		Arc::new(JellyfinPlaylist {
			ctx: Arc::clone(&self.ctx),
		})
	}

	fn activity(&self) -> Arc<dyn ActivityProvider> {
		Arc::new(JellyfinActivity {
			ctx: Arc::clone(&self.ctx),
		})
	}

	fn search(&self) -> Arc<dyn SearchProvider> {
		Arc::new(JellyfinSearch {
			ctx: Arc::clone(&self.ctx),
		})
	}
}
