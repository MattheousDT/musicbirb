use crate::error::MusicbirbError;
use crate::models::{
	Album, AlbumDetails, AlbumId, ArtistDetails, ArtistId, CoverArtId, Playlist, PlaylistDetails, PlaylistId, Track,
	TrackId, TrackScrobble,
};
use async_trait::async_trait;

#[cfg(feature = "jellyfin")]
pub mod jellyfin;
#[cfg(feature = "subsonic")]
pub mod subsonic;

#[cfg_attr(feature = "ffi", uniffi::export)]
#[async_trait]
pub trait Provider: Send + Sync {
	/// Retrieves the direct streaming URL for a given track, attaching necessary auth tokens.
	async fn get_stream_url(&self, track_id: &TrackId) -> Result<String, MusicbirbError>;

	/// Fetches a single track's metadata.
	async fn get_track(&self, track_id: &TrackId) -> Result<Track, MusicbirbError>;

	/// Fetches all tracks associated with a given album ID.
	async fn get_album_tracks(&self, album_id: &AlbumId) -> Result<Vec<Track>, MusicbirbError>;

	/// Fetches detailed album metadata, including the list of tracks inside it.
	async fn get_album_details(&self, album_id: &AlbumId) -> Result<AlbumDetails, MusicbirbError>;

	/// Fetches detailed artist metadata, including their albums, top songs, and biography.
	async fn get_artist_details(&self, artist_id: &ArtistId) -> Result<ArtistDetails, MusicbirbError>;

	/// Fetches all tracks associated with a given playlist ID.
	async fn get_playlist_tracks(&self, playlist_id: &PlaylistId) -> Result<Vec<Track>, MusicbirbError>;

	/// Fetches detailed playlist metadata, including its tracks and track count.
	async fn get_playlist_details(&self, playlist_id: &PlaylistId) -> Result<PlaylistDetails, MusicbirbError>;

	/// Constructs a URL to fetch cover art, optionally constrained to a specific size.
	fn get_cover_art_url(&self, cover_id: &CoverArtId, size: Option<u32>) -> Result<String, MusicbirbError>;

	/// Directly downloads cover art bytes to be cached locally.
	async fn get_cover_art_bytes(&self, cover_id: &CoverArtId) -> Result<Vec<u8>, MusicbirbError>;

	/// Reports to the server that a track has started playing.
	async fn now_playing(&self, track_id: &TrackId) -> Result<(), MusicbirbError>;

	/// Submits an array of completed track scrobbles to update play counts.
	async fn scrobble(&self, tracks: Vec<TrackScrobble>) -> Result<(), MusicbirbError>;

	/// Discovery: Fetches albums recently played by the user.
	async fn get_last_played_albums(&self) -> Result<Vec<Album>, MusicbirbError>;

	/// Discovery: Fetches albums recently imported into the server library.
	async fn get_recently_added_albums(&self) -> Result<Vec<Album>, MusicbirbError>;

	/// Discovery: Fetches albums sorted by their chronological release year.
	async fn get_newly_released_albums(&self) -> Result<Vec<Album>, MusicbirbError>;

	/// Fetches all playlists owned by or visible to the user.
	async fn get_playlists(&self) -> Result<Vec<Playlist>, MusicbirbError>;

	/// Validates that the provider's connection and tokens are still active.
	async fn ping(&self) -> Result<(), MusicbirbError>;
}
