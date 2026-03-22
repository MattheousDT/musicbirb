use super::JellyfinContext;
use crate::error::MusicbirbError;
use crate::models::{CoverArtId, TrackId};
use crate::providers::MediaProvider;
use std::sync::Arc;

pub struct JellyfinMedia {
	pub ctx: Arc<JellyfinContext>,
}

#[macros::async_ffi]
impl MediaProvider for JellyfinMedia {
	async fn get_stream_url(&self, track_id: &TrackId) -> Result<String, MusicbirbError> {
		let mut url = format!("{}/Audio/{}/stream?static=true", self.ctx.base_url, track_id.0);
		if let Some(token) = &self.ctx.token {
			url.push_str(&format!("&api_key={}", token));
		}
		Ok(url)
	}

	fn get_cover_art_url(&self, cover_id: &CoverArtId, size: Option<u32>) -> Result<String, MusicbirbError> {
		let mut url = format!("{}/Items/{}/Images/Primary", self.ctx.base_url, cover_id.0);
		if let Some(s) = size {
			url.push_str(&format!("?maxWidth={}", s));
		}
		Ok(url)
	}

	async fn get_cover_art_bytes(&self, cover_id: &CoverArtId) -> Result<Vec<u8>, MusicbirbError> {
		let url = self.get_cover_art_url(cover_id, Some(600))?;
		let mut req = self.ctx.http.get(&url);
		if self.ctx.token.is_some() {
			req = req.header("X-Emby-Authorization", self.ctx.auth_header());
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
}
