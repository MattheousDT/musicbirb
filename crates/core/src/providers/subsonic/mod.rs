use crate::MusicbirbError;
use crate::providers::*;
use async_trait::async_trait;
use std::sync::Arc;
use submarine::{Client, auth::AuthBuilder};

use activity::*;
use album::*;
use artist::*;
use media::*;
use playlist::*;
use search::*;
use track::*;

pub mod models;

pub mod activity;
pub mod album;
pub mod artist;
pub mod media;
pub mod playlist;
pub mod search;
pub mod track;

pub struct SubsonicContext {
	pub client: Client,
	pub http_client: reqwest::Client,
	pub username: String,
}

pub struct SubsonicProvider {
	ctx: Arc<SubsonicContext>,
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
			ctx: Arc::new(SubsonicContext {
				client,
				http_client,
				username: username.to_string(),
			}),
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
	async fn ping(&self) -> Result<(), MusicbirbError> {
		self.ctx
			.client
			.get_user(self.ctx.username.clone())
			.await
			.map_err(|e| MusicbirbError::Api(format!("Validation failed: {}", e)))?;
		Ok(())
	}

	fn media(&self) -> Arc<dyn MediaProvider> {
		Arc::new(SubsonicMedia {
			ctx: Arc::clone(&self.ctx),
		})
	}

	fn track(&self) -> Arc<dyn TrackProvider> {
		Arc::new(SubsonicTrack {
			ctx: Arc::clone(&self.ctx),
		})
	}

	fn album(&self) -> Arc<dyn AlbumProvider> {
		Arc::new(SubsonicAlbum {
			ctx: Arc::clone(&self.ctx),
		})
	}

	fn artist(&self) -> Arc<dyn ArtistProvider> {
		Arc::new(SubsonicArtist {
			ctx: Arc::clone(&self.ctx),
		})
	}

	fn playlist(&self) -> Arc<dyn PlaylistProvider> {
		Arc::new(SubsonicPlaylist {
			ctx: Arc::clone(&self.ctx),
		})
	}

	fn activity(&self) -> Arc<dyn ActivityProvider> {
		Arc::new(SubsonicActivity {
			ctx: Arc::clone(&self.ctx),
		})
	}

	fn search(&self) -> Arc<dyn SearchProvider> {
		Arc::new(SubsonicSearch {
			ctx: Arc::clone(&self.ctx),
		})
	}
}
