use crate::art_cache::ArtCache;
use crate::backend::{AudioBackend, BackendEvent, PlayerState, PlayerStatus};
use crate::models::{CoverArtId, Track};
use crate::providers::Provider;
use crate::scrobble::{ScrobbleManager, ScrobbleTracker};
use crate::state::{CoreMessage, CoreState, PlaybackSync};
use image::DynamicImage;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::{mpsc, watch};
use tokio::time::{Sleep, sleep};

#[cfg(feature = "os-media-controls")]
use crate::mpris::MprisManager;

pub struct CoreActor {
	queue: Vec<Track>,
	queue_position: usize,
	active_index: Option<usize>,
	fetching_index: Option<usize>,
	preloading_index: Option<usize>,

	art_cache: ArtCache,
	current_art_id: Option<CoverArtId>,
	current_art: Option<Arc<DynamicImage>>,

	scrobble_tracker: ScrobbleTracker,
	scrobble_manager: Arc<Mutex<ScrobbleManager>>,
	scrobble_timer: Option<Pin<Box<Sleep>>>,
	scrobble_flush_timer: Option<Pin<Box<Sleep>>>,

	auto_play: bool,
	last_track_start_time: Instant,

	#[cfg(feature = "os-media-controls")]
	mpris: Option<MprisManager>,
}

impl CoreActor {
	/// Creates a new CoreActor instance with default state and initialized scrobble managers.
	pub fn new(data_dir: Option<PathBuf>, cache_dir: Option<PathBuf>) -> Self {
		Self {
			queue: Vec::new(),
			queue_position: 0,
			active_index: None,
			fetching_index: None,
			preloading_index: None,

			art_cache: ArtCache::new(cache_dir),
			current_art_id: None,
			current_art: None,

			scrobble_tracker: ScrobbleTracker::new(),
			scrobble_manager: Arc::new(Mutex::new(ScrobbleManager::new(data_dir))),
			scrobble_timer: None,
			scrobble_flush_timer: Some(Box::pin(sleep(std::time::Duration::from_secs(60)))),

			auto_play: true,
			last_track_start_time: Instant::now(),

			#[cfg(feature = "os-media-controls")]
			mpris: None,
		}
	}

	/// Main event loop for the core actor.
	/// Listens for messages from the UI, events from the audio backend, and scrobble timers.
	pub async fn run(
		mut self,
		mut rx: mpsc::UnboundedReceiver<CoreMessage>,
		tx: mpsc::UnboundedSender<CoreMessage>,
		state_tx: watch::Sender<CoreState>,
		api: Arc<dyn Provider>,
		player: Arc<dyn AudioBackend>,
	) {
		#[cfg(feature = "os-media-controls")]
		{
			self.mpris = MprisManager::new(tx.clone());
		}

		let (backend_tx, mut backend_rx) = mpsc::unbounded_channel();
		player.set_event_sender(backend_tx);

		loop {
			tokio::select! {
				msg = rx.recv() => {
					match msg {
						Some(CoreMessage::Shutdown) | None => break,
						Some(m) => self.handle_message(m, &player, &api, &tx, &state_tx).await,
					}
				},
				Some(event) = backend_rx.recv() => self.handle_backend_event(event, &player, &api, &tx, &state_tx).await,
				_ = async {
					if let Some(timer) = self.scrobble_timer.as_mut() {
						timer.await;
					} else {
						std::future::pending::<()>().await;
					}
				}, if self.scrobble_timer.is_some() => {
					self.scrobble_timer = None;
					self.trigger_scrobble(&api);
				},
				_ = async {
					if let Some(timer) = self.scrobble_flush_timer.as_mut() {
						timer.await;
					} else {
						std::future::pending::<()>().await;
					}
				} => {
					self.scrobble_flush_timer = Some(Box::pin(sleep(std::time::Duration::from_secs(60))));
					self.spawn_scrobble_flush(&api);
				}
			}
		}

		let _ = player.stop().await;
	}

	/// Commits the time elapsed since the last playback sync to the scrobble tracker.
	fn sync_scrobble_duration(&mut self, current_sync: &PlaybackSync) {
		if current_sync.status == PlayerStatus::Playing {
			let elapsed = std::time::Instant::now().duration_since(current_sync.timestamp);
			self.scrobble_tracker.commit_played_time(elapsed);
		}
	}

	/// Handles events emitted by the audio backend (e.g., MPV).
	async fn handle_backend_event(
		&mut self,
		event: BackendEvent,
		player: &Arc<dyn AudioBackend>,
		api: &Arc<dyn Provider>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
		state_tx: &watch::Sender<CoreState>,
	) {
		let current_sync = state_tx.borrow().sync.clone();
		let p_state = player.get_state();

		match event {
			BackendEvent::TrackStarted => {
				self.sync_scrobble_duration(&current_sync);
				// If the backend has moved to the second item in its internal playlist,
				// and that item matches what we preloaded, we treat it as a gapless transition.
				if p_state.playlist_index > 0
					&& self.preloading_index == Some(self.queue_position + 1)
				{
					self.advance_queue_gapless(player, api, state_tx).await;
				} else {
					self.dispatch_state(&p_state, state_tx);
				}
			}
			BackendEvent::StatusUpdate(_) => {
				self.sync_scrobble_duration(&current_sync);
				self.scrobble_tracker.sync_position(p_state.position_secs);

				if p_state.status == PlayerStatus::Playing {
					self.start_scrobble_timer();
				} else {
					self.scrobble_timer = None;
				}

				if p_state.status == PlayerStatus::Stopped {
					if self.active_index.is_some() {
						if self.queue_position + 1 < self.queue.len() {
							self.advance_queue(player, api, tx, state_tx).await;
						} else {
							self.reset_to_start(player, api, tx, state_tx).await;
							return;
						}
					}
				}
				self.dispatch_state(&p_state, state_tx);
			}
			BackendEvent::PositionCorrection { seconds, .. } => {
				self.sync_scrobble_duration(&current_sync);
				self.scrobble_tracker.sync_position(seconds);

				if p_state.playlist_index > 0
					&& self.preloading_index == Some(self.queue_position + 1)
				{
					self.advance_queue_gapless(player, api, state_tx).await;
				}

				if p_state.status == PlayerStatus::Playing {
					self.start_scrobble_timer();
				}
				self.dispatch_state(&p_state, state_tx);
			}
			BackendEvent::EndOfTrack => {
				self.scrobble_timer = None;

				if self.active_index.is_some() && self.last_track_start_time.elapsed().as_secs() > 2
				{
					if self.queue_position + 1 < self.queue.len() {
						self.advance_queue(player, api, tx, state_tx).await;
					} else {
						self.reset_to_start(player, api, tx, state_tx).await;
					}
				}
			}
		}
		self.sync_resources(api, tx, &player.get_state());
	}

	/// Steps the queue forward when the backend has stopped naturally.
	async fn advance_queue(
		&mut self,
		player: &Arc<dyn AudioBackend>,
		api: &Arc<dyn Provider>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
		state_tx: &watch::Sender<CoreState>,
	) {
		self.queue_position += 1;
		self.active_index = None;
		self.fetching_index = None;
		self.preloading_index = None;
		self.scrobble_timer = None;
		self.auto_play = true;

		let _ = player.stop().await;
		let _ = player.clear_playlist().await;

		self.sync_resources(api, tx, &player.get_state());
		self.dispatch_state(&player.get_state(), state_tx);
	}

	/// Steps the queue forward when the backend has already started playing the next track in its buffer.
	async fn advance_queue_gapless(
		&mut self,
		player: &Arc<dyn AudioBackend>,
		api: &Arc<dyn Provider>,
		state_tx: &watch::Sender<CoreState>,
	) {
		self.queue_position += 1;
		self.active_index = Some(self.queue_position);
		self.preloading_index = None;

		self.on_track_start(api);
		// Remove the finished track from the backend's internal playlist
		let _ = player.remove_index(0).await;
		let _ = player.play().await;

		self.dispatch_state(&player.get_state(), state_tx);
	}

	/// Resets the queue to the beginning and stops playback.
	async fn reset_to_start(
		&mut self,
		player: &Arc<dyn AudioBackend>,
		api: &Arc<dyn Provider>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
		state_tx: &watch::Sender<CoreState>,
	) {
		self.queue_position = 0;
		self.active_index = None;
		self.fetching_index = None;
		self.preloading_index = None;
		self.scrobble_timer = None;
		self.auto_play = false;

		let _ = player.stop().await;
		let _ = player.clear_playlist().await;

		let p_state = player.get_state();
		self.sync_resources(api, tx, &p_state);
		self.dispatch_state(&p_state, state_tx);
	}

	/// Processes incoming messages from the UI or Internal components.
	async fn handle_message(
		&mut self,
		msg: CoreMessage,
		player: &Arc<dyn AudioBackend>,
		api: &Arc<dyn Provider>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
		state_tx: &watch::Sender<CoreState>,
	) {
		let current_sync = state_tx.borrow().sync.clone();
		self.sync_scrobble_duration(&current_sync);

		match msg {
			CoreMessage::Shutdown => {}
			CoreMessage::AddTracks(tracks) => {
				self.queue.extend(tracks);
				self.sync_resources(api, tx, &player.get_state());
				self.dispatch_state(&player.get_state(), state_tx);
			}
			CoreMessage::ReplaceTracks(tracks) => {
				self.queue = tracks;
				self.queue_position = 0;
				self.active_index = None;
				self.fetching_index = None;
				self.preloading_index = None;
				self.scrobble_timer = None;
				self.auto_play = true;

				let _ = player.stop().await;
				let _ = player.clear_playlist().await;

				let p_state = player.get_state();
				self.sync_resources(api, tx, &p_state);
				self.dispatch_state(&p_state, state_tx);
			}
			CoreMessage::ClearQueue => {
				self.queue.clear();
				self.queue_position = 0;
				self.active_index = None;
				self.fetching_index = None;
				self.preloading_index = None;
				self.scrobble_timer = None;
				self.auto_play = false;

				let _ = player.stop().await;
				let _ = player.clear_playlist().await;

				let p_state = player.get_state();
				self.sync_resources(api, tx, &p_state);
				self.dispatch_state(&p_state, state_tx);
			}
			CoreMessage::RemoveIndex(idx) => {
				if idx >= self.queue.len() {
					return;
				}

				self.queue.remove(idx);

				if idx < self.queue_position {
					// We removed an earlier track, decrement our tracking indexes to match the shift
					self.queue_position -= 1;
					if let Some(ref mut a) = self.active_index {
						*a -= 1;
					}
					if let Some(ref mut f) = self.fetching_index {
						*f -= 1;
					}
					if let Some(ref mut p) = self.preloading_index {
						*p -= 1;
					}
				} else if idx == self.queue_position {
					// We removed the currently playing track! Hard stop and load whatever is now at this position.
					self.active_index = None;
					self.fetching_index = None;
					self.preloading_index = None;
					self.scrobble_timer = None;

					let _ = player.stop().await;
					let _ = player.clear_playlist().await;

					if self.queue_position >= self.queue.len() {
						if self.queue.is_empty() {
							self.queue_position = 0;
						} else {
							self.queue_position = self.queue.len() - 1;
						}
					}
				} else {
					// We removed a future track. If it was the one currently preloaded in gapless memory, evict it.
					if Some(idx) == self.preloading_index {
						self.preloading_index = None;
						if player.get_state().playlist_count > 1 {
							let _ = player.remove_index(1).await;
						}
					} else if let Some(p) = self.preloading_index {
						if idx < p {
							self.preloading_index = Some(p - 1);
						}
					}
				}

				let p_state = player.get_state();
				self.sync_resources(api, tx, &p_state);
				self.dispatch_state(&p_state, state_tx);
			}
			CoreMessage::Next | CoreMessage::Prev | CoreMessage::PlayIndex(_) => {
				let possible = match msg {
					CoreMessage::Next => self.queue_position + 1 < self.queue.len(),
					CoreMessage::Prev => self.queue_position > 0,
					CoreMessage::PlayIndex(idx) => idx < self.queue.len(),
					_ => false,
				};

				if possible {
					self.queue_position = match msg {
						CoreMessage::Next => self.queue_position + 1,
						CoreMessage::Prev => self.queue_position - 1,
						CoreMessage::PlayIndex(idx) => idx,
						_ => self.queue_position,
					};

					// Hard reset on explicit track change
					self.active_index = None;
					self.fetching_index = None;
					self.preloading_index = None;
					self.scrobble_timer = None;
					self.auto_play = true;

					let _ = player.stop().await;
					let _ = player.clear_playlist().await;

					self.sync_resources(api, tx, &player.get_state());
					self.dispatch_state(&player.get_state(), state_tx);
				}
			}
			CoreMessage::SeekRelative(secs) => {
				let _ = player.seek_relative(secs).await;
			}
			CoreMessage::TogglePause => {
				let _ = player.toggle_pause().await;
			}
			CoreMessage::UrlReady {
				url,
				index,
				is_preload,
			} => {
				if is_preload && Some(index) == self.preloading_index {
					let _ = player.add(&url).await;
				} else if !is_preload && Some(index) == self.fetching_index {
					let _ = player.clear_playlist().await;
					let _ = player.add(&url).await;
					let _ = player.play_index(0).await;
					if self.auto_play {
						let _ = player.play().await;
					} else {
						let _ = player.pause().await;
					}
					self.active_index = Some(index);
					self.fetching_index = None;
					self.on_track_start(api);
					self.dispatch_state(&player.get_state(), state_tx);
				}
			}
			CoreMessage::ArtDownloaded { id, bytes } => {
				if let Some(art) = self.art_cache.save_and_load(&id, &bytes) {
					if Some(id) == self.current_art_id {
						self.current_art = Some(art);
						self.dispatch_state(&player.get_state(), state_tx);
					}
				}
			}
		}
	}

	/// Calculates and starts a timer to trigger a scrobble once the track threshold is met.
	fn start_scrobble_timer(&mut self) {
		if self.scrobble_tracker.has_scrobbled {
			self.scrobble_timer = None;
			return;
		}

		if let Some(track) = self.queue.get(self.queue_position) {
			if let Some(rem) = self
				.scrobble_tracker
				.get_remaining_duration(track.duration_secs)
			{
				self.scrobble_timer = Some(Box::pin(sleep(rem)));
			}
		}
	}

	/// Pushes a scrobble entry to the manager and attempts to flush.
	fn trigger_scrobble(&mut self, api: &Arc<dyn Provider>) {
		if let Some(track) = self.queue.get(self.queue_position) {
			self.scrobble_tracker.has_scrobbled = true;
			self.scrobble_manager
				.lock()
				.unwrap()
				.push(&track.id, self.scrobble_tracker.start_time);
			self.spawn_scrobble_flush(api);
		}
	}

	/// Updates the public core state and external integrations (MPRIS).
	fn dispatch_state(&mut self, p_state: &PlayerState, state_tx: &watch::Sender<CoreState>) {
		let track = self.active_index.and_then(|i| self.queue.get(i));
		let mark = track.and_then(|t| self.scrobble_tracker.get_mark_pos(t.duration_secs));

		#[cfg(feature = "os-media-controls")]
		if let Some(mpris) = &mut self.mpris {
			let art_path = self
				.current_art_id
				.as_ref()
				.map(|id| self.art_cache.get_path(id));
			mpris.sync(track, p_state.status, art_path.as_deref());
		}

		let _ = state_tx.send(CoreState {
			queue: self.queue.clone(),
			queue_position: self.queue_position,
			sync: PlaybackSync {
				position_secs: p_state.position_secs,
				timestamp: p_state.timestamp,
				status: p_state.status,
			},
			current_art: self.current_art.clone(),
			scrobble_mark_pos: mark,
		});
	}

	/// Handles initialization tasks when a track begins playback.
	fn on_track_start(&mut self, api: &Arc<dyn Provider>) {
		self.scrobble_tracker.reset();
		if let Some(track) = self.queue.get(self.queue_position) {
			let id = track.id.clone();
			let api_clone = Arc::clone(api);
			tokio::spawn(async move {
				let _ = api_clone.now_playing(&id).await;
			});
			self.start_scrobble_timer();
		}
	}

	/// Ensures the backend and actor have the necessary URLs and metadata for the current and next track.
	fn sync_resources(
		&mut self,
		api: &Arc<dyn Provider>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
		p_state: &PlayerState,
	) {
		if self.queue.is_empty() {
			return;
		}

		// Fetch current track URL if not active or currently being fetched
		if self.active_index != Some(self.queue_position)
			&& self.fetching_index != Some(self.queue_position)
		{
			self.fetching_index = Some(self.queue_position);
			self.spawn_url_fetch(api, tx, self.queue_position, false);
		} else if self.active_index == Some(self.queue_position)
			&& p_state.playlist_count < 2
			&& self.queue_position + 1 < self.queue.len()
			&& self.preloading_index != Some(self.queue_position + 1)
		{
			let next = self.queue_position + 1;
			self.preloading_index = Some(next);
			self.spawn_url_fetch(api, tx, next, true);
		}

		let art_id = self
			.queue
			.get(self.queue_position)
			.and_then(|t| t.cover_art.clone());
		self.update_art_state(art_id, api, tx);
	}

	/// Checks if the requested art ID is different from current and triggers a load or fetch.
	fn update_art_state(
		&mut self,
		art_id: Option<CoverArtId>,
		api: &Arc<dyn Provider>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
	) {
		if art_id != self.current_art_id {
			self.current_art_id = art_id.clone();
			self.current_art = None;
			let Some(id) = art_id else { return };
			if let Some(img) = self.art_cache.load_image(&id) {
				self.current_art = Some(img);
			} else {
				self.spawn_art_fetch(&id, api, tx);
			}
		}
	}

	/// Spawns an async task to resolve a Subsonic stream URL.
	fn spawn_url_fetch(
		&self,
		api: &Arc<dyn Provider>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
		index: usize,
		is_preload: bool,
	) {
		let track_id = self.queue[index].id.clone();
		let (api_c, tx_c) = (Arc::clone(api), tx.clone());
		tokio::spawn(async move {
			if let Ok(url) = api_c.get_stream_url(&track_id).await {
				let _ = tx_c.send(CoreMessage::UrlReady {
					url,
					index,
					is_preload,
				});
			}
		});
	}

	/// Spawns an async task to download cover art bytes.
	fn spawn_art_fetch(
		&self,
		id: &CoverArtId,
		api: &Arc<dyn Provider>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
	) {
		let (api_c, tx_c, id_c) = (Arc::clone(api), tx.clone(), id.clone());
		tokio::spawn(async move {
			if let Ok(bytes) = api_c.get_cover_art_bytes(&id_c).await {
				let _ = tx_c.send(CoreMessage::ArtDownloaded { id: id_c, bytes });
			}
		});
	}

	/// Spawns an async task to send pending scrobbles to the server.
	fn spawn_scrobble_flush(&self, api: &Arc<dyn Provider>) {
		let (sm, api_c) = (Arc::clone(&self.scrobble_manager), Arc::clone(api));
		tokio::spawn(async move {
			let items = sm.lock().unwrap().get_all();
			if !items.is_empty() && api_c.scrobble(items.clone()).await.is_ok() {
				sm.lock().unwrap().remove_flushed(items.len());
			}
		});
	}
}
