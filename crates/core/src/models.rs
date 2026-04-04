#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrackId(pub String);

impl From<String> for TrackId {
	fn from(s: String) -> Self {
		TrackId(s)
	}
}

impl From<TrackId> for String {
	fn from(id: TrackId) -> Self {
		id.0
	}
}

#[cfg(feature = "ffi")]
uniffi::custom_type!(TrackId, String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlbumId(pub String);

impl From<String> for AlbumId {
	fn from(s: String) -> Self {
		AlbumId(s)
	}
}

impl From<AlbumId> for String {
	fn from(id: AlbumId) -> Self {
		id.0
	}
}

#[cfg(feature = "ffi")]
uniffi::custom_type!(AlbumId, String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtistId(pub String);

impl From<String> for ArtistId {
	fn from(s: String) -> Self {
		ArtistId(s)
	}
}

impl From<ArtistId> for String {
	fn from(id: ArtistId) -> Self {
		id.0
	}
}

#[cfg(feature = "ffi")]
uniffi::custom_type!(ArtistId, String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlaylistId(pub String);

impl From<String> for PlaylistId {
	fn from(s: String) -> Self {
		PlaylistId(s)
	}
}

impl From<PlaylistId> for String {
	fn from(id: PlaylistId) -> Self {
		id.0
	}
}

#[cfg(feature = "ffi")]
uniffi::custom_type!(PlaylistId, String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CoverArtId(pub String);

impl From<String> for CoverArtId {
	fn from(s: String) -> Self {
		CoverArtId(s)
	}
}

impl From<CoverArtId> for String {
	fn from(id: CoverArtId) -> Self {
		id.0
	}
}

#[cfg(feature = "ffi")]
uniffi::custom_type!(CoverArtId, String);

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayGain {
	pub track_gain: Option<f32>,
	pub track_peak: Option<f32>,
	pub album_gain: Option<f32>,
	pub album_peak: Option<f32>,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
	pub id: TrackId,
	pub title: String,
	pub artist: String,
	pub artist_id: Option<ArtistId>,
	pub album: String,
	pub album_id: Option<AlbumId>,
	pub duration_secs: u32,
	pub cover_art: Option<CoverArtId>,
	pub track_num: Option<u32>,
	pub disc_num: Option<u32>,
	pub year: Option<u32>,
	pub genre: Option<String>,
	pub play_count: Option<u64>,
	pub bit_rate: Option<u32>,
	pub size: Option<u64>,
	pub created_timestamp: Option<i64>,
	pub starred_timestamp: Option<i64>,
	pub content_type: Option<String>,
	pub suffix: Option<String>,
	pub starred: Option<bool>,
	pub user_rating: Option<u8>,
	//
	pub musicbrainz_id: Option<String>,
	pub lastfm_url: Option<String>,
	pub replay_gain: Option<ReplayGain>,
	pub bpm: Option<u32>,
	pub comment: Option<String>,
	pub sort_name: Option<String>,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq)]
pub struct Album {
	pub id: AlbumId,
	pub title: String,
	pub artist: String,
	pub artist_id: Option<ArtistId>,
	pub year: Option<u32>,
	pub cover_art: Option<CoverArtId>,
	pub duration_secs: Option<u32>,
	pub play_count: Option<u64>,
	pub created_timestamp: Option<i64>,
	pub starred_timestamp: Option<i64>,
	pub song_count: Option<u32>,
	pub starred: Option<bool>,
	pub user_rating: Option<u8>,
	pub release_type: Option<String>,
	//
	pub musicbrainz_id: Option<String>,
	pub lastfm_url: Option<String>,
	pub genre: Option<String>,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq)]
pub struct AlbumDetails {
	pub id: AlbumId,
	pub title: String,
	pub artist: String,
	pub artist_id: Option<ArtistId>,
	pub cover_art: Option<CoverArtId>,
	pub song_count: u32,
	pub duration_secs: u32,
	pub year: Option<u32>,
	pub genre: Option<String>,
	pub play_count: Option<u64>,
	pub created_timestamp: Option<i64>,
	pub starred_timestamp: Option<i64>,
	pub songs: Vec<Track>,
	pub starred: Option<bool>,
	pub user_rating: Option<u8>,
	pub release_type: Option<String>,
	pub musicbrainz_id: Option<String>,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq)]
pub struct Artist {
	pub id: ArtistId,
	pub name: String,
	pub cover_art: Option<CoverArtId>,
	pub artist_image_url: Option<String>,
	pub starred: Option<bool>,
	pub musicbrainz_id: Option<String>,
	pub lastfm_url: Option<String>,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq)]
pub struct ArtistDetails {
	pub id: ArtistId,
	pub name: String,
	pub cover_art: Option<CoverArtId>,
	pub album_count: u32,
	pub song_count: u32,
	pub albums: Vec<Album>,
	pub biography: Option<String>,
	pub similar_artists: Vec<Artist>,
	pub top_songs: Vec<Track>,
	pub starred: Option<bool>,
	pub musicbrainz_id: Option<String>,
	pub lastfm_url: Option<String>,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq)]
pub struct Playlist {
	pub id: PlaylistId,
	pub name: String,
	pub song_count: u32,
	pub duration_secs: u32,
	pub cover_art: Option<CoverArtId>,
	pub owner: Option<String>,
	pub public: Option<bool>,
	pub created_timestamp: i64,
	pub changed_timestamp: i64,
	pub comment: Option<String>,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistDetails {
	pub id: PlaylistId,
	pub name: String,
	pub song_count: u32,
	pub duration_secs: u32,
	pub cover_art: Option<CoverArtId>,
	pub owner: Option<String>,
	pub public: Option<bool>,
	pub created_timestamp: i64,
	pub changed_timestamp: i64,
	pub comment: Option<String>,
	pub songs: Vec<Track>,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackScrobble {
	pub id: TrackId,
	pub timestamp: u64,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
#[derive(Clone, Debug, PartialEq)]
pub enum SearchPreset {
	LastPlayedAlbums,
	RecentlyAddedAlbums,
	NewlyReleasedAlbums,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Clone, Debug, PartialEq)]
pub struct SearchQuery {
	pub keyword: Option<String>,
	pub preset: Option<SearchPreset>,
	pub limit: Option<i32>,
	pub offset: Option<i32>,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Clone, Debug, PartialEq)]
pub struct SearchResults {
	pub tracks: Vec<Track>,
	pub albums: Vec<Album>,
	pub artists: Vec<Artist>,
}
