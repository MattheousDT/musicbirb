use crate::error::MusicbirbError;
use crate::models::{
	Album, AlbumDetails, AlbumId, ArtistDetails, ArtistId, CoverArtId, Playlist, PlaylistDetails, PlaylistId, Track,
	TrackId, TrackScrobble,
};
use async_trait::async_trait;

#[cfg(feature = "subsonic")]
pub mod subsonic;

#[cfg_attr(feature = "ffi", uniffi::export)]
#[async_trait]
pub trait Provider: Send + Sync {
	async fn get_stream_url(&self, track_id: &TrackId) -> Result<String, MusicbirbError>;
	async fn get_track(&self, track_id: &TrackId) -> Result<Track, MusicbirbError>;
	async fn get_album_tracks(&self, album_id: &AlbumId) -> Result<Vec<Track>, MusicbirbError>;
	async fn get_album_details(&self, album_id: &AlbumId) -> Result<AlbumDetails, MusicbirbError>;
	async fn get_artist_details(&self, artist_id: &ArtistId) -> Result<ArtistDetails, MusicbirbError>;
	async fn get_playlist_tracks(&self, playlist_id: &PlaylistId) -> Result<Vec<Track>, MusicbirbError>;
	async fn get_playlist_details(&self, playlist_id: &PlaylistId) -> Result<PlaylistDetails, MusicbirbError>;
	async fn get_cover_art_bytes(&self, cover_id: &CoverArtId) -> Result<Vec<u8>, MusicbirbError>;
	async fn now_playing(&self, track_id: &TrackId) -> Result<(), MusicbirbError>;
	async fn scrobble(&self, tracks: Vec<TrackScrobble>) -> Result<(), MusicbirbError>;
	async fn get_last_played_albums(&self) -> Result<Vec<Album>, MusicbirbError>;
	async fn get_recently_added_albums(&self) -> Result<Vec<Album>, MusicbirbError>;
	async fn get_newly_released_albums(&self) -> Result<Vec<Album>, MusicbirbError>;
	async fn get_playlists(&self) -> Result<Vec<Playlist>, MusicbirbError>;
}
