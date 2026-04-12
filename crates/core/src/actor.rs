use crate::art_cache::ArtCache;
use crate::backend::{AudioBackend, BackendEvent, PlayerState, PlayerStatus};
use crate::models::{CoverArtId, RepeatMode, ReplayGainMode, ShuffleType, Track};
use crate::providers::Provider;
use crate::scrobble::{ScrobbleManager, ScrobbleTracker};
use crate::state::{CoreMessage, CoreState, PlaybackSync};
use image::DynamicImage;
use rand::seq::SliceRandom;
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
	replay_gain_mode: ReplayGainMode,

	repeat_mode: RepeatMode,
	shuffle: bool,
	shuffle_type: ShuffleType,
	consume: bool,
	stop_after_current: bool,

	shuffle_history: Vec<usize>,
	shuffle_unplayed: Vec<usize>,
	next_in_sequence: Option<usize>,

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
			replay_gain_mode: ReplayGainMode::Auto,

			repeat_mode: RepeatMode::None,
			shuffle: false,
			shuffle_type: ShuffleType::Smart,
			consume: false,
			stop_after_current: false,

			shuffle_history: Vec::new(),
			shuffle_unplayed: Vec::new(),
			next_in_sequence: None,

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
	fn sync_scrobble_duration(
		&mut self,
		current_sync: &PlaybackSync,
		api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>,
	) {
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

					if let Some(next_idx) = self.next_in_sequence {
						self.commit_transition(next_idx);
					} else {
						self.commit_transition(self.queue_position + 1);
					}

					self.player_has_current = true;
					self.player_has_next = false;

					if self.stop_after_current {
						self.stop_after_current = false;
					}

					self.update_art_state(api, tx);
					self.recalculate_next();
					self.check_preload(api, tx);
				}

				self.on_track_start(api);
				self.apply_replay_gain(player).await;
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

						let was_stop_after = self.stop_after_current;
						if was_stop_after {
							self.stop_after_current = false;
						}

						self.recalculate_next();

						if let Some(next_idx) = self.next_in_sequence {
							self.commit_transition(next_idx);
							self.play_track_at(self.queue_position, !was_stop_after, player, api, tx, state_tx)
								.await;
							return;
						} else {
							self.stop_playback(player, state_tx).await;
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
			CoreMessage::SetReplayGainMode(mode) => {
				self.replay_gain_mode = mode;
				self.apply_replay_gain(player).await;
			}
			CoreMessage::ProviderChanged => {
				self.queue.clear();
				self.shuffle_history.clear();
				self.shuffle_unplayed.clear();
				self.next_in_sequence = None;
				self.stop_playback(player, state_tx).await;
				self.update_art_state(api, tx);
			}
			CoreMessage::AddTracks(tracks, next) => {
				let was_empty = self.queue.is_empty();
				let new_count = tracks.len();

				if was_empty {
					self.queue.extend(tracks);
					if self.shuffle && self.shuffle_type == ShuffleType::Smart {
						self.shuffle_unplayed = (1..self.queue.len()).collect();
						self.shuffle_unplayed.shuffle(&mut rand::rng());
					}
					self.recalculate_next();
					self.play_track_at(0, true, player, api, tx, state_tx).await;
				} else {
					if next {
						let insert_pos = self.queue_position + 1;

						for i in &mut self.shuffle_history {
							if *i >= insert_pos {
								*i += new_count;
							}
						}
						for i in &mut self.shuffle_unplayed {
							if *i >= insert_pos {
								*i += new_count;
							}
						}

						let mut new_indices: Vec<usize> = (insert_pos..(insert_pos + new_count)).collect();
						if self.shuffle && self.shuffle_type == ShuffleType::Smart {
							new_indices.shuffle(&mut rand::rng());
							self.shuffle_unplayed.extend(new_indices);
						}

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
						let start_idx = self.queue.len();
						let mut new_indices: Vec<usize> = (start_idx..(start_idx + new_count)).collect();
						if self.shuffle && self.shuffle_type == ShuffleType::Smart {
							new_indices.shuffle(&mut rand::rng());
							self.shuffle_unplayed.extend(new_indices);
						}
						self.queue.extend(tracks);
					}

					self.recalculate_next();
					self.fix_preload(player, api, tx).await;
					self.dispatch_state(&player.get_state(), state_tx);
				}
			}
			CoreMessage::ReplaceTracks(tracks, start_index) => {
				self.queue = tracks;
				let idx = if start_index < self.queue.len() { start_index } else { 0 };

				self.shuffle_history.clear();
				if self.shuffle && self.shuffle_type == ShuffleType::Smart {
					self.shuffle_unplayed = (0..self.queue.len()).filter(|&i| i != idx).collect();
					self.shuffle_unplayed.shuffle(&mut rand::rng());
				}

				self.queue_position = idx;
				self.play_track_at(idx, true, player, api, tx, state_tx).await;
			}
			CoreMessage::ClearQueue => {
				self.queue.clear();
				self.shuffle_history.clear();
				self.shuffle_unplayed.clear();
				self.next_in_sequence = None;
				self.stop_playback(player, state_tx).await;
				self.update_art_state(api, tx);
			}
			CoreMessage::RemoveIndex(idx) => {
				if idx >= self.queue.len() {
					return;
				}
				let playing_removed = idx == self.queue_position;

				self.remove_index_internal(idx);

				if playing_removed {
					if self.queue.is_empty() {
						self.stop_playback(player, state_tx).await;
					} else {
						self.recalculate_next();
						let next = self.next_in_sequence.unwrap_or(self.queue_position);
						let actual = next.min(self.queue.len().saturating_sub(1));
						self.play_track_at(actual, true, player, api, tx, state_tx).await;
					}
				} else {
					self.recalculate_next();
					self.fix_preload(player, api, tx).await;
					self.dispatch_state(&player.get_state(), state_tx);
				}
			}
			CoreMessage::MoveIndex(from, to) => {
				if from >= self.queue.len() || to >= self.queue.len() {
					return;
				}
				if from == to {
					return;
				}

				let item = self.queue.remove(from);
				self.queue.insert(to, item);

				let adjust = |idx: &mut usize| {
					if *idx == from {
						*idx = to;
					} else if from < *idx && *idx <= to {
						*idx -= 1;
					} else if from > *idx && *idx >= to {
						*idx += 1;
					}
				};
				for i in &mut self.shuffle_history {
					adjust(i);
				}
				for i in &mut self.shuffle_unplayed {
					adjust(i);
				}

				if self.queue_position == from {
					self.queue_position = to;
				} else if from < self.queue_position && to >= self.queue_position {
					self.queue_position -= 1;
				} else if from > self.queue_position && to <= self.queue_position {
					self.queue_position += 1;
				}

				self.recalculate_next();
				self.fix_preload(player, api, tx).await;
				self.dispatch_state(&player.get_state(), state_tx);
			}
			CoreMessage::Next => {
				let next_idx = self.compute_next_index(true);
				if let Some(idx) = next_idx {
					self.commit_transition(idx);
					self.play_track_at(self.queue_position, true, player, api, tx, state_tx)
						.await;
				} else if !self.queue.is_empty() {
					self.commit_transition(0);
					self.play_track_at(self.queue_position, false, player, api, tx, state_tx)
						.await;
				}
			}
			CoreMessage::Prev => {
				if self.shuffle && self.shuffle_type == ShuffleType::Smart && !self.shuffle_history.is_empty() {
					let prev_idx = self.shuffle_history.pop().unwrap();
					self.shuffle_unplayed.push(self.queue_position);
					self.queue_position = prev_idx;
					self.play_track_at(self.queue_position, true, player, api, tx, state_tx)
						.await;
				} else {
					if self.queue_position > 0 {
						self.queue_position -= 1;
						self.play_track_at(self.queue_position, true, player, api, tx, state_tx)
							.await;
					} else if !self.queue.is_empty() {
						if self.repeat_mode == RepeatMode::All {
							self.queue_position = self.queue.len() - 1;
							self.play_track_at(self.queue_position, true, player, api, tx, state_tx)
								.await;
						} else {
							self.play_track_at(0, true, player, api, tx, state_tx).await;
						}
					}
				}
			}
			CoreMessage::PlayIndex(idx) => {
				if idx < self.queue.len() {
					self.queue_position = idx;
					self.play_track_at(idx, true, player, api, tx, state_tx).await;
				}
			}
			CoreMessage::Seek(secs) => {
				let _ = player.seek(secs).await;
				self.dispatch_state(&player.get_state(), state_tx);
			}
			CoreMessage::SeekRelative(secs) => {
				let _ = player.seek_relative(secs).await;
				self.dispatch_state(&player.get_state(), state_tx);
			}
			CoreMessage::Play => {
				let _ = player.play().await;
			}
			CoreMessage::Pause => {
				let _ = player.pause().await;
			}
			CoreMessage::TogglePause => {
				let _ = player.toggle_pause().await;
			}
			CoreMessage::SetRepeatMode(mode) => {
				self.repeat_mode = mode;
				self.recalculate_next();
				self.fix_preload(player, api, tx).await;
				self.dispatch_state(&player.get_state(), state_tx);
			}
			CoreMessage::SetShuffle(s) => {
				if self.shuffle != s {
					self.shuffle = s;
					if s && self.shuffle_type == ShuffleType::Smart {
						self.shuffle_history.clear();
						self.shuffle_unplayed = (0..self.queue.len()).filter(|&i| i != self.queue_position).collect();
						self.shuffle_unplayed.shuffle(&mut rand::rng());
					}
					self.recalculate_next();
					self.fix_preload(player, api, tx).await;
					self.dispatch_state(&player.get_state(), state_tx);
				}
			}
			CoreMessage::SetShuffleType(st) => {
				if self.shuffle_type != st {
					self.shuffle_type = st;
					if self.shuffle && st == ShuffleType::Smart {
						self.shuffle_history.clear();
						self.shuffle_unplayed = (0..self.queue.len()).filter(|&i| i != self.queue_position).collect();
						self.shuffle_unplayed.shuffle(&mut rand::rng());
					}
					self.recalculate_next();
					self.fix_preload(player, api, tx).await;
					self.dispatch_state(&player.get_state(), state_tx);
				}
			}
			CoreMessage::SetConsume(c) => {
				self.consume = c;
				self.dispatch_state(&player.get_state(), state_tx);
			}
			CoreMessage::SetStopAfterCurrent(s) => {
				self.stop_after_current = s;
				self.recalculate_next();
				self.fix_preload(player, api, tx).await;
				self.dispatch_state(&player.get_state(), state_tx);
			}
			CoreMessage::UrlReady { url, index, is_preload } => {
				if is_preload {
					if self.fetching_preload_for == Some(index) {
						self.fetching_preload_for = None;
					}
					// Verify this is still the correct next track (avoid race conditions from rapidly pressing next)
					if Some(index) == self.next_in_sequence {
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

	async fn stop_playback(&mut self, player: &Arc<dyn AudioBackend>, state_tx: &watch::Sender<CoreState>) {
		self.queue_position = 0;
		self.player_has_current = false;
		self.player_has_next = false;
		self.auto_play = false;
		self.fetching_current_for = None;
		self.fetching_preload_for = None;

		let _ = player.stop().await;
		let _ = player.clear_playlist().await;
		let mut p_state = player.get_state();
		p_state.status = PlayerStatus::Stopped;
		self.dispatch_state(&p_state, state_tx);
	}

	/// Clears the player out and manually forces playback at a specific target index
	async fn play_track_at(
		&mut self,
		index: usize,
		auto_play: bool,
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
		self.auto_play = auto_play;

		self.recalculate_next();

		let _ = player.stop().await;
		let _ = player.clear_playlist().await;

		if index < self.queue.len() {
			self.spawn_url_fetch(api, tx, index, false);
		}

		self.update_art_state(api, tx);
		self.dispatch_state(&player.get_state(), state_tx);
	}

	/// Checks to see if there is an empty index trailing the current actively playing track in the backend
	fn recalculate_next(&mut self) {
		self.next_in_sequence = self.compute_next_index(false);
	}

	fn compute_next_index(&mut self, force_advance: bool) -> Option<usize> {
		if self.queue.is_empty() {
			return None;
		}
		if self.stop_after_current && !force_advance {
			return None;
		}
		if self.repeat_mode == RepeatMode::One && !force_advance {
			return Some(self.queue_position);
		}

		if self.shuffle {
			match self.shuffle_type {
				ShuffleType::Random => Some(rand::random_range(0..self.queue.len())),
				ShuffleType::Smart => {
					if self.shuffle_unplayed.is_empty() {
						if self.repeat_mode == RepeatMode::All || force_advance {
							self.shuffle_unplayed =
								(0..self.queue.len()).filter(|&i| i != self.queue_position).collect();
							self.shuffle_unplayed.shuffle(&mut rand::rng());
						} else {
							return None;
						}
					}
					self.shuffle_unplayed.last().copied()
				}
			}
		} else {
			if self.queue_position + 1 < self.queue.len() {
				Some(self.queue_position + 1)
			} else if self.repeat_mode == RepeatMode::All || force_advance {
				Some(0)
			} else {
				None
			}
		}
	}

	fn commit_transition(&mut self, mut new_idx: usize) {
		if self.shuffle && self.shuffle_type == ShuffleType::Smart {
			self.shuffle_history.push(self.queue_position);
			if let Some(pos) = self.shuffle_unplayed.iter().position(|&i| i == new_idx) {
				self.shuffle_unplayed.remove(pos);
			}
		}

		if self.consume {
			let old_pos = self.queue_position;
			self.remove_index_internal(old_pos);
			if new_idx > old_pos {
				new_idx -= 1;
			}
		}
		self.queue_position = new_idx;
	}

	fn remove_index_internal(&mut self, idx: usize) {
		self.queue.remove(idx);
		self.shuffle_history.retain(|&i| i != idx);
		for i in &mut self.shuffle_history {
			if *i > idx {
				*i -= 1;
			}
		}

		self.shuffle_unplayed.retain(|&i| i != idx);
		for i in &mut self.shuffle_unplayed {
			if *i > idx {
				*i -= 1;
			}
		}

		if self.queue_position > idx {
			self.queue_position -= 1;
		}
	}

	async fn fix_preload(
		&mut self,
		player: &Arc<dyn AudioBackend>,
		api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
	) {
		if self.player_has_next {
			// Try to evict if we no longer want it or it changed
			let _ = player.remove_index(1).await;
			self.player_has_next = false;
			self.fetching_preload_for = None;
		}
		self.check_preload(api, tx);
	}

	fn check_preload(
		&mut self,
		api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
	) {
		if self.player_has_current && !self.player_has_next {
			if let Some(next_idx) = self.next_in_sequence {
				if !self.stop_after_current {
					self.spawn_url_fetch(api, tx, next_idx, true);
				}
			}
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
			if let Some(provider) = api_c.read().await.clone() {
				if let Ok(url) = provider.media().get_stream_url(&track_id).await {
					let _ = tx_c.send(CoreMessage::UrlReady { url, index, is_preload });
				}
			}
		});
	}

	fn update_art_state(
		&mut self,
		api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
	) {
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
						if let Some(provider) = api_c.read().await.clone() {
							if let Ok(bytes) = provider.media().get_cover_art_bytes(&id_c).await {
								let _ = tx_c.send(CoreMessage::ArtDownloaded { id: id_c, bytes });
							}
						}
					});
				}
			}
		}
	}

	/// Calculates and applies the ReplayGain volume multiplier based on the current settings and track.
	async fn apply_replay_gain(&self, player: &Arc<dyn AudioBackend>) {
		let gain = if let Some(track) = self.queue.get(self.queue_position) {
			let rg = track.replay_gain.as_ref();
			match self.replay_gain_mode {
				ReplayGainMode::Disabled => 0.0,
				ReplayGainMode::Track => rg
					.and_then(|r| r.track_gain)
					.or_else(|| rg.and_then(|r| r.album_gain))
					.unwrap_or(0.0),
				ReplayGainMode::Album => rg
					.and_then(|r| r.album_gain)
					.or_else(|| rg.and_then(|r| r.track_gain))
					.unwrap_or(0.0),
				ReplayGainMode::Auto => {
					let is_consecutive = if self.queue_position > 0 {
						if let Some(prev) = self.queue.get(self.queue_position - 1) {
							!track.album.is_empty() && track.album == prev.album
						} else {
							false
						}
					} else {
						false
					};

					if is_consecutive {
						rg.and_then(|r| r.album_gain)
							.or_else(|| rg.and_then(|r| r.track_gain))
							.unwrap_or(0.0)
					} else {
						rg.and_then(|r| r.track_gain)
							.or_else(|| rg.and_then(|r| r.album_gain))
							.unwrap_or(0.0)
					}
				}
			}
		} else {
			0.0
		};

		let factor = 10.0_f32.powf(gain / 20.0) as f64;
		let _ = player.set_volume(100.0 * factor).await;
	}

	/// Handles initialization tasks when a track begins playback.
	fn on_track_start(&mut self, api: &Arc<tokio::sync::RwLock<Option<Arc<dyn Provider>>>>) {
		self.scrobble_tracker.reset();

		if let Some(track) = self.queue.get(self.queue_position) {
			let id = track.id.clone();
			let api_clone = Arc::clone(api);
			tokio::spawn(async move {
				if let Some(provider) = api_clone.read().await.clone() {
					let _ = provider.activity().now_playing(&id).await;
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
				if let Some(provider) = api_c.read().await.clone() {
					if provider.activity().scrobble(items.clone()).await.is_ok() {
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
			repeat_mode: self.repeat_mode,
			shuffle: self.shuffle,
			shuffle_type: self.shuffle_type,
			consume: self.consume,
			stop_after_current: self.stop_after_current,
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::{RepeatMode, TrackId};

	fn mock_track(id: &str) -> Track {
		Track {
			id: TrackId(id.to_string()),
			..Default::default()
		}
	}

	#[tokio::test]
	async fn test_compute_next_index_normal() {
		let mut actor = CoreActor::new(None, None);
		actor.queue = vec![mock_track("1"), mock_track("2"), mock_track("3")];
		actor.queue_position = 0;

		assert_eq!(actor.compute_next_index(false), Some(1));
		actor.queue_position = 2;
		assert_eq!(actor.compute_next_index(false), None);
		actor.repeat_mode = RepeatMode::All;
		assert_eq!(actor.compute_next_index(false), Some(0));
	}

	#[tokio::test]
	async fn test_compute_next_index_repeat_one() {
		let mut actor = CoreActor::new(None, None);
		actor.queue = vec![mock_track("1"), mock_track("2"), mock_track("3")];
		actor.queue_position = 1;
		actor.repeat_mode = RepeatMode::One;

		assert_eq!(actor.compute_next_index(false), Some(1));
		assert_eq!(actor.compute_next_index(true), Some(2)); // Force advance test
	}

	#[tokio::test]
	async fn test_stop_after_current() {
		let mut actor = CoreActor::new(None, None);
		actor.queue = vec![mock_track("1"), mock_track("2"), mock_track("3")];
		actor.queue_position = 0;
		actor.stop_after_current = true;

		assert_eq!(actor.compute_next_index(false), None);
	}
}
