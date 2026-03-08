use crate::api::SubsonicClient;
use crate::models::{CoverArtId, Track};
use crate::player::{Player, PlayerState, PlayerStatus};
use crate::scrobble::ScrobbleManager;
use crate::state::{CoreMessage, CoreState};
use image::DynamicImage;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, watch};

pub struct CoreActor {
	queue: Vec<Track>,
	queue_position: usize,
	active_index: Option<usize>,
	fetching_index: Option<usize>,
	art_cache: HashMap<CoverArtId, Arc<DynamicImage>>,
	current_art_id: Option<CoverArtId>,
	current_art: Option<Arc<DynamicImage>>,
	scrobble_manager: Arc<Mutex<ScrobbleManager>>,
	accumulated_listen_time: f64,
	last_playhead_pos: f64,
	has_scrobbled: bool,
	track_start_time: u64,
	scrobble_flush_timer: Instant,
	last_tick_time: Instant,
}

impl Default for CoreActor {
	fn default() -> Self {
		Self::new()
	}
}

impl CoreActor {
	pub fn new() -> Self {
		Self {
			queue: Vec::new(),
			queue_position: 0,
			active_index: None,
			fetching_index: None,
			art_cache: HashMap::new(),
			current_art_id: None,
			current_art: None,
			scrobble_manager: Arc::new(Mutex::new(ScrobbleManager::new())),
			accumulated_listen_time: 0.0,
			last_playhead_pos: 0.0,
			has_scrobbled: false,
			track_start_time: 0,
			scrobble_flush_timer: Instant::now(),
			last_tick_time: Instant::now(),
		}
	}

	pub async fn run(
		mut self,
		mut rx: mpsc::UnboundedReceiver<CoreMessage>,
		tx: mpsc::UnboundedSender<CoreMessage>,
		state_tx: watch::Sender<CoreState>,
		api: Arc<SubsonicClient>,
		player: Player,
	) {
		let mut interval = tokio::time::interval(Duration::from_millis(33));

		loop {
			tokio::select! {
				_ = interval.tick() => {
					self.handle_tick(&player, &api, &tx, &state_tx);
				}
				Some(msg) = rx.recv() => {
					self.handle_message(msg, &player, &api);
				}
			}
		}
	}

	fn handle_tick(
		&mut self,
		player: &Player,
		api: &Arc<SubsonicClient>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
		state_tx: &watch::Sender<CoreState>,
	) {
		let now = Instant::now();
		let delta_time = now.duration_since(self.last_tick_time).as_secs_f64();
		self.last_tick_time = now;

		let p_state = player.get_state();
		let current_pos = p_state.position_secs;

		self.advance_queue_if_stopped(&p_state);
		self.fetch_url_if_needed(api, tx);
		self.fetch_art_if_needed(api, tx);
		self.update_listen_time(&p_state, current_pos, delta_time);

		let scrobble_mark_pos = self.handle_scrobbling(current_pos, api);
		self.flush_scrobbles_if_needed(api);

		let _ = state_tx.send(CoreState {
			queue: self.queue.clone(),
			queue_position: self.queue_position,
			time: p_state.position_secs,
			status: p_state.status,
			current_art: self.current_art.clone(),
			scrobble_mark_pos,
		});
	}

	fn advance_queue_if_stopped(&mut self, p_state: &PlayerState) {
		if p_state.status == PlayerStatus::Stopped && self.active_index == Some(self.queue_position)
		{
			if self.queue_position + 1 < self.queue.len() {
				self.queue_position += 1;
			}
		}
	}

	fn fetch_url_if_needed(
		&mut self,
		api: &Arc<SubsonicClient>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
	) {
		if !self.queue.is_empty()
			&& Some(self.queue_position) != self.active_index
			&& Some(self.queue_position) != self.fetching_index
		{
			self.fetching_index = Some(self.queue_position);
			let track_id = self.queue[self.queue_position].id.clone();
			let api_clone = Arc::clone(api);
			let tx_clone = tx.clone();
			let idx = self.queue_position;

			tokio::spawn(async move {
				if let Ok(url) = api_clone.get_stream_url(&track_id).await {
					let _ = tx_clone.send(CoreMessage::UrlReady { url, index: idx });
				}
			});
		}
	}

	fn fetch_art_if_needed(
		&mut self,
		api: &Arc<SubsonicClient>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
	) {
		if let Some(track) = self.queue.get(self.queue_position) {
			if track.cover_art != self.current_art_id {
				self.current_art_id = track.cover_art.clone();
				self.current_art = None;

				if let Some(art_id) = &self.current_art_id {
					if let Some(cached) = self.art_cache.get(art_id) {
						self.current_art = Some(Arc::clone(cached));
					} else {
						let api_clone = Arc::clone(api);
						let tx_clone = tx.clone();
						let aid = art_id.clone();

						tokio::spawn(async move {
							if let Ok(bytes) = api_clone.get_cover_art_bytes(&aid).await {
								if let Ok(img) = image::load_from_memory(&bytes) {
									let _ = tx_clone.send(CoreMessage::ArtReady {
										art: Arc::new(img),
										id: aid,
									});
								}
							}
						});
					}
				}
			}
		}
	}

	fn update_listen_time(&mut self, p_state: &PlayerState, current_pos: f64, delta_time: f64) {
		if p_state.status == PlayerStatus::Playing && self.active_index.is_some() {
			let pos_diff = current_pos - self.last_playhead_pos;
			if pos_diff > 0.0 && pos_diff <= delta_time + 1.0 {
				self.accumulated_listen_time += pos_diff;
			}
		}
		self.last_playhead_pos = current_pos;
	}

	fn handle_scrobbling(&mut self, current_pos: f64, api: &Arc<SubsonicClient>) -> Option<f64> {
		let mut scrobble_mark_pos = None;

		if let Some(idx) = self.active_index {
			if let Some(track) = self.queue.get(idx) {
				let duration = track.duration_secs as f64;
				if duration >= 30.0 {
					let threshold = (duration / 2.0).min(240.0);
					if !self.has_scrobbled {
						let remaining = threshold - self.accumulated_listen_time;
						let mark = current_pos + remaining;
						if mark <= duration + 1.0 {
							scrobble_mark_pos = Some(mark);
						}

						if self.accumulated_listen_time >= threshold {
							self.has_scrobbled = true;
							self.scrobble_manager
								.lock()
								.unwrap()
								.push(&track.id, self.track_start_time);
							self.spawn_scrobble_flush(api);
						}
					}
				}
			}
		}

		scrobble_mark_pos
	}

	fn flush_scrobbles_if_needed(&mut self, api: &Arc<SubsonicClient>) {
		if self.scrobble_flush_timer.elapsed().as_secs() >= 60 {
			self.scrobble_flush_timer = Instant::now();
			self.spawn_scrobble_flush(api);
		}
	}

	fn spawn_scrobble_flush(&self, api: &Arc<SubsonicClient>) {
		let sm = Arc::clone(&self.scrobble_manager);
		let api_clone = Arc::clone(api);
		tokio::spawn(async move {
			let items = sm.lock().unwrap().get_all();
			if !items.is_empty() {
				let count = items.len();
				if api_clone.scrobble(&items).await.is_ok() {
					sm.lock().unwrap().remove_flushed(count);
				}
			}
		});
	}

	fn handle_message(&mut self, msg: CoreMessage, player: &Player, api: &Arc<SubsonicClient>) {
		match msg {
			CoreMessage::AddTracks(tracks) => {
				self.queue.extend(tracks);
			}
			CoreMessage::Next => {
				if self.queue_position + 1 < self.queue.len() {
					self.queue_position += 1;
					self.active_index = None;
					let _ = player.stop();
				}
			}
			CoreMessage::Prev => {
				if self.queue_position > 0 {
					self.queue_position -= 1;
					self.active_index = None;
					let _ = player.stop();
				} else {
					self.active_index = None;
					let _ = player.stop();
				}
			}
			CoreMessage::SeekRelative(secs) => {
				let _ = player.seek_relative(secs);
			}
			CoreMessage::TogglePause => {
				let _ = player.toggle_pause();
			}
			CoreMessage::UrlReady { url, index } => {
				if Some(index) == self.fetching_index && index == self.queue_position {
					let _ = player.play(&url);
					self.active_index = Some(index);
					self.fetching_index = None;

					self.accumulated_listen_time = 0.0;
					self.last_playhead_pos = 0.0;
					self.has_scrobbled = false;
					self.track_start_time = SystemTime::now()
						.duration_since(UNIX_EPOCH)
						.unwrap_or_default()
						.as_millis() as u64;

					let api_clone = Arc::clone(api);
					let id_clone = self.queue[index].id.clone();
					tokio::spawn(async move {
						let _ = api_clone.now_playing(&id_clone).await;
					});
				}
			}
			CoreMessage::ArtReady { art, id } => {
				self.art_cache.insert(id.clone(), Arc::clone(&art));
				if Some(id) == self.current_art_id {
					self.current_art = Some(art);
				}
			}
		}
	}
}
