uniffi::setup_scaffolding!("musicbirb_ffi");
musicbirb::uniffi_reexport_scaffolding!();

use lazy_static::lazy_static;
use musicbirb::{AudioBackend, BackendEvent, Musicbirb, PlayerState, PlayerStatus, SubsonicClient};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::sync::mpsc;

lazy_static! {
	static ref RUNTIME: tokio::runtime::Runtime =
		tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
}

#[derive(Error, Debug, uniffi::Error)]
pub enum FfiInitError {
	#[error("Initialization error: {0}")]
	Init(String),
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
	fn on_state_changed(&self, state: musicbirb::state::UiState);
}

pub struct MobileBackend {
	delegate: Box<dyn AudioEngineDelegate>,
	tx: Arc<Mutex<Option<mpsc::UnboundedSender<BackendEvent>>>>,
}

#[async_trait::async_trait]
impl AudioBackend for MobileBackend {
	fn set_event_sender(&self, tx: mpsc::UnboundedSender<BackendEvent>) {
		*self.tx.lock().unwrap() = Some(tx);
	}

	async fn play(&self) -> Result<(), musicbirb::MusicbirbError> {
		self.delegate.play();
		Ok(())
	}
	async fn pause(&self) -> Result<(), musicbirb::MusicbirbError> {
		self.delegate.pause();
		Ok(())
	}
	async fn toggle_pause(&self) -> Result<(), musicbirb::MusicbirbError> {
		self.delegate.toggle_pause();
		Ok(())
	}
	async fn stop(&self) -> Result<(), musicbirb::MusicbirbError> {
		self.delegate.stop();
		Ok(())
	}
	async fn add(&self, url: &str) -> Result<(), musicbirb::MusicbirbError> {
		self.delegate.add(url.to_string());
		Ok(())
	}
	async fn insert(&self, url: &str, index: i64) -> Result<(), musicbirb::MusicbirbError> {
		self.delegate.insert(url.to_string(), index as i32);
		Ok(())
	}
	async fn remove_index(&self, index: i64) -> Result<(), musicbirb::MusicbirbError> {
		self.delegate.remove_index(index as i32);
		Ok(())
	}
	async fn clear_playlist(&self) -> Result<(), musicbirb::MusicbirbError> {
		self.delegate.clear_playlist();
		Ok(())
	}
	async fn play_index(&self, index: i64) -> Result<(), musicbirb::MusicbirbError> {
		self.delegate.play_index(index as i32);
		Ok(())
	}
	async fn seek_relative(&self, seconds: f64) -> Result<(), musicbirb::MusicbirbError> {
		self.delegate.seek_relative(seconds);
		Ok(())
	}
	async fn seek_absolute(&self, seconds: f64) -> Result<(), musicbirb::MusicbirbError> {
		self.delegate.seek_absolute(seconds);
		Ok(())
	}
	async fn set_volume(&self, volume: f64) -> Result<(), musicbirb::MusicbirbError> {
		self.delegate.set_volume(volume);
		Ok(())
	}
	async fn get_volume(&self) -> Result<f64, musicbirb::MusicbirbError> {
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

#[derive(uniffi::Object)]
pub struct MusicbirbMobile {
	core: Arc<Musicbirb>,
	event_target: Arc<AudioEventTarget>,
}

#[uniffi::export]
impl MusicbirbMobile {
	#[uniffi::constructor]
	pub fn new(
		url: String,
		user: String,
		pass: String,
		data_dir: String,
		cache_dir: String,
		delegate: Box<dyn AudioEngineDelegate>,
		observer: Box<dyn StateObserver>,
	) -> Result<Arc<Self>, FfiInitError> {
		let _guard = RUNTIME.enter();

		let api = SubsonicClient::new(&url, &user, &pass)
			.map_err(|e| FfiInitError::Init(e.to_string()))?;

		let shared_tx = Arc::new(Mutex::new(None));
		let backend = Arc::new(MobileBackend {
			delegate,
			tx: Arc::clone(&shared_tx),
		});

		let event_target = Arc::new(AudioEventTarget {
			tx: Arc::clone(&shared_tx),
		});

		let data_dir_opt = if data_dir.is_empty() {
			None
		} else {
			Some(std::path::PathBuf::from(data_dir))
		};
		let cache_dir_opt = if cache_dir.is_empty() {
			None
		} else {
			Some(std::path::PathBuf::from(cache_dir))
		};

		let core = Musicbirb::with_paths(api, backend, data_dir_opt, cache_dir_opt);

		let mut state_rx = core.subscribe();
		let observer_arc = Arc::new(observer);
		let core_clone = Arc::clone(&core);

		RUNTIME.spawn(async move {
			observer_arc.on_state_changed(core_clone.get_ui_state()); // Push initial
			while state_rx.changed().await.is_ok() {
				observer_arc.on_state_changed(core_clone.get_ui_state()); // Push updates
			}
		});

		Ok(Arc::new(Self { core, event_target }))
	}

	pub fn get_event_target(&self) -> Arc<AudioEventTarget> {
		Arc::clone(&self.event_target)
	}

	pub fn get_ui_state(&self) -> musicbirb::state::UiState {
		self.core.get_ui_state()
	}

	pub async fn queue_track(&self, id: String) -> Result<(), musicbirb::MusicbirbError> {
		let core = Arc::clone(&self.core);
		RUNTIME
			.spawn(async move { core.queue_track(id.into()).await })
			.await
			.map_err(|e| musicbirb::MusicbirbError::Internal(e.to_string()))?
	}

	pub async fn queue_album(&self, id: String) -> Result<u32, musicbirb::MusicbirbError> {
		let core = Arc::clone(&self.core);
		RUNTIME
			.spawn(async move { core.queue_album(id.into()).await })
			.await
			.map_err(|e| musicbirb::MusicbirbError::Internal(e.to_string()))?
	}

	pub async fn queue_playlist(&self, id: String) -> Result<u32, musicbirb::MusicbirbError> {
		let core = Arc::clone(&self.core);
		RUNTIME
			.spawn(async move { core.queue_playlist(id.into()).await })
			.await
			.map_err(|e| musicbirb::MusicbirbError::Internal(e.to_string()))?
	}

	pub async fn play_track(&self, id: String) -> Result<(), musicbirb::MusicbirbError> {
		let core = Arc::clone(&self.core);
		RUNTIME
			.spawn(async move { core.play_track(id.into()).await })
			.await
			.map_err(|e| musicbirb::MusicbirbError::Internal(e.to_string()))?
	}

	pub async fn play_album(&self, id: String) -> Result<u32, musicbirb::MusicbirbError> {
		let core = Arc::clone(&self.core);
		RUNTIME
			.spawn(async move { core.play_album(id.into()).await })
			.await
			.map_err(|e| musicbirb::MusicbirbError::Internal(e.to_string()))?
	}

	pub async fn play_playlist(&self, id: String) -> Result<u32, musicbirb::MusicbirbError> {
		let core = Arc::clone(&self.core);
		RUNTIME
			.spawn(async move { core.play_playlist(id.into()).await })
			.await
			.map_err(|e| musicbirb::MusicbirbError::Internal(e.to_string()))?
	}

	pub fn clear_queue(&self) -> Result<(), musicbirb::MusicbirbError> {
		self.core.clear_queue()
	}

	pub fn remove_index(&self, index: u32) -> Result<(), musicbirb::MusicbirbError> {
		self.core.remove_index(index)
	}

	pub async fn get_last_played_albums(
		&self,
	) -> Result<Vec<musicbirb::models::Album>, musicbirb::MusicbirbError> {
		let core = Arc::clone(&self.core);
		RUNTIME
			.spawn(async move { core.get_last_played_albums().await })
			.await
			.map_err(|e| musicbirb::MusicbirbError::Internal(e.to_string()))?
	}

	pub async fn get_recently_added_albums(
		&self,
	) -> Result<Vec<musicbirb::models::Album>, musicbirb::MusicbirbError> {
		let core = Arc::clone(&self.core);
		RUNTIME
			.spawn(async move { core.get_recently_added_albums().await })
			.await
			.map_err(|e| musicbirb::MusicbirbError::Internal(e.to_string()))?
	}

	pub async fn get_new_releases(
		&self,
	) -> Result<Vec<musicbirb::models::Album>, musicbirb::MusicbirbError> {
		let core = Arc::clone(&self.core);
		RUNTIME
			.spawn(async move { core.get_newly_released_albums().await })
			.await
			.map_err(|e| musicbirb::MusicbirbError::Internal(e.to_string()))?
	}

	pub async fn get_playlists(
		&self,
	) -> Result<Vec<musicbirb::models::Playlist>, musicbirb::MusicbirbError> {
		let core = Arc::clone(&self.core);
		RUNTIME
			.spawn(async move { core.get_playlists().await })
			.await
			.map_err(|e| musicbirb::MusicbirbError::Internal(e.to_string()))?
	}

	pub fn next(&self) -> Result<(), musicbirb::MusicbirbError> {
		self.core.next()
	}

	pub fn prev(&self) -> Result<(), musicbirb::MusicbirbError> {
		self.core.prev()
	}

	pub fn play_index(&self, index: u32) -> Result<(), musicbirb::MusicbirbError> {
		self.core.play_index(index)
	}

	pub fn seek(&self, seconds: f64) -> Result<(), musicbirb::MusicbirbError> {
		self.core.seek(seconds)
	}

	pub fn toggle_pause(&self) -> Result<(), musicbirb::MusicbirbError> {
		self.core.toggle_pause()
	}

	pub fn shutdown(&self) {
		self.core.shutdown()
	}
}
