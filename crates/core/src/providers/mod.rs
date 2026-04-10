use crate::error::MusicbirbError;
use crate::models::{
	AlbumDetails, AlbumId, ArtistDetails, ArtistId, CoverArtId, Playlist, PlaylistDetails, PlaylistId, SearchQuery,
	SearchResults, Track, TrackId, TrackScrobble,
};
use moka_query::moka_query_proxy;
use std::sync::Arc;

#[cfg(feature = "jellyfin")]
pub mod jellyfin;
#[cfg(feature = "subsonic")]
pub mod subsonic;

/// The primary gateway to backend APIs.
/// Providers return Arc-wrapped domain sub-providers to keep logic modular.
#[cfg_attr(feature = "uniffi", uniffi::export(with_foreign))]
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
	/// Validates that the provider's connection and tokens are still active.
	async fn ping(&self) -> Result<(), MusicbirbError>;

	/// Access methods for streaming and raw media bytes (e.g., covers)
	fn media(&self) -> Arc<dyn MediaProvider>;
	/// Access methods for singular tracks and track-level interactions
	fn track(&self) -> Arc<CachedTrackProvider>;
	/// Access methods for albums
	fn album(&self) -> Arc<CachedAlbumProvider>;
	/// Access methods for artists
	fn artist(&self) -> Arc<CachedArtistProvider>;
	/// Access methods for playlists
	fn playlist(&self) -> Arc<CachedPlaylistProvider>;
	/// Access methods for scrobbling and presence
	fn activity(&self) -> Arc<CachedActivityProvider>;
	/// Access methods for discovery and generalized searching
	fn search(&self) -> Arc<CachedSearchProvider>;
}

/// Handles low-level stream URLs and fetching raw image bytes.
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[macros::async_ffi]
pub trait MediaProvider: Send + Sync {
	/// Retrieves the direct streaming URL for a given track, attaching necessary auth tokens.
	async fn get_stream_url(&self, track_id: &TrackId) -> Result<String, MusicbirbError>;

	/// Constructs a URL to fetch cover art, optionally constrained to a specific size.
	fn get_cover_art_url(&self, cover_id: &CoverArtId, size: Option<u32>) -> Result<String, MusicbirbError>;

	/// Directly downloads cover art bytes to be cached locally.
	async fn get_cover_art_bytes(&self, cover_id: &CoverArtId) -> Result<Vec<u8>, MusicbirbError>;
}

/// Handles fetching and modifying specific tracks.
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[moka_query_proxy(namespace = "Track")]
pub trait TrackProvider: Send + Sync {
	/// Fetches a single track's metadata.
	#[query(key = "Track({track_id:?})")]
	async fn get_track(&self, track_id: &TrackId) -> Result<Track, MusicbirbError>;
}

/// Handles fetching and modifying albums.
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[moka_query_proxy(namespace = "Album")]
pub trait AlbumProvider: Send + Sync {
	/// Fetches all tracks associated with a given album ID.
	#[query(key = "AlbumTracks({album_id:?})")]
	async fn get_album_tracks(&self, album_id: &AlbumId) -> Result<Vec<Track>, MusicbirbError>;

	/// Fetches detailed album metadata, including the full list of tracks inside it.
	#[query(key = "AlbumDetails({album_id:?})")]
	async fn get_album_details(&self, album_id: &AlbumId) -> Result<AlbumDetails, MusicbirbError>;
}

/// Handles fetching and modifying artists.
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[moka_query_proxy(namespace = "Artist")]
pub trait ArtistProvider: Send + Sync {
	/// Fetches basic artist metadata, biography, and similar artists.
	#[query(key = "ArtistDetails({artist_id:?})")]
	async fn get_artist_details(&self, artist_id: &ArtistId) -> Result<ArtistDetails, MusicbirbError>;

	/// Fetches global top songs for an artist.
	#[query(key = "TopSongs({artist_id:?})")]
	async fn get_top_songs(&self, artist_id: &ArtistId) -> Result<Vec<Track>, MusicbirbError>;

	/// Fetches top songs for an artist based on the current user's play history.
	#[query(key = "PersonalTopSongs({artist_id:?})")]
	async fn get_personal_top_songs(&self, artist_id: &ArtistId) -> Result<Vec<Track>, MusicbirbError>;
}

/// Handles fetching, browsing, and editing playlists.
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[moka_query_proxy(namespace = "Playlist")]
pub trait PlaylistProvider: Send + Sync {
	/// Fetches all playlists owned by or visible to the user.
	#[query(key = "AllPlaylists")]
	async fn get_playlists(&self) -> Result<Vec<Playlist>, MusicbirbError>;

	/// Fetches all tracks associated with a given playlist ID.
	#[query(key = "PlaylistTracks({playlist_id:?})")]
	async fn get_playlist_tracks(&self, playlist_id: &PlaylistId) -> Result<Vec<Track>, MusicbirbError>;

	/// Fetches detailed playlist metadata, including its tracks and track count.
	#[query(key = "PlaylistDetails({playlist_id:?})")]
	async fn get_playlist_details(&self, playlist_id: &PlaylistId) -> Result<PlaylistDetails, MusicbirbError>;

	/// Creates a new empty playlist on the server.
	#[mutation(invalidates = ["Playlist/*"])]
	async fn create_playlist(
		&self,
		name: String,
		description: Option<String>,
		public: bool,
	) -> Result<Playlist, MusicbirbError>;

	/// Updates an existing playlist (e.g., adding/removing tracks, renaming).
	#[mutation(invalidates = ["Playlist/*"])]
	async fn update_playlist(
		&self,
		id: &PlaylistId,
		name: Option<String>,
		description: Option<String>,
		public: Option<bool>,
	) -> Result<(), MusicbirbError>;

	/// Deletes a playlist from the server.
	#[mutation(invalidates = ["Playlist/*"])]
	async fn delete_playlist(&self, id: &PlaylistId) -> Result<(), MusicbirbError>;

	/// Adds an array of track IDs to a playlist
	#[mutation(invalidates = ["Playlist/*"])]
	async fn add_to_playlist(&self, id: &PlaylistId, track_ids: Vec<TrackId>) -> Result<(), MusicbirbError>;

	/// Removes tracks at specific index offsets from the playlist
	#[mutation(invalidates = ["Playlist/*"])]
	async fn remove_from_playlist(&self, id: &PlaylistId, track_indices: Vec<u32>) -> Result<(), MusicbirbError>;

	/// Entirely replaces a playlist's tracks with a new sequence of tracks
	#[mutation(invalidates = ["Playlist/*"])]
	async fn replace_playlist_tracks(&self, id: &PlaylistId, track_ids: Vec<TrackId>) -> Result<(), MusicbirbError>;
}

/// Handles reporting playback data back to the server.
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[moka_query_proxy(namespace = "Activity")]
pub trait ActivityProvider: Send + Sync {
	/// Reports to the server that a track has started playing.
	#[mutation(invalidates = [])]
	async fn now_playing(&self, track_id: &TrackId) -> Result<(), MusicbirbError>;

	/// Submits an array of completed track scrobbles to update play counts.
	#[mutation(invalidates =["Artist/*", "Album/*", "Search/*"])]
	async fn scrobble(&self, tracks: Vec<TrackScrobble>) -> Result<(), MusicbirbError>;
}

/// Handles searching the library, either by textual query or via explicit preset filters
/// (like Recently Added, Newly Released, etc).
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[moka_query_proxy(namespace = "Search")]
pub trait SearchProvider: Send + Sync {
	/// Submits a query to the server, returning mixed tracks, albums, and artists.
	#[query(key = "Search({query:?})")]
	async fn search(&self, query: SearchQuery) -> Result<SearchResults, MusicbirbError>;
}
