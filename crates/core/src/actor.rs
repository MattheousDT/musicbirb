use crate::api::SubsonicClient;
use crate::art_cache::ArtCache;
use crate::models::{CoverArtId, Track};
use crate::player::{Player, PlayerState, PlayerStatus};
use crate::scrobble::{ScrobbleManager, ScrobbleTracker};
use crate::state::{CoreMessage, CoreState};
use image::DynamicImage;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, watch};

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
	scrobble_flush_timer: Instant,

	last_tick_time: Instant,

	#[cfg(feature = "os-media-controls")]
	mpris: Option<MprisManager>,
}

impl CoreActor {
	pub fn new() -> Self {
		Self {
			queue: Vec::new(),
			queue_position: 0,
			active_index: None,
			fetching_index: None,
			preloading_index: None,

			art_cache: ArtCache::new(),
			current_art_id: None,
			current_art: None,

			scrobble_tracker: ScrobbleTracker::new(),
			scrobble_manager: Arc::new(Mutex::new(ScrobbleManager::new())),
			scrobble_flush_timer: Instant::now(),

			last_tick_time: Instant::now(),

			#[cfg(feature = "os-media-controls")]
			mpris: None,
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
		#[cfg(feature = "os-media-controls")]
		{
			self.mpris = MprisManager::new(tx.clone());
		}

		let mut interval = tokio::time::interval(Duration::from_millis(33));

		loop {
			tokio::select! {
				_ = interval.tick() => self.tick(&player, &api, &tx, &state_tx),
				Some(msg) = rx.recv() => self.handle_message(msg, &player, &api),
			}
		}
	}

	fn tick(
		&mut self,
		player: &Player,
		api: &Arc<SubsonicClient>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
		state_tx: &watch::Sender<CoreState>,
	) {
		let now = Instant::now();
		let delta = now.duration_since(self.last_tick_time).as_secs_f64();
		self.last_tick_time = now;

		let p_state = player.get_state();

		self.sync_playback_engine(&p_state, player, api);

		self.sync_resources(api, tx, &p_state);

		self.scrobble_tracker.update(
			p_state.position_secs,
			delta,
			p_state.status == PlayerStatus::Playing,
		);
		let mark = self.update_scrobbles(api);

		self.dispatch_state(&p_state, state_tx, mark);
	}

	fn sync_playback_engine(
		&mut self,
		p_state: &PlayerState,
		player: &Player,
		api: &Arc<SubsonicClient>,
	) {
		let has_next = self.queue_position + 1 < self.queue.len();

		if p_state.status == PlayerStatus::Stopped {
			if self.active_index == Some(self.queue_position) && has_next {
				self.queue_position += 1;
				self.active_index = None;
			}
		} else if p_state.playlist_index > 0 && has_next {
			self.queue_position += 1;
			self.active_index = Some(self.queue_position);
			self.preloading_index = None;
			self.on_track_start(api);
			let _ = player.remove_index(0);
		}
	}

	fn sync_resources(
		&mut self,
		api: &Arc<SubsonicClient>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
		p_state: &PlayerState,
	) {
		if self.queue.is_empty() {
			return;
		}

		if self.active_index != Some(self.queue_position)
			&& self.fetching_index != Some(self.queue_position)
		{
			self.fetching_index = Some(self.queue_position);
			self.spawn_url_fetch(api, tx, self.queue_position, false);
		} else if p_state.playlist_count < 2
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

	fn update_art_state(
		&mut self,
		art_id: Option<CoverArtId>,
		api: &Arc<SubsonicClient>,
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

	fn update_scrobbles(&mut self, api: &Arc<SubsonicClient>) -> Option<f64> {
		let track = self.active_index.and_then(|i| self.queue.get(i))?;
		let (ready, mark) = self.scrobble_tracker.check_threshold(track.duration_secs);

		if ready && !self.scrobble_tracker.has_scrobbled {
			self.scrobble_tracker.has_scrobbled = true;
			self.scrobble_manager
				.lock()
				.unwrap()
				.push(&track.id, self.scrobble_tracker.start_time);
			self.spawn_scrobble_flush(api);
		}

		if self.scrobble_flush_timer.elapsed().as_secs() >= 60 {
			self.scrobble_flush_timer = Instant::now();
			self.spawn_scrobble_flush(api);
		}
		mark
	}

	fn dispatch_state(
		&mut self,
		p_state: &PlayerState,
		state_tx: &watch::Sender<CoreState>,
		scrobble_mark: Option<f64>,
	) {
		#[cfg(feature = "os-media-controls")]
		if let Some(mpris) = &mut self.mpris {
			let track = self.active_index.and_then(|i| self.queue.get(i));
			let art_path = self
				.current_art_id
				.as_ref()
				.map(|id| self.art_cache.get_path(id));
			mpris.sync(track, p_state.status, art_path.as_deref());
		}

		let _ = state_tx.send(CoreState {
			queue: self.queue.clone(),
			queue_position: self.queue_position,
			time: p_state.position_secs,
			status: p_state.status,
			current_art: self.current_art.clone(),
			scrobble_mark_pos: scrobble_mark,
		});
	}

	fn on_track_start(&mut self, api: &Arc<SubsonicClient>) {
		self.scrobble_tracker.reset();
		if let Some(track) = self.queue.get(self.queue_position) {
			let id = track.id.clone();
			let api_clone = Arc::clone(api);
			tokio::spawn(async move {
				let _ = api_clone.now_playing(&id).await;
			});
		}
	}

	fn spawn_url_fetch(
		&self,
		api: &Arc<SubsonicClient>,
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

	fn spawn_art_fetch(
		&self,
		id: &CoverArtId,
		api: &Arc<SubsonicClient>,
		tx: &mpsc::UnboundedSender<CoreMessage>,
	) {
		let (api_c, tx_c, id_c) = (Arc::clone(api), tx.clone(), id.clone());
		tokio::spawn(async move {
			if let Ok(bytes) = api_c.get_cover_art_bytes(&id_c).await {
				let _ = tx_c.send(CoreMessage::ArtDownloaded { id: id_c, bytes });
			}
		});
	}

	fn spawn_scrobble_flush(&self, api: &Arc<SubsonicClient>) {
		let (sm, api_c) = (Arc::clone(&self.scrobble_manager), Arc::clone(api));
		tokio::spawn(async move {
			let items = sm.lock().unwrap().get_all();
			if !items.is_empty() && api_c.scrobble(&items).await.is_ok() {
				sm.lock().unwrap().remove_flushed(items.len());
			}
		});
	}

	fn handle_message(&mut self, msg: CoreMessage, player: &Player, api: &Arc<SubsonicClient>) {
		match msg {
			CoreMessage::AddTracks(tracks) => self.queue.extend(tracks),
			CoreMessage::Next | CoreMessage::Prev => {
				let next = matches!(msg, CoreMessage::Next);
				let possible = if next {
					self.queue_position + 1 < self.queue.len()
				} else {
					self.queue_position > 0
				};
				if possible {
					self.queue_position = if next {
						self.queue_position + 1
					} else {
						self.queue_position - 1
					};
					self.active_index = None;
					self.fetching_index = None;
					self.preloading_index = None;
					let _ = player.stop();
				}
			}
			CoreMessage::SeekRelative(secs) => {
				let _ = player.seek_relative(secs);
			}
			CoreMessage::TogglePause => {
				let _ = player.toggle_pause();
			}
			CoreMessage::UrlReady {
				url,
				index,
				is_preload,
			} => {
				if is_preload && Some(index) == self.preloading_index {
					let _ = player.play(&url, false);
				} else if !is_preload && Some(index) == self.fetching_index {
					let _ = player.play(&url, true);
					self.active_index = Some(index);
					self.fetching_index = None;
					self.on_track_start(api);
				}
			}
			CoreMessage::ArtDownloaded { id, bytes } => {
				if let Some(art) = self.art_cache.save_and_load(&id, &bytes) {
					if Some(id) == self.current_art_id {
						self.current_art = Some(art);
					}
				}
			}
			_ => {}
		}
	}
}
