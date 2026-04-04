use super::{dto, navidrome_dto};
use crate::models::{
	Album, AlbumDetails, AlbumId, Artist, ArtistId, CoverArtId, Playlist, PlaylistDetails, PlaylistId, ReplayGain,
	Track, TrackId,
};

fn parse_is_starred(s: &Option<String>) -> Option<bool> {
	Some(s.is_some())
}

impl From<dto::ReplayGainDto> for ReplayGain {
	fn from(d: dto::ReplayGainDto) -> Self {
		Self {
			track_gain: d.track_gain,
			track_peak: d.track_peak,
			album_gain: d.album_gain,
			album_peak: d.album_peak,
		}
	}
}

impl From<dto::Child> for Track {
	fn from(c: dto::Child) -> Self {
		Self {
			id: TrackId(c.id.clone()),
			title: c.title.or(c.name).unwrap_or_else(|| "Unknown".to_string()),
			artist: c.artist.unwrap_or_else(|| "Unknown".to_string()),
			artist_id: c.artist_id.map(ArtistId),
			album: c.album.unwrap_or_else(|| "Unknown".to_string()),
			album_id: c.album_id.map(AlbumId),
			duration_secs: c.duration.unwrap_or(0) as u32,
			cover_art: c.cover_art.or(Some(c.id)).map(CoverArtId),
			track_num: c.track.map(|t| t as u32),
			disc_num: c.disc_number.map(|d| d as u32),
			year: c.year.map(|y| y as u32),
			genre: c.genre,
			play_count: c.play_count.map(|c| c as u64),
			bit_rate: c.bit_rate.map(|b| b as u32),
			size: c.size.map(|s| s as u64),
			created_timestamp: None,
			starred_timestamp: None,
			content_type: c.content_type,
			suffix: c.suffix,
			starred: parse_is_starred(&c.starred),
			user_rating: c.user_rating,
			musicbrainz_id: c.music_brainz_id,
			lastfm_url: None, // Subsonic track objects rarely have lastfm urls directly
			replay_gain: c.replay_gain.map(ReplayGain::from),
			bpm: c.bpm,
			comment: c.comment,
			sort_name: c.sort_name,
		}
	}
}

impl From<dto::Child> for Album {
	fn from(c: dto::Child) -> Self {
		Self {
			id: AlbumId(c.id.clone()),
			title: c.title.or(c.name).unwrap_or_else(|| "Unknown".to_string()),
			artist: c.artist.unwrap_or_else(|| "Unknown".to_string()),
			artist_id: c.artist_id.map(ArtistId),
			year: c.year.map(|y| y as u32),
			cover_art: c.cover_art.or(Some(c.id)).map(CoverArtId),
			duration_secs: c.duration.map(|d| d as u32),
			play_count: c.play_count.map(|c| c as u64),
			created_timestamp: None,
			starred_timestamp: None,
			song_count: None,
			starred: parse_is_starred(&c.starred),
			user_rating: c.user_rating,
			release_type: None,
			musicbrainz_id: c.music_brainz_id,
			lastfm_url: None,
			genre: c.genre,
		}
	}
}

impl From<dto::Album> for AlbumDetails {
	fn from(a: dto::Album) -> Self {
		Self {
			id: AlbumId(a.id.clone()),
			title: a.name,
			artist: a.artist.unwrap_or_else(|| "Unknown".to_string()),
			artist_id: a.artist_id.map(ArtistId),
			cover_art: a.cover_art.or(Some(a.id)).map(CoverArtId),
			song_count: a.song_count.unwrap_or(0) as u32,
			duration_secs: a.duration.unwrap_or(0) as u32,
			year: a.year.map(|y| y as u32),
			genre: a.genre,
			play_count: a.play_count.map(|c| c as u64),
			created_timestamp: None,
			starred_timestamp: None,
			starred: parse_is_starred(&a.starred),
			user_rating: a.user_rating,
			release_type: None,
			musicbrainz_id: a.music_brainz_id,
			songs: a.song.into_iter().map(Track::from).collect(),
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
			starred: parse_is_starred(&a.starred),
			musicbrainz_id: a.music_brainz_id,
			lastfm_url: a.last_fm_url,
		}
	}
}

impl From<dto::Playlist> for Playlist {
	fn from(pl: dto::Playlist) -> Self {
		Self {
			id: PlaylistId(pl.id.clone()),
			name: pl.name,
			song_count: pl.song_count.unwrap_or(0) as u32,
			duration_secs: pl.duration.unwrap_or(0) as u32,
			cover_art: pl.cover_art.or(Some(pl.id)).map(CoverArtId),
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
			id: PlaylistId(pl.base.id.clone()),
			name: pl.base.name.clone(),
			song_count: pl.base.song_count.unwrap_or(0) as u32,
			duration_secs: pl.base.duration.unwrap_or(0) as u32,
			cover_art: pl.base.cover_art.clone().or(Some(pl.base.id.clone())).map(CoverArtId),
			owner: pl.base.owner.clone(),
			public: pl.base.public,
			created_timestamp: pl.base.created.timestamp(),
			changed_timestamp: pl.base.changed.timestamp(),
			comment: pl.base.comment.clone(),
			songs: pl.entry.into_iter().map(Track::from).collect(),
		}
	}
}

impl From<navidrome_dto::NavidromeAlbum> for Album {
	fn from(a: navidrome_dto::NavidromeAlbum) -> Self {
		Self {
			id: AlbumId(a.id.clone()),
			title: a.name,
			artist: a.artist.unwrap_or_else(|| "Unknown".to_string()),
			artist_id: a.artist_id.map(ArtistId),
			year: a.min_year.map(|y| y as u32),
			cover_art: Some(CoverArtId(a.id)),
			duration_secs: a.duration.map(|d| d as u32),
			play_count: a.play_count.map(|c| c as u64),
			created_timestamp: None,
			starred_timestamp: None,
			song_count: a.song_count.map(|c| c as u32),
			starred: a.starred,
			user_rating: a.rating,
			release_type: if a.compilation == Some(true) {
				Some("Compilation".to_string())
			} else {
				None
			},
			musicbrainz_id: None, // Nd album doesn't always return mbz id in list
			lastfm_url: None,
			genre: a.genre,
		}
	}
}

impl From<navidrome_dto::NavidromeSong> for Track {
	fn from(s: navidrome_dto::NavidromeSong) -> Self {
		Self {
			id: TrackId(s.id.clone()),
			title: s.title,
			artist: s.artist,
			artist_id: Some(ArtistId(s.artist_id)),
			album: s.album,
			album_id: Some(AlbumId(s.album_id)),
			duration_secs: s.duration.unwrap_or(0.0) as u32,
			cover_art: Some(CoverArtId(s.id)),
			track_num: s.track_number,
			disc_num: s.disc_number,
			year: s.year,
			genre: s.genre,
			play_count: s.play_count,
			bit_rate: s.bit_rate,
			size: s.size.map(|sz| sz as u64),
			created_timestamp: None,
			starred_timestamp: None,
			content_type: None,
			suffix: s.suffix,
			starred: s.starred,
			user_rating: s.rating,
			musicbrainz_id: None, // Map if needed
			lastfm_url: None,
			replay_gain: Some(ReplayGain {
				track_gain: s.rg_track_gain,
				track_peak: s.rg_track_peak,
				album_gain: s.rg_album_gain,
				album_peak: s.rg_album_peak,
			}),
			bpm: None,
			comment: None,
			sort_name: s.order_title,
		}
	}
}
