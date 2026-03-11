uniffi::setup_scaffolding!("musicbirb_ffi");

use lazy_static::lazy_static;
use musicbirb::{
	AlbumId, AudioBackend, BackendEvent, Musicbirb, PlayerState, PlayerStatus, PlaylistId,
	SubsonicClient, TrackId,
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

lazy_static! {
	static ref RUNTIME: tokio::runtime::Runtime =
		tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
}

#[derive(uniffi::Enum, Clone, Copy, Debug)]
pub enum FfiPlayerStatus {
	Stopped,
	Playing,
	Paused,
}

impl From<FfiPlayerStatus> for PlayerStatus {
	fn from(s: FfiPlayerStatus) -> Self {
		match s {
			FfiPlayerStatus::Stopped => PlayerStatus::Stopped,
			FfiPlayerStatus::Playing => PlayerStatus::Playing,
			FfiPlayerStatus::Paused => PlayerStatus::Paused,
		}
	}
}

impl From<PlayerStatus> for FfiPlayerStatus {
	fn from(s: PlayerStatus) -> Self {
		match s {
			PlayerStatus::Stopped => FfiPlayerStatus::Stopped,
			PlayerStatus::Playing => FfiPlayerStatus::Playing,
			PlayerStatus::Paused => FfiPlayerStatus::Paused,
		}
	}
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct FfiPlayerState {
	pub position_secs: f64,
	pub status: FfiPlayerStatus,
	pub playlist_index: i32,
	pub playlist_count: i32,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct FfiTrack {
	pub id: String,
	pub title: String,
	pub artist: String,
	pub album: String,
	pub duration_secs: u32,
	pub cover_art_id: Option<String>,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct FfiUiState {
	pub queue: Vec<FfiTrack>,
	pub queue_position: u32,
	pub position_secs: f64,
	pub is_playing: bool,
	pub scrobble_mark_pos: Option<f64>,
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
			status: st.status.into(),
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
	pub fn on_status_update(&self, status: FfiPlayerStatus) {
		if let Some(tx) = self.tx.lock().unwrap().as_ref() {
			let _ = tx.send(BackendEvent::StatusUpdate(status.into()));
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
		delegate: Box<dyn AudioEngineDelegate>,
	) -> Arc<Self> {
		let _guard = RUNTIME.enter();

		let api = SubsonicClient::new(&url, &user, &pass).unwrap();
		let shared_tx = Arc::new(Mutex::new(None));

		let backend = Arc::new(MobileBackend {
			delegate,
			tx: Arc::clone(&shared_tx),
		});

		let event_target = Arc::new(AudioEventTarget {
			tx: Arc::clone(&shared_tx),
		});

		let core = Musicbirb::new(api, backend);
		Arc::new(Self { core, event_target })
	}

	pub fn get_event_target(&self) -> Arc<AudioEventTarget> {
		Arc::clone(&self.event_target)
	}

	pub fn queue_track(&self, id: String) {
		let core = Arc::clone(&self.core);
		RUNTIME.spawn(async move {
			let _ = core.queue_track(&TrackId(id)).await;
		});
	}

	pub fn queue_album(&self, id: String) {
		let core = Arc::clone(&self.core);
		RUNTIME.spawn(async move {
			let _ = core.queue_album(&AlbumId(id)).await;
		});
	}

	pub fn queue_playlist(&self, id: String) {
		let core = Arc::clone(&self.core);
		RUNTIME.spawn(async move {
			let _ = core.queue_playlist(&PlaylistId(id)).await;
		});
	}

	pub fn next(&self) {
		let _ = self.core.next();
	}

	pub fn prev(&self) {
		let _ = self.core.prev();
	}

	pub fn play_index(&self, index: i32) {
		let _ = self.core.play_index(index as usize);
	}

	pub fn toggle_pause(&self) {
		let _ = self.core.toggle_pause();
	}

	pub fn seek(&self, seconds: f64) {
		let _ = self.core.seek(seconds);
	}

	pub fn get_ui_state(&self) -> FfiUiState {
		let state = self.core.subscribe().borrow().clone();

		FfiUiState {
			queue: state
				.queue
				.into_iter()
				.map(|t| FfiTrack {
					id: t.id.0,
					title: t.title,
					artist: t.artist,
					album: t.album,
					duration_secs: t.duration_secs,
					cover_art_id: t.cover_art.map(|c| c.0),
				})
				.collect(),
			queue_position: state.queue_position as u32,
			position_secs: state.sync.position_secs,
			is_playing: state.sync.status == PlayerStatus::Playing,
			scrobble_mark_pos: state.scrobble_mark_pos,
		}
	}
}
