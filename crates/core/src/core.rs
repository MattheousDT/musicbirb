use crate::{
	AlbumId, AudioBackend, CoreMessage, CoreState, CoverArtId, MusicbirbError, PlaylistId, Provider, TrackId,
	actor::CoreActor,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn init_client(
	provider: Option<Arc<dyn crate::Provider>>,
	data_dir: String,
	cache_dir: String,
	delegate: Box<dyn crate::ffi::AudioEngineDelegate>,
	observer: Box<dyn crate::ffi::StateObserver>,
) -> Result<Arc<Musicbirb>, MusicbirbError> {
	let _guard = crate::RUNTIME.enter();

	let shared_tx = Arc::new(std::sync::Mutex::new(None));
	let backend = Arc::new(crate::ffi::MobileBackend::new(delegate, Arc::clone(&shared_tx)));
	let event_target = Arc::new(crate::ffi::AudioEventTarget::new(Arc::clone(&shared_tx)));

	let data_dir_opt = if data_dir.is_empty() {
		None
	} else {
		Some(PathBuf::from(data_dir))
	};
	let cache_dir_opt = if cache_dir.is_empty() {
		None
	} else {
		Some(PathBuf::from(cache_dir))
	};

	let (tx, rx) = mpsc::unbounded_channel();
	let (state_tx, mut state_rx) = watch::channel(CoreState::default());

	let api_lock = Arc::new(tokio::sync::RwLock::new(provider));
	let core = Arc::new(Musicbirb {
		api: Arc::clone(&api_lock),
		tx: tx.clone(),
		state_rx: state_rx.clone(),
		event_target: Some(Arc::clone(&event_target)),
	});

	let actor = CoreActor::new(data_dir_opt, cache_dir_opt);

	crate::RUNTIME.spawn(async move {
		actor.run(rx, tx, state_tx, api_lock, backend).await;
	});

	let observer_arc = Arc::new(observer);
	let core_clone = Arc::clone(&core);

	crate::RUNTIME.spawn(async move {
		observer_arc.on_state_changed(core_clone.get_ui_state());
		while state_rx.changed().await.is_ok() {
			observer_arc.on_state_changed(core_clone.get_ui_state());
		}
	});

	Ok(core)
}

#[cfg_attr(feature = "ffi", derive(uniffi::Object))]
pub struct Musicbirb {
	api: Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>,
	tx: mpsc::UnboundedSender<CoreMessage>,
	state_rx: watch::Receiver<CoreState>,
	#[cfg(feature = "ffi")]
	event_target: Option<Arc<crate::ffi::AudioEventTarget>>,
}

#[cfg_attr(feature = "ffi", uniffi::export)]
#[macros::async_ffi]
impl Musicbirb {
	#[cfg(feature = "ffi")]
	pub fn get_event_target(&self) -> Arc<crate::ffi::AudioEventTarget> {
		Arc::clone(self.event_target.as_ref().unwrap())
	}

	#[cfg(feature = "ffi")]
	pub fn get_ui_state(&self) -> crate::state::UiState {
		let state = self.state_rx.borrow().clone();
		crate::state::UiState {
			queue: state.queue,
			queue_position: state.queue_position as u32,
			position_secs: state.sync.position_secs,
			status: state.sync.status,
			scrobble_mark_pos: state.scrobble_mark_pos,
		}
	}

	// ------------- ASYNC METHODS WITH OUR SAFE MACRO WRAPPER -------------

	async fn get_provider(&self) -> Result<Arc<dyn Provider>, MusicbirbError> {
		self.api
			.read()
			.await
			.clone()
			.ok_or_else(|| MusicbirbError::Internal("No active provider".into()))
	}

	pub async fn queue_track(self: Arc<Self>, id: TrackId, next: bool) -> Result<(), MusicbirbError> {
		let provider: Arc<dyn Provider> = self.get_provider().await?;
		let track = provider.track().get_track(&id).await?;
		self.tx
			.send(CoreMessage::AddTracks(vec![track], next))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
		Ok(())
	}

	pub async fn queue_album(self: Arc<Self>, id: AlbumId, next: bool) -> Result<u32, MusicbirbError> {
		let provider: Arc<dyn Provider> = self.get_provider().await?;
		let tracks = provider.album().get_album_tracks(&id).await?;
		let count = tracks.len();
		self.tx
			.send(CoreMessage::AddTracks(tracks, next))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
		Ok(count as u32)
	}

	pub async fn queue_playlist(self: Arc<Self>, id: PlaylistId, next: bool) -> Result<u32, MusicbirbError> {
		let provider: Arc<dyn Provider> = self.get_provider().await?;
		let tracks = provider.playlist().get_playlist_tracks(&id).await?;
		let count = tracks.len();
		self.tx
			.send(CoreMessage::AddTracks(tracks, next))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
		Ok(count as u32)
	}

	pub async fn play_tracks(
		self: Arc<Self>,
		ids: Vec<TrackId>,
		start_index: Option<u32>,
	) -> Result<(), MusicbirbError> {
		let provider: Arc<dyn Provider> = self.get_provider().await?;
		let mut tracks = Vec::with_capacity(ids.len());
		for id in ids {
			tracks.push(provider.track().get_track(&id).await?);
		}
		self.tx
			.send(CoreMessage::ReplaceTracks(tracks, start_index.unwrap_or(0) as usize))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
		Ok(())
	}

	pub async fn play_album(self: Arc<Self>, id: AlbumId, start_index: Option<u32>) -> Result<u32, MusicbirbError> {
		let provider: Arc<dyn Provider> = self.get_provider().await?;
		let tracks = provider.album().get_album_tracks(&id).await?;
		let count = tracks.len();
		self.tx
			.send(CoreMessage::ReplaceTracks(tracks, start_index.unwrap_or(0) as usize))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
		Ok(count as u32)
	}

	pub async fn play_playlist(
		self: Arc<Self>,
		id: PlaylistId,
		start_index: Option<u32>,
	) -> Result<u32, MusicbirbError> {
		let provider: Arc<dyn Provider> = self.get_provider().await?;
		let tracks = provider.playlist().get_playlist_tracks(&id).await?;
		let count = tracks.len();
		self.tx
			.send(CoreMessage::ReplaceTracks(tracks, start_index.unwrap_or(0) as usize))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
		Ok(count as u32)
	}

	// ------------- SYNCHRONOUS METHODS (No Macro Needed) -------------

	pub fn clear_queue(&self) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::ClearQueue)
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))
	}

	pub fn remove_index(&self, index: u32) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::RemoveIndex(index as usize))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))
	}

	pub fn next(&self) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::Next)
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))
	}

	pub fn get_cover_art_url(&self, id: CoverArtId, size: Option<u32>) -> Option<String> {
		let api = self.api.try_read().ok()?;
		api.as_ref()?.media().get_cover_art_url(&id, size).ok()
	}

	pub fn prev(&self) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::Prev)
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))
	}

	pub fn play_index(&self, index: u32) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::PlayIndex(index as usize))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))
	}

	pub fn seek(&self, seconds: f64) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::Seek(seconds))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))
	}

	pub fn seek_relative(&self, seconds: f64) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::SeekRelative(seconds))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))
	}

	pub fn toggle_pause(&self) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::TogglePause)
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))
	}

	pub fn shutdown(&self) {
		let _ = self.tx.send(CoreMessage::Shutdown);
	}

	pub async fn set_provider(self: Arc<Self>, provider: Option<Arc<dyn Provider>>) {
		*self.api.write().await = provider;
		let _ = self.tx.send(CoreMessage::ProviderChanged);
		Ok::<(), MusicbirbError>(()).unwrap();
	}
}

// Pure Rust Methods
impl Musicbirb {
	pub fn new(api: Option<Arc<dyn Provider>>, player: Arc<dyn AudioBackend>) -> Arc<Self> {
		Self::with_paths(api, player, None, None)
	}

	pub fn with_paths(
		api: Option<Arc<dyn Provider>>,
		player: Arc<dyn AudioBackend>,
		data_dir: Option<PathBuf>,
		cache_dir: Option<PathBuf>,
	) -> Arc<Self> {
		let (tx, rx) = mpsc::unbounded_channel();
		let (state_tx, state_rx) = watch::channel(CoreState::default());

		let api_lock = Arc::new(tokio::sync::RwLock::new(api));
		let core = Arc::new(Self {
			api: Arc::clone(&api_lock),
			tx: tx.clone(),
			state_rx,
			#[cfg(feature = "ffi")]
			event_target: None,
		});

		let actor = CoreActor::new(data_dir, cache_dir);
		let tx_clone = tx.clone();

		#[cfg(feature = "ffi")]
		crate::RUNTIME.spawn(async move {
			actor.run(rx, tx_clone, state_tx, api_lock, player).await;
		});

		#[cfg(not(feature = "ffi"))]
		tokio::spawn(async move {
			actor.run(rx, tx_clone, state_tx, api_lock, player).await;
		});

		core
	}

	pub fn subscribe(&self) -> watch::Receiver<CoreState> {
		self.state_rx.clone()
	}
}
