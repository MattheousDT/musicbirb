//! Audio backend interface for `musicbirb`.
//!
//! This module defines the generic [`AudioBackend`] trait which abstracts over
//! underlying platform-specific audio playback engines.

use crate::error::MusicbirbError;

#[cfg(feature = "mpv")]
pub mod mpv;

/// Represents the current playback status of the audio backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStatus {
	/// Playback is completely stopped or no media is loaded.
	Stopped,
	/// Media is currently playing.
	Playing,
	/// Playback is temporarily paused.
	Paused,
}

/// A snapshot of the audio backend's current state.
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
}

/// The generic interface required for all audio engines used by the core player.
///
/// Implementers of this trait are responsible for interfacing with their respective
/// native audio libraries, managing playback queues, and returning accurate states.
pub trait AudioBackend: Send + Sync {
	/// Resumes playback (the inverse of pause).
	fn play(&self) -> Result<(), MusicbirbError>;

	/// Pauses the current playback.
	fn pause(&self) -> Result<(), MusicbirbError>;

	/// Toggles the current playback state between playing and paused.
	fn toggle_pause(&self) -> Result<(), MusicbirbError>;

	/// Completely stops current playback. Does not guarantee clearing the playlist.
	fn stop(&self) -> Result<(), MusicbirbError>;

	/// Appends a media stream to the end of the backend's internal playlist.
	fn add(&self, url: &str) -> Result<(), MusicbirbError>;

	/// Inserts a media stream at a specific index in the backend's internal playlist.
	fn insert(&self, url: &str, index: i64) -> Result<(), MusicbirbError>;

	/// Removes an item from the underlying engine's internal playlist by index.
	fn remove_index(&self, index: i64) -> Result<(), MusicbirbError>;

	/// Clears the backend's internal playlist.
	fn clear_playlist(&self) -> Result<(), MusicbirbError>;

	/// Starts playback at the specified index in the backend's internal playlist.
	fn play_index(&self, index: i64) -> Result<(), MusicbirbError>;

	/// Seeks the current playback position by a relative offset in seconds.
	///
	/// Can be a negative value to seek backward.
	fn seek_relative(&self, seconds: f64) -> Result<(), MusicbirbError>;

	/// Seeks the current playback position to an absolute time in seconds.
	fn seek_absolute(&self, seconds: f64) -> Result<(), MusicbirbError>;

	/// Sets the current playback volume.
	fn set_volume(&self, volume: f64) -> Result<(), MusicbirbError>;

	/// Retrieves the current playback volume.
	fn get_volume(&self) -> Result<f64, MusicbirbError>;

	/// Retrieves a snapshot of the player's current state (status, time, and playlist info).
	///
	/// This is typically called frequently (e.g., on a tick interval) by the orchestrator
	/// to sync the core state with the backend's state.
	fn get_state(&self) -> PlayerState;
}
