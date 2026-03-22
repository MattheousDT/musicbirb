use crate::backend::{AudioBackend, BackendEvent, PlayerState, PlayerStatus};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::sync::mpsc;

/// Transparently wraps any future and ensures that the Tokio runtime context is active
/// while it is being polled across the FFI boundaries.
pub struct WithTokio<F> {
	future: F,
}

impl<F> WithTokio<F> {
	pub fn new(future: F) -> Self {
		Self { future }
	}
}

impl<F: Future> Future for WithTokio<F> {
	type Output = F::Output;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let _guard = crate::RUNTIME.enter();

		// SAFETY: We never move the inner future out, so we can safely pin project it.
		unsafe { self.map_unchecked_mut(|s| &mut s.future).poll(cx) }
	}
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct FfiPlayerState {
	pub position_secs: f64,
	pub status: PlayerStatus,
	pub playlist_index: i32,
	pub playlist_count: i32,
}

#[uniffi::export(callback_interface)]
pub trait AudioEngineDelegate: Send + Sync {
	fn play(&self);
	fn pause(&self);
	fn toggle_pause(&self);
	fn stop(&self);
	fn add(&self, url: String);
	fn insert(&self, url: String, index: i32);
	fn remove_index(&self, index: i32);
	fn clear_playlist(&self);
	fn play_index(&self, index: i32);
	fn seek_relative(&self, seconds: f64);
	fn seek_absolute(&self, seconds: f64);
	fn set_volume(&self, volume: f64);
	fn get_volume(&self) -> f64;
	fn get_state(&self) -> FfiPlayerState;
}

#[uniffi::export(callback_interface)]
pub trait StateObserver: Send + Sync {
	fn on_state_changed(&self, state: crate::state::UiState);
}

pub struct MobileBackend {
	delegate: Box<dyn AudioEngineDelegate>,
	tx: Arc<Mutex<Option<mpsc::UnboundedSender<BackendEvent>>>>,
}

impl MobileBackend {
	pub fn new(
		delegate: Box<dyn AudioEngineDelegate>,
		tx: Arc<Mutex<Option<mpsc::UnboundedSender<BackendEvent>>>>,
	) -> Self {
		Self { delegate, tx }
	}
}

#[async_trait::async_trait]
impl AudioBackend for MobileBackend {
	fn set_event_sender(&self, tx: mpsc::UnboundedSender<BackendEvent>) {
		*self.tx.lock().unwrap() = Some(tx);
	}

	async fn play(&self) -> Result<(), crate::MusicbirbError> {
		self.delegate.play();
		Ok(())
	}
	async fn pause(&self) -> Result<(), crate::MusicbirbError> {
		self.delegate.pause();
		Ok(())
	}
	async fn toggle_pause(&self) -> Result<(), crate::MusicbirbError> {
		self.delegate.toggle_pause();
		Ok(())
	}
	async fn stop(&self) -> Result<(), crate::MusicbirbError> {
		self.delegate.stop();
		Ok(())
	}
	async fn add(&self, url: &str) -> Result<(), crate::MusicbirbError> {
		self.delegate.add(url.to_string());
		Ok(())
	}
	async fn insert(&self, url: &str, index: i64) -> Result<(), crate::MusicbirbError> {
		self.delegate.insert(url.to_string(), index as i32);
		Ok(())
	}
	async fn remove_index(&self, index: i64) -> Result<(), crate::MusicbirbError> {
		self.delegate.remove_index(index as i32);
		Ok(())
	}
	async fn clear_playlist(&self) -> Result<(), crate::MusicbirbError> {
		self.delegate.clear_playlist();
		Ok(())
	}
	async fn play_index(&self, index: i64) -> Result<(), crate::MusicbirbError> {
		self.delegate.play_index(index as i32);
		Ok(())
	}
	async fn seek_relative(&self, seconds: f64) -> Result<(), crate::MusicbirbError> {
		self.delegate.seek_relative(seconds);
		Ok(())
	}
	async fn seek_absolute(&self, seconds: f64) -> Result<(), crate::MusicbirbError> {
		self.delegate.seek_absolute(seconds);
		Ok(())
	}
	async fn set_volume(&self, volume: f64) -> Result<(), crate::MusicbirbError> {
		self.delegate.set_volume(volume);
		Ok(())
	}
	async fn get_volume(&self) -> Result<f64, crate::MusicbirbError> {
		Ok(self.delegate.get_volume())
	}
	fn get_state(&self) -> PlayerState {
		let st = self.delegate.get_state();
		PlayerState {
			position_secs: st.position_secs,
			status: st.status,
			playlist_index: st.playlist_index as i64,
			playlist_count: st.playlist_count as i64,
			timestamp: std::time::Instant::now(),
		}
	}
}

#[derive(uniffi::Object)]
pub struct AudioEventTarget {
	tx: Arc<Mutex<Option<mpsc::UnboundedSender<BackendEvent>>>>,
}

impl AudioEventTarget {
	pub fn new(tx: Arc<Mutex<Option<mpsc::UnboundedSender<BackendEvent>>>>) -> Self {
		Self { tx }
	}
}

#[uniffi::export]
impl AudioEventTarget {
	pub fn on_status_update(&self, status: PlayerStatus) {
		if let Some(tx) = self.tx.lock().unwrap().as_ref() {
			let _ = tx.send(BackendEvent::StatusUpdate(status));
		}
	}
	pub fn on_track_started(&self) {
		if let Some(tx) = self.tx.lock().unwrap().as_ref() {
			let _ = tx.send(BackendEvent::TrackStarted);
		}
	}
	pub fn on_end_of_track(&self) {
		if let Some(tx) = self.tx.lock().unwrap().as_ref() {
			let _ = tx.send(BackendEvent::EndOfTrack);
		}
	}
	pub fn on_position_correction(&self, seconds: f64) {
		if let Some(tx) = self.tx.lock().unwrap().as_ref() {
			let _ = tx.send(BackendEvent::PositionCorrection {
				seconds,
				timestamp: std::time::Instant::now(),
			});
		}
	}
}
