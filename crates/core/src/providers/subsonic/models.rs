use crate::{
	Playlist,
	models::{
		Album, AlbumDetails, AlbumId, Artist, ArtistId, CoverArtId, PlaylistDetails, PlaylistId,
		Track, TrackId,
	},
};

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

impl From<submarine::data::AlbumWithSongsId3> for AlbumDetails {
	fn from(a: submarine::data::AlbumWithSongsId3) -> Self {
		Self {
			id: AlbumId(a.base.id),
			title: a.base.name,
			artist: a.base.artist.unwrap_or_else(|| "Unknown".to_string()),
			artist_id: a.base.artist_id.map(ArtistId),
			cover_art: a.base.cover_art.map(CoverArtId),
			song_count: a.base.song_count as u32,
			duration_secs: a.base.duration as u32,
			year: a.base.year.map(|y| y as u32),
			genre: a.base.genre,
			play_count: a.base.play_count.map(|c| c as u64),
			created_timestamp: Some(a.base.created.timestamp()),
			starred_timestamp: a.base.starred.map(|d| d.timestamp()),
			songs: a.song.into_iter().map(Track::from).collect(),
		}
	}
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

impl From<submarine::data::PlaylistWithSongs> for PlaylistDetails {
	fn from(pl: submarine::data::PlaylistWithSongs) -> Self {
		Self {
			id: PlaylistId(pl.base.id.clone()),
			name: pl.base.name.clone(),
			song_count: pl.base.song_count as u32,
			duration_secs: pl.base.duration as u32,
			cover_art: pl.base.cover_art.clone().map(CoverArtId),
			owner: pl.base.owner.clone(),
			public: pl.base.public,
			created_timestamp: pl.base.created.timestamp(),
			changed_timestamp: pl.base.changed.timestamp(),
			comment: pl.base.comment.clone(),
			songs: pl.entry.into_iter().map(Track::from).collect(),
		}
	}
}
