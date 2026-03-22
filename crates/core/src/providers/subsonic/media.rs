use super::SubsonicContext;
use crate::error::MusicbirbError;
use crate::models::{CoverArtId, TrackId};
use crate::providers::MediaProvider;
use reqwest::StatusCode;
use std::sync::Arc;

pub struct SubsonicMedia {
	pub ctx: Arc<SubsonicContext>,
}

#[macros::async_ffi]
impl MediaProvider for SubsonicMedia {
	async fn get_stream_url(&self, track_id: &TrackId) -> Result<String, MusicbirbError> {
		let url = self
			.ctx
			.client
			.stream_url(&track_id.0, None, None::<String>, None, None::<String>, None, None)
			.map_err(|e| MusicbirbError::Api(format!("Failed to build stream URL: {}", e)))?;
		Ok(url.to_string())
	}

	fn get_cover_art_url(&self, cover_id: &CoverArtId, size: Option<u32>) -> Result<String, MusicbirbError> {
		let url = self
			.ctx
			.client
			.get_cover_art_url(&cover_id.0, size.map(|s| s as i32))
			.map_err(|e| MusicbirbError::Api(e.to_string()))?;
		Ok(url.to_string())
	}

	async fn get_cover_art_bytes(&self, cover_id: &CoverArtId) -> Result<Vec<u8>, MusicbirbError> {
		let url = self
			.ctx
			.client
			.get_cover_art_url(&cover_id.0, Some(600))
			.map_err(|e| MusicbirbError::Api(e.to_string()))?;

		let resp = self
			.ctx
			.http_client
			.get(url.clone())
			.send()
			.await
			.map_err(|e| MusicbirbError::Network(e.to_string()))?;

		if resp.status() != StatusCode::OK {
			return Err(MusicbirbError::Api(format!("Image download failed: {}", resp.status())));
		}

		let bytes = resp.bytes().await.map_err(|e| MusicbirbError::Network(e.to_string()))?;
		Ok(bytes.to_vec())
	}
}
