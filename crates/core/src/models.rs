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
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artist {
	pub id: ArtistId,
	pub name: String,
	pub cover_art: Option<CoverArtId>,
	pub artist_image_url: Option<String>,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtistDetails {
	pub id: ArtistId,
	pub name: String,
	pub cover_art: Option<CoverArtId>,
	pub album_count: u32,
	pub albums: Vec<Album>,
	pub biography: Option<String>,
	pub similar_artists: Vec<Artist>,
	pub top_songs: Vec<Track>,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
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
