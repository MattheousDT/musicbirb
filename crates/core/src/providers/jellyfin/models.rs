use crate::{
	Album, AlbumId, ArtistId, CoverArtId, Playlist, PlaylistId, ReleaseSubtype, ReleaseType, Track, TrackId,
	providers::jellyfin::dto::*,
};

impl From<BaseItemDto> for Album {
	fn from(item: BaseItemDto) -> Self {
		Album {
			id: AlbumId(item.id.clone()),
			title: item.name.unwrap_or_else(|| "Unknown".to_string()),
			artist: item
				.artists
				.and_then(|a| a.first().cloned())
				.unwrap_or_else(|| "Unknown".to_string()),
			artist_id: item.artist_items.and_then(|mut a| a.pop().map(|x| ArtistId(x.id))),
			year: item.production_year,
			cover_art: Some(CoverArtId(item.id)), // Jellyfin serves art directly from item id
			duration_secs: item.run_time_ticks.map(|t| (t / 10_000_000) as u32),
			play_count: None,
			created_timestamp: None,
			starred_timestamp: None,
			song_count: None,
			starred: None,
			user_rating: None,
			release_type: ReleaseType::Album,
			release_subtype: ReleaseSubtype::None,
			musicbrainz_id: None,
			lastfm_url: None,
			genre: None,
		}
	}
}

impl From<BaseItemDto> for Track {
	fn from(item: BaseItemDto) -> Self {
		Track {
			id: TrackId(item.id.clone()),
			title: item.name.unwrap_or_else(|| "Unknown".to_string()),
			artist: item
				.artists
				.and_then(|a| a.first().cloned())
				.unwrap_or_else(|| "Unknown".to_string()),
			artist_id: item.artist_items.and_then(|mut a| a.pop().map(|x| ArtistId(x.id))),
			album: item.album.unwrap_or_else(|| "Unknown".to_string()),
			album_id: item.album_id.map(AlbumId),
			duration_secs: item.run_time_ticks.map(|t| (t / 10_000_000) as u32).unwrap_or(0),
			cover_art: Some(CoverArtId(item.id)),
			track_num: item.index_number,
			disc_num: item.parent_index_number,
			year: item.production_year,
			genre: None,
			play_count: None,
			bit_rate: None,
			size: None,
			created_timestamp: None,
			starred_timestamp: None,
			content_type: None,
			suffix: None,
			starred: None,
			user_rating: None,
			musicbrainz_id: None,
			lastfm_url: None,
			replay_gain: None,
			bpm: None,
			comment: None,
			sort_name: None,
		}
	}
}

impl From<BaseItemDto> for Playlist {
	fn from(item: BaseItemDto) -> Self {
		Playlist {
			id: PlaylistId(item.id.clone()),
			name: item.name.unwrap_or_else(|| "Unknown".to_string()),
			song_count: item.child_count.unwrap_or(0),
			duration_secs: item.run_time_ticks.map(|t| (t / 10_000_000) as u32).unwrap_or(0),
			cover_art: Some(CoverArtId(item.id)),
			owner: None,
			public: None,
			created_timestamp: 0,
			changed_timestamp: 0,
			comment: None,
		}
	}
}
