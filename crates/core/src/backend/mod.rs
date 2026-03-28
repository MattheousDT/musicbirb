use crate::error::MusicbirbError;
use std::time::Instant;
use tokio::sync::mpsc;

#[cfg(feature = "mpv")]
pub mod mpv;

#[cfg(feature = "rodio")]
pub mod rodio;

/// Represents the current playback status of the audio backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
pub enum PlayerStatus {
	/// Playback is completely stopped or no media is loaded.
	Stopped,
	/// Media is currently playing.
	Playing,
	/// Playback is temporarily paused.
	Paused,
	/// Track is being loaded
	Buffering,
}

/// Events emitted by the audio backend to notify the core actor of state changes.
#[derive(Debug, Clone)]
pub enum BackendEvent {
	/// Emitted when the play/pause/stop status changes.
	StatusUpdate(PlayerStatus),
	/// Emitted periodically or on seek to synchronize the current playback time.
	PositionCorrection {
		/// The current playback position in seconds.
		seconds: f64,
		/// The monotonic timestamp when this position was recorded.
		timestamp: Instant,
	},
	/// Emitted when the backend successfully starts a new file in its internal playlist.
	TrackStarted,
	/// Emitted when a track reaches its natural end.
	EndOfTrack,
}

/// A snapshot of the audio backend's internal state.
#[derive(Debug, Clone)]
pub struct PlayerState {
	/// Current playback position in seconds.
	pub position_secs: f64,
	/// Current playback status.
	pub status: PlayerStatus,
	/// The index of the currently playing track in the backend's internal playlist.
	pub playlist_index: i64,
	/// The total number of items in the backend's internal playlist.
	pub playlist_count: i64,
	/// The monotonic timestamp when this state snapshot was captured.
	pub timestamp: Instant,
}

/// Defines the interface for audio playback engines.
#[macros::async_ffi]
pub trait AudioBackend: Send + Sync {
	/// Provides the backend with a channel to emit events.
	///
	/// This is called once during the initialization of the core actor. Implementation
	/// should be non-blocking.
	fn set_event_sender(&self, tx: mpsc::UnboundedSender<BackendEvent>);

	/// Resumes playback (the inverse of pause).
	async fn play(&self) -> Result<(), MusicbirbError>;

	/// Pauses the current playback.
	async fn pause(&self) -> Result<(), MusicbirbError>;

	/// Toggles the current playback state between playing and paused.
	async fn toggle_pause(&self) -> Result<(), MusicbirbError>;

	/// Completely stops current playback.
	async fn stop(&self) -> Result<(), MusicbirbError>;

	/// Appends a media stream URL to the end of the backend's internal playlist.
	async fn add(&self, url: &str) -> Result<(), MusicbirbError>;

	/// Inserts a media stream URL at a specific index in the backend's internal playlist.
	async fn insert(&self, url: &str, index: i64) -> Result<(), MusicbirbError>;

	/// Removes an item from the backend's internal playlist by index.
	async fn remove_index(&self, index: i64) -> Result<(), MusicbirbError>;

	/// Clears all items from the backend's internal playlist.
	async fn clear_playlist(&self) -> Result<(), MusicbirbError>;

	/// Starts playback of the item at the specified index in the backend's internal playlist.
	async fn play_index(&self, index: i64) -> Result<(), MusicbirbError>;

	/// Seeks the current playback position to an absolute time in seconds.
	async fn seek(&self, seconds: f64) -> Result<(), MusicbirbError>;

	/// Seeks the current playback position by a relative offset in seconds.
	///
	/// Use a negative value to seek backward.
	async fn seek_relative(&self, seconds: f64) -> Result<(), MusicbirbError>;

	/// Sets the output volume (typically 0.0 to 100.0).
	async fn set_volume(&self, volume: f64) -> Result<(), MusicbirbError>;

	/// Retrieves the current output volume.
	async fn get_volume(&self) -> Result<f64, MusicbirbError>;

	/// Retrieves a snapshot of the player's current state.
	///
	/// This is called reactively by the orchestrator to reconcile internal core state
	/// with the truth of the audio engine after events or user messages occur.
	fn get_state(&self) -> PlayerState;
}
