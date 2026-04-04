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
		let url = self.ctx.build_rest_url("stream", &[("id", &track_id.0)]);
		Ok(url.to_string())
	}

	fn get_cover_art_url(&self, cover_id: &CoverArtId, size: Option<u32>) -> Result<String, MusicbirbError> {
		let mut params = vec![("id", cover_id.0.as_str())];
		let size_str = size.map(|s| s.to_string());

		if let Some(s) = &size_str {
			params.push(("size", s.as_str()));
		}

		let url = self.ctx.build_rest_url("getCoverArt", &params);
		Ok(url.to_string())
	}

	async fn get_cover_art_bytes(&self, cover_id: &CoverArtId) -> Result<Vec<u8>, MusicbirbError> {
		let url = self.get_cover_art_url(cover_id, Some(600))?;

		let resp = self
			.ctx
			.client
			.get(url)
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
