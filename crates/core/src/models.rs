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
