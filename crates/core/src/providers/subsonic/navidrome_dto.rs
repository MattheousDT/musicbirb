use crate::models::{ReleaseSubtype, ReleaseType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct NavidromeLoginRequest<'a> {
	pub username: &'a str,
	pub password: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavidromeLoginResponse {
	pub id: String,
	pub token: String,
	pub username: String,
	pub is_admin: Option<bool>,
	pub last_fm_api_key: Option<String>,
	pub subsonic_salt: Option<String>,
	pub subsonic_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NavidromeGenre {
	pub id: String,
	pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavidromeArtist {
	pub id: String,
	pub name: String,
	pub album_count: Option<i32>,
	pub song_count: Option<i32>,
	pub genres: Option<Vec<NavidromeGenre>>,
	pub full_text: Option<String>,
	pub order_artist_name: Option<String>,
	pub size: Option<i64>,
	pub mbz_artist_id: Option<String>,
	pub small_image_url: Option<String>,
	pub medium_image_url: Option<String>,
	pub large_image_url: Option<String>,
	pub external_url: Option<String>,
	pub play_count: Option<i64>,
	pub starred: Option<bool>,
	pub starred_at: Option<DateTime<Utc>>,
	pub rating: Option<u8>,
	pub play_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NavidromeAlbum {
	pub id: String,
	pub name: String,
	pub artist_id: Option<String>,
	pub artist: Option<String>,
	pub album_artist_id: Option<String>,
	pub album_artist: Option<String>,
	pub all_artist_ids: Option<String>,
	pub song_count: Option<i32>,
	pub duration: Option<f64>,
	pub size: Option<i64>,
	pub play_count: Option<i64>,
	pub starred: Option<bool>,
	pub starred_at: Option<DateTime<Utc>>,
	pub min_year: Option<i32>,
	pub max_year: Option<i32>,
	pub date: Option<String>,
	pub genre: Option<String>,
	pub genres: Option<Vec<NavidromeGenre>>,
	pub releases: Option<i32>,
	pub compilation: Option<bool>,
	pub embed_art_path: Option<String>,
	pub rating: Option<u8>,
	pub play_date: Option<DateTime<Utc>>,
	pub full_text: Option<String>,
	pub order_album_name: Option<String>,
	pub order_album_artist_name: Option<String>,
	pub type_: Option<String>,
	pub tags: Option<HashMap<String, Vec<String>>>,
}

impl NavidromeAlbum {
	pub fn get_release_type(&self) -> ReleaseType {
		// Priority 1: Check the tags field (preferred for Navidrome)
		if let Some(tags) = &self.tags {
			if let Some(types) = tags.get("releasetype") {
				if types.iter().any(|t| t.to_lowercase() == "single") {
					return ReleaseType::Single;
				}
				if types.iter().any(|t| t.to_lowercase() == "ep") {
					return ReleaseType::Ep;
				}
				if types.iter().any(|t| t.to_lowercase() == "album") {
					return ReleaseType::Album;
				}
			}
		}

		// Priority 2: Fallback to the type_ field
		match self.type_.as_deref().map(|s| s.to_lowercase()).as_deref() {
			Some("ep") => ReleaseType::Ep,
			Some("single") => ReleaseType::Single,
			Some("album") => ReleaseType::Album,
			_ => ReleaseType::Album,
		}
	}

	pub fn get_release_subtype(&self) -> ReleaseSubtype {
		if let Some(tags) = &self.tags {
			if let Some(types) = tags.get("releasetype") {
				if types.iter().any(|t| t.to_lowercase() == "soundtrack") {
					return ReleaseSubtype::Soundtrack;
				}
				if types.iter().any(|t| t.to_lowercase() == "live") {
					return ReleaseSubtype::Live;
				}
				if types.iter().any(|t| t.to_lowercase() == "compilation") {
					return ReleaseSubtype::Compilation;
				}
				if types.iter().any(|t| t.to_lowercase() == "remix") {
					return ReleaseSubtype::Remix;
				}
				if types.iter().any(|t| t.to_lowercase() == "demo") {
					return ReleaseSubtype::Demo;
				}
				if types.iter().any(|t| t.to_lowercase() == "broadcast") {
					return ReleaseSubtype::Broadcast;
				}
			}
		}

		if self.compilation.unwrap_or(false) {
			return ReleaseSubtype::Compilation;
		}

		ReleaseSubtype::None
	}
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavidromeSong {
	pub id: String,
	pub path: String,
	pub title: String,
	pub album: String,
	pub album_id: String,
	pub artist: String,
	pub artist_id: String,
	pub album_artist: Option<String>,
	pub album_artist_id: Option<String>,
	pub track_number: Option<u32>,
	pub disc_number: Option<u32>,
	pub year: Option<u32>,
	pub date: Option<String>,
	pub size: Option<i64>,
	pub suffix: Option<String>,
	pub duration: Option<f64>,
	pub bit_rate: Option<u32>,
	pub channels: Option<u32>,
	pub genre: Option<String>,
	pub genres: Option<Vec<NavidromeGenre>>,
	pub play_count: Option<u64>,
	pub play_date: Option<DateTime<Utc>>,
	pub rating: Option<u8>,
	pub starred: Option<bool>,
	pub starred_at: Option<DateTime<Utc>>,
	pub bookmark_position: Option<u64>,
	pub has_cover_art: Option<bool>,
	pub compilation: Option<bool>,
	pub lyrics: Option<String>,
	pub full_text: Option<String>,
	pub order_title: Option<String>,
	pub order_album_name: Option<String>,
	pub order_artist_name: Option<String>,
	pub rg_track_gain: Option<f32>,
	pub rg_track_peak: Option<f32>,
	pub rg_album_gain: Option<f32>,
	pub rg_album_peak: Option<f32>,
}
