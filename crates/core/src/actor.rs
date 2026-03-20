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
use tokio::sync::{mpsc, watch};
use tokio::time::{Sleep, sleep};

#[cfg(feature = "os-media-controls")]
use crate::mpris::MprisManager;

pub struct CoreActor {
	queue: Vec<Track>,
	queue_position: usize,

	// Backend synchronization state
	player_has_current: bool,
	player_has_next: bool,
	fetching_current_for: Option<usize>,
	fetching_preload_for: Option<usize>,

	art_cache: ArtCache,
	current_art_id: Option<CoverArtId>,
	current_art: Option<Arc<DynamicImage>>,

	scrobble_tracker: ScrobbleTracker,
	scrobble_manager: Arc<Mutex<ScrobbleManager>>,
	scrobble_flush_timer: Option<Pin<Box<Sleep>>>,

	auto_play: bool,

	#[cfg(feature = "os-media-controls")]
	mpris: Option<MprisManager>,
}

impl CoreActor {
	pub fn new(data_dir: Option<PathBuf>, cache_dir: Option<PathBuf>) -> Self {
		Self {
			queue: Vec::new(),
			queue_position: 0,

			player_has_current: false,
			player_has_next: false,
			fetching_current_for: None,
			fetching_preload_for: None,

			art_cache: ArtCache::new(cache_dir),
			current_art_id: None,
			current_art: None,

			scrobble_tracker: ScrobbleTracker::new(),
			scrobble_manager: Arc::new(Mutex::new(ScrobbleManager::new(data_dir))),
			scrobble_flush_timer: Some(Box::pin(sleep(std::time::Duration::from_secs(60)))),

			auto_play: true,

			#[cfg(feature = "os-media-controls")]
			mpris: None,
		}
	}

	pub async fn run(
		mut self,
		mut rx: mpsc::UnboundedReceiver<CoreMessage>,
		tx: mpsc::UnboundedSender<CoreMessage>,
		state_tx: watch::Sender<CoreState>,
		api: Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>,
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
				Some(event) = backend_rx.recv() => {
					self.handle_backend_event(event, &player, &api, &tx, &state_tx).await;
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
	fn sync_scrobble_duration(&mut self, current_sync: &PlaybackSync, api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>) {
		if current_sync.status == PlayerStatus::Playing {
			let elapsed = std::time::Instant::now().duration_since(current_sync.timestamp);
			self.scrobble_tracker.commit_played_time(elapsed);

			if !self.scrobble_tracker.has_scrobbled {
				if let Some(track) = self.queue.get(self.queue_position) {
					if track.duration_secs >= 30 {
						let threshold = (track.duration_secs as f64 / 2.0).min(240.0);
						if self.scrobble_tracker.accumulated_time >= threshold {
							self.trigger_scrobble(api);
						}
					}
				}
			}
		}
	}

	/// Handles events emitted by the audio backend.
	async fn handle_backend_event(
		&mut self,
		event: BackendEvent,
		player: &Arc<dyn AudioBackend>,
		api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
		state_tx: &watch::Sender<CoreState>,
	) {
		let current_sync = state_tx.borrow().sync.clone();
		let mut p_state = player.get_state();

		// Log accumulated playtime up to this exact moment
		self.sync_scrobble_duration(&current_sync, api);

		match event {
			BackendEvent::TrackStarted => {
				self.scrobble_tracker.sync_position(p_state.position_secs);

				if p_state.playlist_index > 0 {
					let advanced_by = p_state.playlist_index as usize;
					for _ in 0..advanced_by {
						let _ = player.remove_index(0).await;
					}
					self.queue_position += advanced_by;
					self.player_has_current = true;
					self.player_has_next = false;

					self.update_art_state(api, tx);
					self.check_preload(api, tx);
				}

				self.on_track_start(api);
				self.dispatch_state(&p_state, state_tx);
			}
			BackendEvent::StatusUpdate(status) => {
				let actual_status = p_state.status;
				p_state.status = status; // Trust the explicit event status over get_state()
				self.scrobble_tracker.sync_position(p_state.position_secs);

				if status == PlayerStatus::Stopped && self.auto_play && self.player_has_current {
					// Prevent race condition: if MPV has already begun loading/playing the next track, ignore this stale Stopped event.
					if actual_status == PlayerStatus::Stopped {
						self.player_has_current = false;
						if self.queue_position + 1 < self.queue.len() {
							self.play_track_at(self.queue_position + 1, player, api, tx, state_tx)
								.await;
							return;
						}
					}
				}

				self.dispatch_state(&p_state, state_tx);
			}
			BackendEvent::PositionCorrection { seconds, .. } => {
				p_state.position_secs = seconds;
				self.scrobble_tracker.sync_position(seconds);
				self.dispatch_state(&p_state, state_tx);
			}
			BackendEvent::EndOfTrack => {}
		}
	}

	/// Processes explicit user and system messages.
	async fn handle_message(
		&mut self,
		msg: CoreMessage,
		player: &Arc<dyn AudioBackend>,
		api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
		state_tx: &watch::Sender<CoreState>,
	) {
		let current_sync = state_tx.borrow().sync.clone();
		self.sync_scrobble_duration(&current_sync, api);

		match msg {
			CoreMessage::Shutdown => {}
			CoreMessage::ProviderChanged => {
				self.queue.clear();
				self.queue_position = 0;
				self.player_has_current = false;
				self.player_has_next = false;
				self.auto_play = false;
				self.fetching_current_for = None;
				self.fetching_preload_for = None;

				let _ = player.stop().await;
				let _ = player.clear_playlist().await;

				self.update_art_state(api, tx);
				self.dispatch_state(&player.get_state(), state_tx);
			}
			CoreMessage::AddTracks(tracks, next) => {
				let was_empty = self.queue.is_empty();
				if was_empty {
					self.queue.extend(tracks);
					self.play_track_at(0, player, api, tx, state_tx).await;
				} else {
					if next {
						let insert_pos = self.queue_position + 1;

						// Evict the currently preloaded next track if we're interrupting it
						if self.player_has_next {
							let _ = player.remove_index(1).await;
							self.player_has_next = false;
							self.fetching_preload_for = None;
						}

						let tail = self.queue.split_off(insert_pos);
						self.queue.extend(tracks);
						self.queue.extend(tail);
					} else {
						self.queue.extend(tracks);
					}

					self.check_preload(api, tx);
					self.dispatch_state(&player.get_state(), state_tx);
				}
			}
			CoreMessage::ReplaceTracks(tracks, start_index) => {
				self.queue = tracks;
				let idx = if start_index < self.queue.len() { start_index } else { 0 };
				self.play_track_at(idx, player, api, tx, state_tx).await;
			}
			CoreMessage::ClearQueue => {
				self.queue.clear();
				self.queue_position = 0;
				self.player_has_current = false;
				self.player_has_next = false;
				self.auto_play = false;
				self.fetching_current_for = None;
				self.fetching_preload_for = None;

				let _ = player.stop().await;
				let _ = player.clear_playlist().await;

				self.update_art_state(api, tx);
				self.dispatch_state(&player.get_state(), state_tx);
			}
			CoreMessage::RemoveIndex(idx) => {
				if idx >= self.queue.len() {
					return;
				}

				if idx < self.queue_position {
					// We removed an earlier track, decrement our tracking index to match the shift
					self.queue.remove(idx);
					self.queue_position -= 1;
				} else if idx == self.queue_position {
					// We removed the currently playing track! Hard stop and load whatever is now at this position.
					self.queue.remove(idx);
					if self.queue.is_empty() {
						self.queue_position = 0;
						self.player_has_current = false;
						self.player_has_next = false;
						self.auto_play = false;
						self.fetching_current_for = None;
						self.fetching_preload_for = None;

						let _ = player.stop().await;
						let _ = player.clear_playlist().await;
						self.update_art_state(api, tx);
					} else {
						if self.queue_position >= self.queue.len() {
							self.queue_position = self.queue.len() - 1;
						}
						self.play_track_at(self.queue_position, player, api, tx, state_tx).await;
						return;
					}
				} else {
					// We removed a future track.
					self.queue.remove(idx);
					if idx == self.queue_position + 1 && self.player_has_next {
						// It was already loaded securely as our gapless preloaded track. Evict it.
						let _ = player.remove_index(1).await;
						self.player_has_next = false;
						self.fetching_preload_for = None;
						self.check_preload(api, tx);
					}
				}
				self.dispatch_state(&player.get_state(), state_tx);
			}
			CoreMessage::Next => {
				if self.queue_position + 1 < self.queue.len() {
					self.play_track_at(self.queue_position + 1, player, api, tx, state_tx)
						.await;
				}
			}
			CoreMessage::Prev => {
				if self.queue_position > 0 {
					self.play_track_at(self.queue_position - 1, player, api, tx, state_tx)
						.await;
				} else if !self.queue.is_empty() {
					// Restart the current track
					self.play_track_at(0, player, api, tx, state_tx).await;
				}
			}
			CoreMessage::PlayIndex(idx) => {
				if idx < self.queue.len() {
					self.play_track_at(idx, player, api, tx, state_tx).await;
				}
			}
			CoreMessage::SeekRelative(secs) => {
				let _ = player.seek_relative(secs).await;
				self.dispatch_state(&player.get_state(), state_tx);
			}
			CoreMessage::TogglePause => {
				let _ = player.toggle_pause().await;
			}
			CoreMessage::UrlReady { url, index, is_preload } => {
				if is_preload {
					if self.fetching_preload_for == Some(index) {
						self.fetching_preload_for = None;
					}
					// Verify this is still the correct next track (avoid race conditions from rapidly pressing next)
					if index == self.queue_position + 1 {
						let _ = player.add(&url).await;
						self.player_has_next = true;
					}
				} else {
					if self.fetching_current_for == Some(index) {
						self.fetching_current_for = None;
					}
					// Verify this is still the current active track
					if index == self.queue_position {
						let _ = player.add(&url).await;

						if !self.auto_play {
							let _ = player.pause().await;
						}

						let _ = player.play_index(0).await;

						if self.auto_play {
							let _ = player.play().await;
						}

						self.player_has_current = true;
						self.check_preload(api, tx);
						self.dispatch_state(&player.get_state(), state_tx);
					}
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

	/// Clears the player out and manually forces playback at a specific target index
	async fn play_track_at(
		&mut self,
		index: usize,
		player: &Arc<dyn AudioBackend>,
		api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
		state_tx: &watch::Sender<CoreState>,
	) {
		self.queue_position = index;
		self.player_has_current = false;
		self.player_has_next = false;
		self.fetching_current_for = None;
		self.fetching_preload_for = None;
		self.auto_play = true;

		let _ = player.stop().await;
		let _ = player.clear_playlist().await;

		if index < self.queue.len() {
			self.spawn_url_fetch(api, tx, index, false);
		}

		self.update_art_state(api, tx);
		self.dispatch_state(&player.get_state(), state_tx);
	}

	/// Checks to see if there is an empty index trailing the current actively playing track in the backend
	fn check_preload(&mut self, api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>, tx: &mpsc::UnboundedSender<CoreMessage>) {
		if self.player_has_current && !self.player_has_next && self.queue_position + 1 < self.queue.len() {
			self.spawn_url_fetch(api, tx, self.queue_position + 1, true);
		}
	}

	/// Spawns an async task to resolve a Subsonic stream URL.
	fn spawn_url_fetch(
		&mut self,
		api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
		index: usize,
		is_preload: bool,
	) {
		if is_preload {
			if self.fetching_preload_for == Some(index) {
				return;
			}
			self.fetching_preload_for = Some(index);
		} else {
			if self.fetching_current_for == Some(index) {
				return;
			}
			self.fetching_current_for = Some(index);
		}

		let track_id = self.queue[index].id.clone();
		let api_c = Arc::clone(api);
		let tx_c = tx.clone();

		tokio::spawn(async move {
			if let Some(provider) = api_c.read().await.as_ref() {
				if let Ok(url) = provider.get_stream_url(&track_id).await {
					let _ = tx_c.send(CoreMessage::UrlReady { url, index, is_preload });
				}
			}
		});
	}

	/// Checks if the requested art ID is different from current and triggers a load or fetch.
	fn update_art_state(&mut self, api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>, tx: &mpsc::UnboundedSender<CoreMessage>) {
		let art_id = self.queue.get(self.queue_position).and_then(|t| t.cover_art.clone());

		if art_id != self.current_art_id {
			self.current_art_id = art_id.clone();
			self.current_art = None;

			if let Some(id) = art_id {
				if let Some(img) = self.art_cache.load_image(&id) {
					self.current_art = Some(img);
				} else {
					let api_c = Arc::clone(api);
					let tx_c = tx.clone();
					let id_c = id.clone();
					tokio::spawn(async move {
						if let Some(provider) = api_c.read().await.as_ref() {
							if let Ok(bytes) = provider.get_cover_art_bytes(&id_c).await {
								let _ = tx_c.send(CoreMessage::ArtDownloaded { id: id_c, bytes });
							}
						}
					});
				}
			}
		}
	}

	/// Handles initialization tasks when a track begins playback.
	fn on_track_start(&mut self, api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>) {
		self.scrobble_tracker.reset();

		if let Some(track) = self.queue.get(self.queue_position) {
			let id = track.id.clone();
			let api_clone = Arc::clone(api);
			tokio::spawn(async move {
				if let Some(provider) = api_clone.read().await.as_ref() {
					let _ = provider.now_playing(&id).await;
				}
			});
		}
	}

	/// Calculates and starts a timer to trigger a scrobble once the track threshold is met.
	/// Pushes a scrobble entry to the manager and attempts to flush.
	fn trigger_scrobble(&mut self, api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>) {
		if let Some(track) = self.queue.get(self.queue_position) {
			self.scrobble_tracker.has_scrobbled = true;
			self.scrobble_manager
				.lock()
				.unwrap()
				.push(&track.id, self.scrobble_tracker.start_time);
			self.spawn_scrobble_flush(api);
		}
	}

	/// Spawns an async task to send pending scrobbles to the server.
	fn spawn_scrobble_flush(&self, api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>) {
		let sm = Arc::clone(&self.scrobble_manager);
		let api_c = Arc::clone(api);
		tokio::spawn(async move {
			let items = sm.lock().unwrap().get_all();
			if !items.is_empty() {
				if let Some(provider) = api_c.read().await.as_ref() {
					if provider.scrobble(items.clone()).await.is_ok() {
						sm.lock().unwrap().remove_flushed(items.len());
					}
				}
			}
		});
	}

	/// Updates the public core state and external integrations (MPRIS).
	fn dispatch_state(&mut self, p_state: &PlayerState, state_tx: &watch::Sender<CoreState>) {
		let track = self.queue.get(self.queue_position);
		let mark = track.and_then(|t| self.scrobble_tracker.get_mark_pos(t.duration_secs));

		#[cfg(feature = "os-media-controls")]
		if let Some(mpris) = &mut self.mpris {
			let art_path = self.current_art_id.as_ref().map(|id| self.art_cache.get_path(id));
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
}
