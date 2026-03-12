#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrackId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlbumId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlaylistId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CoverArtId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
	pub id: TrackId,
	pub title: String,
	pub artist: String,
	pub album: String,
	pub duration_secs: u32,
	pub cover_art: Option<CoverArtId>,
}

impl From<submarine::data::Child> for Track {
	fn from(song: submarine::data::Child) -> Self {
		Self {
			id: TrackId(song.id),
			title: song.title,
			artist: song.artist.unwrap_or_else(|| "Unknown".to_string()),
			album: song.album.unwrap_or_else(|| "Unknown".to_string()),
			duration_secs: song.duration.unwrap_or(0) as u32,
			cover_art: song.cover_art.map(CoverArtId),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Album {
	pub id: AlbumId,
	pub title: String,
	pub artist: String,
	pub cover_art: Option<CoverArtId>,
}

impl From<submarine::data::Child> for Album {
	fn from(item: submarine::data::Child) -> Self {
		Self {
			id: AlbumId(item.id),
			title: item.name,
			artist: item.artist.unwrap_or_else(|| "Unknown".to_string()),
			cover_art: item.cover_art.map(CoverArtId),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Playlist {
	pub id: PlaylistId,
	pub name: String,
	pub song_count: u32,
	pub duration_secs: u32,
	pub cover_art: Option<CoverArtId>,
}

impl From<submarine::data::Playlist> for Playlist {
	fn from(pl: submarine::data::Playlist) -> Self {
		Self {
			id: PlaylistId(pl.id.clone()),
			name: pl.name,
			song_count: pl.song_count as u32,
			duration_secs: pl.duration as u32,
			cover_art: pl.cover_art.map(CoverArtId),
		}
	}
}
