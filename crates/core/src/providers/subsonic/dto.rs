use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SubsonicResponseRoot {
	#[serde(rename = "subsonic-response")]
	pub subsonic_response: SubsonicResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsonicResponse {
	pub status: String,
	pub version: String,
	pub error: Option<SubsonicError>,
	pub album_list2: Option<AlbumList2>,
	pub artist: Option<Artist>,
	pub artist_info2: Option<ArtistInfo2>,
	pub album: Option<Album>,
	pub song: Option<Child>,
	pub playlists: Option<Playlists>,
	pub playlist: Option<PlaylistWithSongs>,
	pub search_result2: Option<SearchResult>,
	pub search_result3: Option<SearchResult>,
	pub top_songs: Option<TopSongs>,
	pub similar_songs: Option<SimilarSongs>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsonicError {
	pub code: i32,
	pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumList2 {
	#[serde(default)]
	pub album: Vec<Child>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
	pub id: String,
	pub name: String,
	pub cover_art: Option<String>,
	pub album_count: Option<i32>,
	pub starred: Option<String>,
	pub music_brainz_id: Option<String>,
	pub last_fm_url: Option<String>,
	#[serde(default)]
	pub album: Vec<Child>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistInfo2 {
	pub biography: Option<String>,
	pub music_brainz_id: Option<String>,
	pub last_fm_url: Option<String>,
	pub small_image_url: Option<String>,
	pub medium_image_url: Option<String>,
	pub large_image_url: Option<String>,
	#[serde(default)]
	pub similar_artist: Vec<Artist>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
	pub id: String,
	pub name: String,
	pub artist: Option<String>,
	pub artist_id: Option<String>,
	pub cover_art: Option<String>,
	pub song_count: Option<i32>,
	pub duration: Option<i32>,
	pub created: DateTime<Utc>,
	pub year: Option<i32>,
	pub genre: Option<String>,
	pub play_count: Option<i64>,
	pub user_rating: Option<u8>,
	pub starred: Option<String>,
	pub music_brainz_id: Option<String>,
	#[serde(default)]
	pub song: Vec<Child>,
	pub is_compilation: Option<bool>,
	pub release_types: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Child {
	pub id: String,
	pub parent: Option<String>,
	pub is_dir: Option<bool>,
	pub title: Option<String>,
	pub name: Option<String>,
	pub album: Option<String>,
	pub artist: Option<String>,
	pub track: Option<i32>,
	pub disc_number: Option<i32>,
	pub year: Option<i32>,
	pub genre: Option<String>,
	pub cover_art: Option<String>,
	pub size: Option<i64>,
	pub content_type: Option<String>,
	pub suffix: Option<String>,
	pub duration: Option<i32>,
	pub bit_rate: Option<i32>,
	pub path: Option<String>,
	pub user_rating: Option<u8>,
	pub starred: Option<String>,
	pub album_id: Option<String>,
	pub artist_id: Option<String>,
	pub play_count: Option<i64>,
	pub created: DateTime<Utc>,
	pub bpm: Option<u32>,
	pub comment: Option<String>,
	pub sort_name: Option<String>,
	pub media_type: Option<String>,
	pub music_brainz_id: Option<String>,
	pub channel_count: Option<u32>,
	pub replay_gain: Option<ReplayGainDto>,
	// OpenSubsonic extensions
	pub is_compilation: Option<bool>,
	pub release_types: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReplayGainDto {
	pub track_gain: Option<f32>,
	pub album_gain: Option<f32>,
	pub track_peak: Option<f32>,
	pub album_peak: Option<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlists {
	#[serde(default)]
	pub playlist: Vec<Playlist>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
	pub id: String,
	pub name: String,
	pub comment: Option<String>,
	pub owner: Option<String>,
	pub public: Option<bool>,
	pub song_count: Option<i32>,
	pub duration: Option<i32>,
	pub created: DateTime<Utc>,
	pub changed: DateTime<Utc>,
	pub cover_art: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistWithSongs {
	#[serde(flatten)]
	pub base: Playlist,
	#[serde(default)]
	pub entry: Vec<Child>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
	#[serde(default)]
	pub artist: Vec<Artist>,
	#[serde(default)]
	pub album: Vec<Child>,
	#[serde(default)]
	pub song: Vec<Child>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopSongs {
	#[serde(default)]
	pub song: Vec<Child>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarSongs {
	#[serde(default)]
	pub song: Vec<Child>,
}
