use super::dto;
use super::navidrome_dto;
use crate::PlaylistDetails;
use crate::models::{
	Album, AlbumDetails, AlbumId, Artist, ArtistId, CoverArtId, Playlist, PlaylistId, ReleaseSubtype, ReleaseType,
	ReplayGain, Track, TrackId,
};

impl From<dto::Child> for Track {
	fn from(c: dto::Child) -> Self {
		Self {
			id: TrackId(c.id),
			title: c.title.unwrap_or_default(),
			artist: c.artist.unwrap_or_default(),
			artist_id: c.artist_id.map(ArtistId),
			album: c.album.unwrap_or_default(),
			album_id: c.album_id.map(AlbumId),
			duration_secs: c.duration.unwrap_or(0) as u32,
			cover_art: c.cover_art.map(CoverArtId),
			track_num: c.track.map(|t| t as u32),
			disc_num: c.disc_number.map(|d| d as u32),
			year: c.year.map(|y| y as u32),
			genre: c.genre,
			play_count: c.play_count.map(|c| c as u64),
			bit_rate: c.bit_rate.map(|b| b as u32),
			size: c.size.map(|s| s as u64),
			created_timestamp: Some(c.created.timestamp()),
			starred_timestamp: None,
			content_type: c.content_type,
			suffix: c.suffix,
			starred: c.starred.map(|_| true),
			user_rating: c.user_rating,
			musicbrainz_id: c.music_brainz_id,
			lastfm_url: None,
			replay_gain: c.replay_gain.map(|rg| ReplayGain {
				track_gain: rg.track_gain,
				track_peak: rg.track_peak,
				album_gain: rg.album_gain,
				album_peak: rg.album_peak,
			}),
			bpm: c.bpm,
			comment: c.comment,
			sort_name: c.sort_name,
		}
	}
}

impl From<dto::Child> for Album {
	fn from(c: dto::Child) -> Self {
		let release_type = if let Some(types) = &c.release_types {
			if types.iter().any(|t| t.to_lowercase() == "single") {
				ReleaseType::Single
			} else if types.iter().any(|t| t.to_lowercase() == "ep") {
				ReleaseType::Ep
			} else {
				ReleaseType::Album
			}
		} else {
			ReleaseType::Album
		};

		let release_subtype = if c.is_compilation.unwrap_or(false) {
			ReleaseSubtype::Compilation
		} else {
			ReleaseSubtype::None
		};

		Self {
			id: AlbumId(c.id),
			title: c.name.unwrap_or_default(),
			artist: c.artist.unwrap_or_default(),
			artist_id: c.artist_id.map(ArtistId),
			year: c.year.map(|y| y as u32),
			cover_art: c.cover_art.map(CoverArtId),
			duration_secs: c.duration.map(|d| d as u32),
			play_count: c.play_count.map(|c| c as u64),
			created_timestamp: Some(c.created.timestamp()),
			starred_timestamp: None,
			song_count: None,
			starred: c.starred.map(|_| true),
			user_rating: c.user_rating,
			release_type,
			release_subtype,
			musicbrainz_id: None,
			lastfm_url: None,
			genre: c.genre,
		}
	}
}

impl From<dto::Album> for AlbumDetails {
	fn from(a: dto::Album) -> Self {
		let release_type = if let Some(types) = &a.release_types {
			if types.iter().any(|t| t.to_lowercase() == "single") {
				ReleaseType::Single
			} else if types.iter().any(|t| t.to_lowercase() == "ep") {
				ReleaseType::Ep
			} else {
				ReleaseType::Album
			}
		} else {
			ReleaseType::Album
		};

		let release_subtype = if a.is_compilation.unwrap_or(false) {
			ReleaseSubtype::Compilation
		} else {
			ReleaseSubtype::None
		};

		Self {
			id: AlbumId(a.id),
			title: a.name,
			artist: a.artist.unwrap_or_default(),
			artist_id: a.artist_id.map(ArtistId),
			cover_art: a.cover_art.map(CoverArtId),
			song_count: a.song_count.unwrap_or(0) as u32,
			duration_secs: a.duration.unwrap_or(0) as u32,
			year: a.year.map(|y| y as u32),
			genre: a.genre,
			play_count: a.play_count.map(|c| c as u64),
			created_timestamp: Some(a.created.timestamp()),
			starred_timestamp: None,
			songs: a.song.into_iter().map(Track::from).collect(),
			starred: a.starred.map(|_| true),
			user_rating: a.user_rating,
			release_type,
			release_subtype,
			musicbrainz_id: a.music_brainz_id,
		}
	}
}

impl From<dto::Artist> for Artist {
	fn from(a: dto::Artist) -> Self {
		Self {
			id: ArtistId(a.id),
			name: a.name,
			cover_art: a.cover_art.map(CoverArtId),
			artist_image_url: None,
			starred: a.starred.map(|_| true),
			musicbrainz_id: a.music_brainz_id,
			lastfm_url: a.last_fm_url,
		}
	}
}

impl From<dto::Playlist> for Playlist {
	fn from(pl: dto::Playlist) -> Self {
		Self {
			id: PlaylistId(pl.id),
			name: pl.name,
			song_count: pl.song_count.unwrap_or(0) as u32,
			duration_secs: pl.duration.unwrap_or(0) as u32,
			cover_art: pl.cover_art.map(CoverArtId),
			owner: pl.owner,
			public: pl.public,
			created_timestamp: pl.created.timestamp(),
			changed_timestamp: pl.changed.timestamp(),
			comment: pl.comment,
		}
	}
}

impl From<dto::PlaylistWithSongs> for PlaylistDetails {
	fn from(pl: dto::PlaylistWithSongs) -> Self {
		Self {
			id: PlaylistId(pl.base.id),
			name: pl.base.name,
			song_count: pl.base.song_count.unwrap_or(0) as u32,
			duration_secs: pl.base.duration.unwrap_or(0) as u32,
			cover_art: pl.base.cover_art.map(CoverArtId),
			owner: pl.base.owner,
			public: pl.base.public,
			created_timestamp: pl.base.created.timestamp(),
			changed_timestamp: pl.base.changed.timestamp(),
			comment: pl.base.comment,
			songs: pl.entry.into_iter().map(Track::from).collect(),
		}
	}
}

impl From<navidrome_dto::NavidromeAlbum> for Album {
	fn from(a: navidrome_dto::NavidromeAlbum) -> Self {
		let release_type = a.get_release_type();
		let release_subtype = a.get_release_subtype();
		Self {
			id: AlbumId(a.id.clone()),
			title: a.name,
			artist: a.artist.unwrap_or_default(),
			artist_id: a.artist_id.map(ArtistId),
			year: a.min_year.map(|y| y as u32),
			cover_art: Some(CoverArtId(a.id)),
			duration_secs: a.duration.map(|d| d as u32),
			play_count: a.play_count.map(|c| c as u64),
			created_timestamp: None,
			starred_timestamp: a.starred_at.map(|s| s.timestamp()),
			song_count: a.song_count.map(|c| c as u32),
			starred: a.starred,
			user_rating: a.rating,
			release_type,
			release_subtype,
			musicbrainz_id: None,
			lastfm_url: None,
			genre: a.genre,
		}
	}
}

impl From<navidrome_dto::NavidromeSong> for Track {
	fn from(s: navidrome_dto::NavidromeSong) -> Self {
		let id = s.id.clone();
		Self {
			id: TrackId(id.clone()),
			title: s.title,
			artist: s.artist,
			artist_id: Some(ArtistId(s.artist_id)),
			album: s.album,
			album_id: Some(AlbumId(s.album_id)),
			duration_secs: s.duration.unwrap_or(0.0) as u32,
			cover_art: if s.has_cover_art.unwrap_or(false) {
				Some(CoverArtId(id))
			} else {
				None
			},
			track_num: s.track_number,
			disc_num: s.disc_number,
			year: s.year,
			genre: s.genre,
			play_count: s.play_count,
			bit_rate: s.bit_rate,
			size: s.size.map(|s| s as u64),
			created_timestamp: None,
			starred_timestamp: s.starred_at.map(|s| s.timestamp()),
			content_type: None,
			suffix: s.suffix,
			starred: s.starred,
			user_rating: s.rating,
			musicbrainz_id: None,
			lastfm_url: None,
			replay_gain: Some(ReplayGain {
				track_gain: s.rg_track_gain,
				track_peak: s.rg_track_peak,
				album_gain: s.rg_album_gain,
				album_peak: s.rg_album_peak,
			}),
			bpm: None,
			comment: None,
			sort_name: None,
		}
	}
}
