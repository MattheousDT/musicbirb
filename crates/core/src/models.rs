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

impl From<submarine::data::Child> for Track {
	fn from(song: submarine::data::Child) -> Self {
		Self {
			id: TrackId(song.id),
			title: song.title,
			artist: song.artist.unwrap_or_else(|| "Unknown".to_string()),
			artist_id: song.artist_id.map(ArtistId),
			album: song.album.unwrap_or_else(|| "Unknown".to_string()),
			album_id: song.album_id.map(AlbumId),
			duration_secs: song.duration.unwrap_or(0) as u32,
			cover_art: song.cover_art.map(CoverArtId),
			track_num: song.track.map(|t| t as u32),
			disc_num: song.disc_number.map(|d| d as u32),
			year: song.year.map(|y| y as u32),
			genre: song.genre,
			play_count: song.play_count.map(|c| c as u64),
			bit_rate: song.bit_rate.map(|b| b as u32),
			size: song.size.map(|s| s as u64),
			created_timestamp: song.created.map(|d| d.timestamp()),
			starred_timestamp: song.starred.map(|d| d.timestamp()),
			content_type: song.content_type,
			suffix: song.suffix,
		}
	}
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

impl From<submarine::data::Child> for Album {
	fn from(item: submarine::data::Child) -> Self {
		Self {
			id: AlbumId(item.id),
			title: item.name,
			artist: item.artist.unwrap_or_else(|| "Unknown".to_string()),
			artist_id: item.artist_id.map(ArtistId),
			year: item.year.map(|y| y as u32),
			cover_art: item.cover_art.map(CoverArtId),
			duration_secs: item.duration.map(|d| d as u32),
			play_count: item.play_count.map(|c| c as u64),
			created_timestamp: item.created.map(|d| d.timestamp()),
			starred_timestamp: item.starred.map(|d| d.timestamp()),
			song_count: None,
		}
	}
}

impl From<submarine::data::AlbumId3> for Album {
	fn from(item: submarine::data::AlbumId3) -> Self {
		Self {
			id: AlbumId(item.id),
			title: item.name,
			artist: item.artist.unwrap_or_else(|| "Unknown".to_string()),
			artist_id: item.artist_id.map(ArtistId),
			year: item.year.map(|y| y as u32),
			cover_art: item.cover_art.map(CoverArtId),
			duration_secs: Some(item.duration as u32),
			play_count: item.play_count.map(|c| c as u64),
			created_timestamp: Some(item.created.timestamp()),
			starred_timestamp: item.starred.map(|d| d.timestamp()),
			song_count: Some(item.song_count as u32),
		}
	}
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

impl From<submarine::data::ArtistId3> for Artist {
	fn from(a: submarine::data::ArtistId3) -> Self {
		Self {
			id: ArtistId(a.id),
			name: a.name,
			cover_art: a.cover_art.map(CoverArtId),
			artist_image_url: a.artist_image_url,
		}
	}
}

impl From<submarine::data::Artist> for Artist {
	fn from(a: submarine::data::Artist) -> Self {
		Self {
			id: ArtistId(a.id),
			name: a.name,
			cover_art: None,
			artist_image_url: a.artist_image_url,
		}
	}
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

impl From<submarine::data::Playlist> for Playlist {
	fn from(pl: submarine::data::Playlist) -> Self {
		Self {
			id: PlaylistId(pl.id),
			name: pl.name,
			song_count: pl.song_count as u32,
			duration_secs: pl.duration as u32,
			cover_art: pl.cover_art.map(CoverArtId),
			owner: pl.owner,
			public: pl.public,
			created_timestamp: pl.created.timestamp(),
			changed_timestamp: pl.changed.timestamp(),
		}
	}
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
