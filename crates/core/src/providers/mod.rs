use crate::error::MusicbirbError;
use crate::models::{
	AlbumDetails, AlbumId, ArtistDetails, ArtistId, CoverArtId, Playlist, PlaylistDetails, PlaylistId, SearchQuery,
	SearchResults, Track, TrackId, TrackScrobble,
};
use std::sync::Arc;

#[cfg(feature = "jellyfin")]
pub mod jellyfin;
#[cfg(feature = "subsonic")]
pub mod subsonic;

/// The primary gateway to backend APIs.
/// Providers return Arc-wrapped domain sub-providers to keep logic modular.
#[cfg_attr(feature = "ffi", uniffi::export)]
#[macros::async_ffi]
pub trait Provider: Send + Sync {
	/// Validates that the provider's connection and tokens are still active.
	async fn ping(&self) -> Result<(), MusicbirbError>;

	/// Access methods for streaming and raw media bytes (e.g., covers)
	fn media(&self) -> Arc<dyn MediaProvider>;
	/// Access methods for singular tracks and track-level interactions
	fn track(&self) -> Arc<dyn TrackProvider>;
	/// Access methods for albums
	fn album(&self) -> Arc<dyn AlbumProvider>;
	/// Access methods for artists
	fn artist(&self) -> Arc<dyn ArtistProvider>;
	/// Access methods for playlists
	fn playlist(&self) -> Arc<dyn PlaylistProvider>;
	/// Access methods for scrobbling and presence
	fn activity(&self) -> Arc<dyn ActivityProvider>;
	/// Access methods for discovery and generalized searching
	fn search(&self) -> Arc<dyn SearchProvider>;
}

/// Handles low-level stream URLs and fetching raw image bytes.
#[cfg_attr(feature = "ffi", uniffi::export)]
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
#[cfg_attr(feature = "ffi", uniffi::export)]
#[macros::async_ffi]
pub trait TrackProvider: Send + Sync {
	/// Fetches a single track's metadata.
	async fn get_track(&self, track_id: &TrackId) -> Result<Track, MusicbirbError>;

	// /// Stars a specific track, marking it as a favorite in the provider's library.
	// TODO: async fn star_track(&self, id: &TrackId) -> Result<(), MusicbirbError>;

	// /// Removes the starred status from a specific track.
	// TODO: async fn unstar_track(&self, id: &TrackId) -> Result<(), MusicbirbError>;

	// /// Sets a 1-5 user rating for a specific track.
	// TODO: async fn set_track_rating(&self, id: &TrackId, rating: u8) -> Result<(), MusicbirbError>;

	// /// Creates a bookmark for a track to save its specific playback position.
	// TODO: async fn create_bookmark(&self, id: &TrackId, position_millis: u64, comment: &str) -> Result<(), MusicbirbError>;

	// /// Deletes an existing bookmark for a track.
	// TODO: async fn delete_bookmark(&self, id: &TrackId) -> Result<(), MusicbirbError>;
}

/// Handles fetching and modifying albums.
#[cfg_attr(feature = "ffi", uniffi::export)]
#[macros::async_ffi]
pub trait AlbumProvider: Send + Sync {
	/// Fetches all tracks associated with a given album ID.
	/// This is more lightweight than `get_album_details` when only the track list is needed.
	async fn get_album_tracks(&self, album_id: &AlbumId) -> Result<Vec<Track>, MusicbirbError>;

	/// Fetches detailed album metadata, including the full list of tracks inside it.
	async fn get_album_details(&self, album_id: &AlbumId) -> Result<AlbumDetails, MusicbirbError>;

	// /// Stars a specific album, marking it as a favorite.
	// TODO: async fn star_album(&self, id: &AlbumId) -> Result<(), MusicbirbError>;

	// /// Removes the starred status from a specific album.
	// TODO: async fn unstar_album(&self, id: &AlbumId) -> Result<(), MusicbirbError>;

	// /// Sets a 1-5 user rating for a specific album.
	// TODO: async fn set_album_rating(&self, id: &AlbumId, rating: u8) -> Result<(), MusicbirbError>;
}

/// Handles fetching and modifying artists.
#[cfg_attr(feature = "ffi", uniffi::export)]
#[macros::async_ffi]
pub trait ArtistProvider: Send + Sync {
	/// Fetches detailed artist metadata, including their albums, top songs, and biography.
	async fn get_artist_details(&self, artist_id: &ArtistId) -> Result<ArtistDetails, MusicbirbError>;

	// /// Stars a specific artist, marking them as a favorite.
	// TODO: async fn star_artist(&self, id: &ArtistId) -> Result<(), MusicbirbError>;

	// /// Removes the starred status from a specific artist.
	// TODO: async fn unstar_artist(&self, id: &ArtistId) -> Result<(), MusicbirbError>;

	// /// Sets a 1-5 user rating for a specific artist.
	// TODO: async fn set_artist_rating(&self, id: &ArtistId, rating: u8) -> Result<(), MusicbirbError>;
}

/// Handles fetching, browsing, and editing playlists.
#[cfg_attr(feature = "ffi", uniffi::export)]
#[macros::async_ffi]
pub trait PlaylistProvider: Send + Sync {
	/// Fetches all playlists owned by or visible to the user.
	async fn get_playlists(&self) -> Result<Vec<Playlist>, MusicbirbError>;

	/// Fetches all tracks associated with a given playlist ID.
	/// This is more lightweight than `get_playlist_details` when only the track list is needed.
	async fn get_playlist_tracks(&self, playlist_id: &PlaylistId) -> Result<Vec<Track>, MusicbirbError>;

	/// Fetches detailed playlist metadata, including its tracks and track count.
	async fn get_playlist_details(&self, playlist_id: &PlaylistId) -> Result<PlaylistDetails, MusicbirbError>;

	// /// Creates a new empty playlist on the server.
	// TODO: async fn create_playlist(&self, name: &str) -> Result<Playlist, MusicbirbError>;

	// /// Updates an existing playlist (e.g., adding/removing tracks, renaming).
	// TODO: async fn update_playlist(&self, id: &PlaylistId, ... ) -> Result<(), MusicbirbError>;

	// /// Creates a new empty playlist on the server.
	async fn create_playlist(
		&self,
		name: &str,
		description: Option<String>,
		public: bool,
	) -> Result<Playlist, MusicbirbError>;

	/// Updates an existing playlist (e.g., adding/removing tracks, renaming).
	async fn update_playlist(
		&self,
		id: &PlaylistId,
		name: Option<String>,
		description: Option<String>,
		public: Option<bool>,
	) -> Result<(), MusicbirbError>;

	/// Deletes a playlist from the server.
	async fn delete_playlist(&self, id: &PlaylistId) -> Result<(), MusicbirbError>;

	/// Adds an array of track IDs to a playlist
	async fn add_to_playlist(&self, id: &PlaylistId, track_ids: Vec<TrackId>) -> Result<(), MusicbirbError>;

	/// Removes tracks at specific index offsets from the playlist
	async fn remove_from_playlist(&self, id: &PlaylistId, track_indices: Vec<u32>) -> Result<(), MusicbirbError>;

	/// Entirely replaces a playlist's tracks with a new sequence of tracks
	async fn replace_playlist_tracks(&self, id: &PlaylistId, track_ids: Vec<TrackId>) -> Result<(), MusicbirbError>;
}

/// Handles reporting playback data back to the server.
#[cfg_attr(feature = "ffi", uniffi::export)]
#[macros::async_ffi]
pub trait ActivityProvider: Send + Sync {
	/// Reports to the server that a track has started playing.
	async fn now_playing(&self, track_id: &TrackId) -> Result<(), MusicbirbError>;

	/// Submits an array of completed track scrobbles to update play counts.
	async fn scrobble(&self, tracks: Vec<TrackScrobble>) -> Result<(), MusicbirbError>;
}

/// Handles searching the library, either by textual query or via explicit preset filters
/// (like Recently Added, Newly Released, etc).
#[cfg_attr(feature = "ffi", uniffi::export)]
#[macros::async_ffi]
pub trait SearchProvider: Send + Sync {
	/// Submits a query to the server, returning mixed tracks, albums, and artists.
	async fn search(&self, query: SearchQuery) -> Result<SearchResults, MusicbirbError>;
}
