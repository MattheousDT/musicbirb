use crate::MusicbirbError;
use crate::providers::*;
use moka_query::QueryClient;
use reqwest::{Client, Url};
use std::sync::{Arc, RwLock};

use activity::*;
use album::*;
use artist::*;
use media::*;
use playlist::*;
use search::*;
use track::*;

pub mod activity;
pub mod album;
pub mod artist;
pub mod media;
pub mod playlist;
pub mod search;
pub mod track;

pub mod dto;
pub mod models;
pub mod navidrome_dto;

#[derive(Clone, PartialEq, Eq)]
pub enum ServerType {
	Subsonic,
	Navidrome,
}

pub struct SubsonicContext {
	pub server_type: ServerType,
	pub base_url: String,
	pub username: String,
	pub pass_or_token: String,
	pub client: Client,
	pub nd_jwt: RwLock<Option<String>>,
	pub nd_id: RwLock<Option<String>>,
}

impl SubsonicContext {
	pub fn build_rest_url(&self, endpoint: &str, params: &[(&str, &str)]) -> Url {
		let mut url = Url::parse(&format!("{}/rest/{}", self.base_url.trim_end_matches('/'), endpoint)).unwrap();

		let salt = "musicbirb_salt";
		let token = format!("{:x}", md5::compute(format!("{}{}", self.pass_or_token, salt)));

		url.query_pairs_mut()
			.append_pair("u", &self.username)
			.append_pair("t", &token)
			.append_pair("s", salt)
			.append_pair("v", "1.16.1")
			.append_pair("c", "Musicbirb")
			.append_pair("f", "json");

		for (k, v) in params {
			url.query_pairs_mut().append_pair(k, v);
		}
		url
	}

	pub async fn get_rest_response(
		&self,
		endpoint: &str,
		params: &[(&str, &str)],
	) -> Result<dto::SubsonicResponse, MusicbirbError> {
		let url = self.build_rest_url(endpoint, params);
		let resp = self
			.client
			.get(url)
			.send()
			.await
			.map_err(|e| MusicbirbError::Network(e.to_string()))?;

		let root: dto::SubsonicResponseRoot = resp
			.json()
			.await
			.map_err(|e| MusicbirbError::Api(format!("Parse error: {}", e)))?;

		if root.subsonic_response.status == "failed" {
			let msg = root.subsonic_response.error.map(|e| e.message).unwrap_or_default();
			return Err(MusicbirbError::Api(format!("Subsonic API error: {}", msg)));
		}

		Ok(root.subsonic_response)
	}

	// Specifically for Navidrome rich extended endpoints
	pub async fn refresh_nd_jwt(&self) -> Result<(), MusicbirbError> {
		let req = navidrome_dto::NavidromeLoginRequest {
			username: &self.username,
			password: &self.pass_or_token,
		};
		let url = format!("{}/auth/login", self.base_url.trim_end_matches('/'));
		let resp = self
			.client
			.post(&url)
			.json(&req)
			.send()
			.await
			.map_err(|e| MusicbirbError::Network(e.to_string()))?;
		if resp.status().is_success() {
			let json = resp
				.json::<navidrome_dto::NavidromeLoginResponse>()
				.await
				.map_err(|e| MusicbirbError::Api(e.to_string()))?;
			*self.nd_jwt.write().unwrap() = Some(json.token);
			*self.nd_id.write().unwrap() = Some(json.id);
		}
		Ok(())
	}

	pub async fn get_nd_api<T: serde::de::DeserializeOwned>(
		&self,
		endpoint: &str,
		params: &[(&str, &str)],
	) -> Result<T, MusicbirbError> {
		let mut url = Url::parse(&format!("{}/api/{}", self.base_url.trim_end_matches('/'), endpoint)).unwrap();
		for (k, v) in params {
			url.query_pairs_mut().append_pair(k, v);
		}

		let mut token = self.nd_jwt.read().unwrap().clone().unwrap_or_default();
		let mut nd_id = self.nd_id.read().unwrap().clone().unwrap_or_default();

		let mut resp = self
			.client
			.get(url.clone())
			.header("x-nd-authorization", format!("Bearer {}", token))
			.header("x-nd-client-unique-id", nd_id.clone())
			.send()
			.await
			.map_err(|e| MusicbirbError::Network(e.to_string()))?;

		if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
			let _ = self.refresh_nd_jwt().await;
			token = self.nd_jwt.read().unwrap().clone().unwrap_or_default();
			nd_id = self.nd_id.read().unwrap().clone().unwrap_or_default();

			resp = self
				.client
				.get(url)
				.header("x-nd-authorization", format!("Bearer {}", token))
				.header("x-nd-client-unique-id", nd_id)
				.send()
				.await
				.map_err(|e| MusicbirbError::Network(e.to_string()))?;
		}

		if !resp.status().is_success() {
			return Err(MusicbirbError::Api(format!("Navidrome API error: {}", resp.status())));
		}

		resp.json()
			.await
			.map_err(|e| MusicbirbError::Api(format!("Navidrome JSON Error: {}", e)))
	}
}

pub struct SubsonicProvider {
	ctx: Arc<SubsonicContext>,
	query_client: Arc<QueryClient>,
}

impl SubsonicProvider {
	pub fn new(url: &str, username: &str, password: &str, server_type_str: &str) -> Result<Self, MusicbirbError> {
		let server_type = if server_type_str.to_lowercase() == "navidrome" {
			ServerType::Navidrome
		} else {
			ServerType::Subsonic
		};

		let client = reqwest::ClientBuilder::new()
			.build()
			.map_err(|e| MusicbirbError::Network(e.to_string()))?;

		Ok(Self {
			ctx: Arc::new(SubsonicContext {
				server_type,
				base_url: url.to_string(),
				username: username.to_string(),
				pass_or_token: password.to_string(),
				client,
				nd_jwt: RwLock::new(None),
				nd_id: RwLock::new(None),
			}),
			query_client: Arc::new(QueryClient::new()),
		})
	}
}

#[cfg(feature = "uniffi")]
#[uniffi::export]
pub fn create_subsonic_provider(
	url: String,
	username: String,
	password: String,
	server_type: String,
) -> Result<Arc<dyn Provider>, MusicbirbError> {
	let provider = SubsonicProvider::new(&url, &username, &password, &server_type)?;
	Ok(Arc::new(provider))
}

#[macros::async_ffi]
impl Provider for SubsonicProvider {
	async fn ping(&self) -> Result<(), MusicbirbError> {
		if self.ctx.server_type == ServerType::Navidrome {
			let _ = self.ctx.refresh_nd_jwt().await;
		}

		self.ctx.get_rest_response("ping", &[]).await?;
		Ok(())
	}

	fn media(&self) -> Arc<dyn MediaProvider> {
		Arc::new(SubsonicMedia {
			ctx: Arc::clone(&self.ctx),
		})
	}

	fn track(&self) -> Arc<CachedTrackProvider> {
		Arc::new(crate::providers::CachedTrackProvider::new(
			Arc::new(SubsonicTrack {
				ctx: Arc::clone(&self.ctx),
			}),
			Arc::clone(&self.query_client),
		))
	}

	fn album(&self) -> Arc<CachedAlbumProvider> {
		Arc::new(CachedAlbumProvider::new(
			Arc::new(SubsonicAlbum {
				ctx: Arc::clone(&self.ctx),
			}),
			Arc::clone(&self.query_client),
		))
	}

	fn artist(&self) -> Arc<CachedArtistProvider> {
		Arc::new(CachedArtistProvider::new(
			Arc::new(SubsonicArtist {
				ctx: Arc::clone(&self.ctx),
			}),
			Arc::clone(&self.query_client),
		))
	}

	fn playlist(&self) -> Arc<CachedPlaylistProvider> {
		Arc::new(CachedPlaylistProvider::new(
			Arc::new(SubsonicPlaylist {
				ctx: Arc::clone(&self.ctx),
			}),
			Arc::clone(&self.query_client),
		))
	}

	fn activity(&self) -> Arc<CachedActivityProvider> {
		Arc::new(CachedActivityProvider::new(
			Arc::new(SubsonicActivity {
				ctx: Arc::clone(&self.ctx),
			}),
			Arc::clone(&self.query_client),
		))
	}

	fn search(&self) -> Arc<CachedSearchProvider> {
		Arc::new(CachedSearchProvider::new(
			Arc::new(SubsonicSearch {
				ctx: Arc::clone(&self.ctx),
			}),
			Arc::clone(&self.query_client),
		))
	}
}
