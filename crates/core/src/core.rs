use crate::{
	AlbumId, AudioBackend, CoreMessage, CoreState, CoverArtId, MusicbirbError, Playlist, PlaylistId, Provider, TrackId,
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
	observer: Box<dyn crate::ffi::StateObserver>,
) -> Result<Arc<Musicbirb>, MusicbirbError> {
	let _guard = crate::RUNTIME.enter();

	#[cfg(feature = "rodio")]
	let backend = Arc::new(crate::backend::rodio::RodioBackend::new()?);

	#[cfg(not(feature = "rodio"))]
	panic!("Rodio backend feature is required for ffi!");

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
	});

	let actor = CoreActor::new(data_dir_opt, cache_dir_opt);

	crate::RUNTIME.spawn(async move {
		actor.run(rx, tx, state_tx, api_lock, backend).await;
	});

	let observer_arc = Arc::new(observer);

	crate::RUNTIME.spawn(async move {
		let mut last_queue = Vec::new();

		{
			let state = state_rx.borrow().clone();
			last_queue = state.queue.clone();
			observer_arc.on_queue_changed(state.queue);
			observer_arc.on_playback_state_changed(crate::state::PlaybackState {
				queue_position: state.queue_position as u32,
				position_secs: state.sync.position_secs,
				status: state.sync.status,
				scrobble_mark_pos: state.scrobble_mark_pos,
			});
		}

		while state_rx.changed().await.is_ok() {
			let state = state_rx.borrow().clone();

			if state.queue != last_queue {
				last_queue = state.queue.clone();
				observer_arc.on_queue_changed(state.queue);
			}

			observer_arc.on_playback_state_changed(crate::state::PlaybackState {
				queue_position: state.queue_position as u32,
				position_secs: state.sync.position_secs,
				status: state.sync.status,
				scrobble_mark_pos: state.scrobble_mark_pos,
			});
		}
	});

	Ok(core)
}

#[cfg_attr(feature = "ffi", derive(uniffi::Object))]
pub struct Musicbirb {
	api: Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>,
	tx: mpsc::UnboundedSender<CoreMessage>,
	state_rx: watch::Receiver<CoreState>,
}

#[cfg_attr(feature = "ffi", uniffi::export)]
#[macros::async_ffi]
impl Musicbirb {
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

	pub async fn create_playlist(
		self: Arc<Self>,
		name: String,
		description: Option<String>,
		is_public: bool,
	) -> Result<Playlist, MusicbirbError> {
		let provider = self.get_provider().await?;
		provider.playlist().create_playlist(&name, description, is_public).await
	}

	pub async fn update_playlist(
		self: Arc<Self>,
		id: PlaylistId,
		name: Option<String>,
		description: Option<String>,
		is_public: Option<bool>,
	) -> Result<(), MusicbirbError> {
		let provider = self.get_provider().await?;
		provider
			.playlist()
			.update_playlist(&id, name, description, is_public)
			.await
	}

	pub async fn delete_playlist(self: Arc<Self>, id: PlaylistId) -> Result<(), MusicbirbError> {
		let provider = self.get_provider().await?;
		provider.playlist().delete_playlist(&id).await
	}

	pub async fn add_tracks_to_playlist(
		self: Arc<Self>,
		id: PlaylistId,
		track_ids: Vec<TrackId>,
		allow_duplicates: bool,
	) -> Result<u32, MusicbirbError> {
		let provider = self.get_provider().await?;
		let mut final_ids = track_ids;
		let mut skipped = 0;

		if !allow_duplicates {
			let existing_tracks = provider.playlist().get_playlist_tracks(&id).await?;
			let existing_ids: std::collections::HashSet<_> = existing_tracks.into_iter().map(|t| t.id.0).collect();
			let original_count = final_ids.len();
			final_ids.retain(|tid| !existing_ids.contains(&tid.0));
			skipped = original_count - final_ids.len();
		}

		if !final_ids.is_empty() {
			provider.playlist().add_to_playlist(&id, final_ids).await?;
		}

		Ok(skipped as u32)
	}

	pub async fn add_album_to_playlist(
		self: Arc<Self>,
		id: PlaylistId,
		album_id: AlbumId,
		allow_duplicates: bool,
	) -> Result<u32, MusicbirbError> {
		let provider = self.get_provider().await?;
		let tracks = provider.album().get_album_tracks(&album_id).await?;
		let track_ids: Vec<TrackId> = tracks.into_iter().map(|t| t.id).collect();
		self.add_tracks_to_playlist(id, track_ids, allow_duplicates).await
	}

	pub async fn replace_playlist_tracks(
		self: Arc<Self>,
		id: PlaylistId,
		track_ids: Vec<TrackId>,
	) -> Result<(), MusicbirbError> {
		let provider = self.get_provider().await?;
		provider.playlist().replace_playlist_tracks(&id, track_ids).await
	}

	pub async fn remove_from_playlist(
		self: Arc<Self>,
		id: PlaylistId,
		indices: Vec<u32>,
	) -> Result<(), MusicbirbError> {
		let provider = self.get_provider().await?;
		provider.playlist().remove_from_playlist(&id, indices).await
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

	pub fn move_index(&self, from: u32, to: u32) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::MoveIndex(from as usize, to as usize))
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

	pub fn play(&self) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::Play)
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))
	}

	pub fn pause(&self) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::Pause)
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
