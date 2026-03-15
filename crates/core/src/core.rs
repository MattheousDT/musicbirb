use crate::actor::CoreActor;
use crate::api::subsonic::SubsonicClient;
use crate::backend::AudioBackend;
use crate::error::MusicbirbError;
use crate::models::{
	Album, AlbumDetails, AlbumId, ArtistDetails, ArtistId, Playlist, PlaylistId, TrackId,
};
use crate::state::{CoreMessage, CoreState};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

/// The macro that protects React Native from missing Tokio contexts.
/// If `ffi` is enabled, it shunts the payload into our rock-solid background runtime.
/// If `ffi` is disabled, it behaves as a normal, zero-overhead await.
macro_rules! run_async {
	($future:expr) => {{
		#[cfg(feature = "ffi")]
		let res = crate::RUNTIME
			.spawn($future)
			.await
			.map_err(|e| crate::error::MusicbirbError::Internal(e.to_string()))
			.and_then(|r| r);

		#[cfg(not(feature = "ffi"))]
		let res = $future.await;

		res
	}};
}

#[cfg(feature = "ffi")]
#[uniffi::export]
pub fn init_client(
	url: String,
	user: String,
	pass: String,
	data_dir: String,
	cache_dir: String,
	delegate: Box<dyn crate::ffi::AudioEngineDelegate>,
	observer: Box<dyn crate::ffi::StateObserver>,
) -> Result<Arc<Musicbirb>, MusicbirbError> {
	// The magic bullet: Synchronous init forces the Tokio context to exist instantly
	let _guard = crate::RUNTIME.enter();

	let api = SubsonicClient::new(&url, &user, &pass)?;

	let shared_tx = Arc::new(std::sync::Mutex::new(None));
	let backend = Arc::new(crate::ffi::MobileBackend::new(
		delegate,
		Arc::clone(&shared_tx),
	));
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

	let core = Arc::new(Musicbirb {
		api: Arc::new(api),
		tx: tx.clone(),
		state_rx: state_rx.clone(),
		event_target: Some(Arc::clone(&event_target)),
	});

	let actor = CoreActor::new(data_dir_opt, cache_dir_opt);
	let api_clone = Arc::clone(&core.api);

	crate::RUNTIME.spawn(async move {
		actor.run(rx, tx, state_tx, api_clone, backend).await;
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
	api: Arc<SubsonicClient>,
	tx: mpsc::UnboundedSender<CoreMessage>,
	state_rx: watch::Receiver<CoreState>,
	#[cfg(feature = "ffi")]
	event_target: Option<Arc<crate::ffi::AudioEventTarget>>,
}

#[cfg_attr(feature = "ffi", uniffi::export)]
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

	pub async fn queue_track(self: Arc<Self>, id: TrackId) -> Result<(), MusicbirbError> {
		run_async!(async move {
			let track = self.api.get_track(&id).await?;
			self.tx
				.send(CoreMessage::AddTracks(vec![track]))
				.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
			Ok(())
		})
	}

	pub async fn queue_album(self: Arc<Self>, id: AlbumId) -> Result<u32, MusicbirbError> {
		run_async!(async move {
			let tracks = self.api.get_album_tracks(&id).await?;
			let count = tracks.len();
			self.tx
				.send(CoreMessage::AddTracks(tracks))
				.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
			Ok(count as u32)
		})
	}

	pub async fn queue_playlist(self: Arc<Self>, id: PlaylistId) -> Result<u32, MusicbirbError> {
		run_async!(async move {
			let tracks = self.api.get_playlist_tracks(&id).await?;
			let count = tracks.len();
			self.tx
				.send(CoreMessage::AddTracks(tracks))
				.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
			Ok(count as u32)
		})
	}

	pub async fn play_track(self: Arc<Self>, id: TrackId) -> Result<(), MusicbirbError> {
		run_async!(async move {
			let track = self.api.get_track(&id).await?;
			self.tx
				.send(CoreMessage::ReplaceTracks(vec![track]))
				.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
			Ok(())
		})
	}

	pub async fn play_album(self: Arc<Self>, id: AlbumId) -> Result<u32, MusicbirbError> {
		run_async!(async move {
			let tracks = self.api.get_album_tracks(&id).await?;
			let count = tracks.len();
			self.tx
				.send(CoreMessage::ReplaceTracks(tracks))
				.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
			Ok(count as u32)
		})
	}

	pub async fn play_playlist(self: Arc<Self>, id: PlaylistId) -> Result<u32, MusicbirbError> {
		run_async!(async move {
			let tracks = self.api.get_playlist_tracks(&id).await?;
			let count = tracks.len();
			self.tx
				.send(CoreMessage::ReplaceTracks(tracks))
				.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
			Ok(count as u32)
		})
	}

	pub async fn get_last_played_albums(self: Arc<Self>) -> Result<Vec<Album>, MusicbirbError> {
		run_async!(async move { self.api.get_last_played_albums().await })
	}

	pub async fn get_recently_added_albums(self: Arc<Self>) -> Result<Vec<Album>, MusicbirbError> {
		run_async!(async move { self.api.get_recently_added_albums().await })
	}

	pub async fn get_newly_released_albums(self: Arc<Self>) -> Result<Vec<Album>, MusicbirbError> {
		run_async!(async move { self.api.get_newly_released_albums().await })
	}

	pub async fn get_album_details(
		self: Arc<Self>,
		album_id: AlbumId,
	) -> Result<AlbumDetails, MusicbirbError> {
		run_async!(async move { self.api.get_album_details(&album_id).await })
	}

	pub async fn get_artist_details(
		self: Arc<Self>,
		artist_id: ArtistId,
	) -> Result<ArtistDetails, MusicbirbError> {
		run_async!(async move { self.api.get_artist_details(&artist_id).await })
	}

	pub async fn get_playlists(self: Arc<Self>) -> Result<Vec<Playlist>, MusicbirbError> {
		run_async!(async move { self.api.get_playlists().await })
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
}

// Pure Rust Methods
impl Musicbirb {
	pub fn new(api: SubsonicClient, player: Arc<dyn AudioBackend>) -> Arc<Self> {
		Self::with_paths(api, player, None, None)
	}

	pub fn with_paths(
		api: SubsonicClient,
		player: Arc<dyn AudioBackend>,
		data_dir: Option<PathBuf>,
		cache_dir: Option<PathBuf>,
	) -> Arc<Self> {
		let (tx, rx) = mpsc::unbounded_channel();
		let (state_tx, state_rx) = watch::channel(CoreState::default());
		let api_arc = Arc::new(api);

		let core = Arc::new(Self {
			api: Arc::clone(&api_arc),
			tx: tx.clone(),
			state_rx,
			#[cfg(feature = "ffi")]
			event_target: None,
		});

		let actor = CoreActor::new(data_dir, cache_dir);
		let api_clone = Arc::clone(&api_arc);
		let tx_clone = tx.clone();

		// Update this to use the same logic as our macro:
		// Use the background RUNTIME for FFI, but standard tokio::spawn for everything else.
		#[cfg(feature = "ffi")]
		crate::RUNTIME.spawn(async move {
			actor.run(rx, tx_clone, state_tx, api_clone, player).await;
		});

		#[cfg(not(feature = "ffi"))]
		tokio::spawn(async move {
			actor.run(rx, tx_clone, state_tx, api_clone, player).await;
		});

		core
	}

	pub fn subscribe(&self) -> watch::Receiver<CoreState> {
		self.state_rx.clone()
	}
}
