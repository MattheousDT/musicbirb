use crate::api::SubsonicClient;
use crate::error::CoreError;
use crate::models::{AlbumId, CoverArtId, PlaylistId, Track, TrackId};
use crate::player::{Player, PlayerStatus};
use image::DynamicImage;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch};

#[derive(Debug, Clone)]
pub struct CoreState {
	pub queue: Vec<Track>,
	pub queue_position: usize,
	pub time: f64,
	pub status: PlayerStatus,
	pub current_art: Option<Arc<DynamicImage>>,
}

impl Default for CoreState {
	fn default() -> Self {
		Self {
			queue: Vec::new(),
			queue_position: 0,
			time: 0.0,
			status: PlayerStatus::Stopped,
			current_art: None,
		}
	}
}

pub enum CoreMessage {
	AddTracks(Vec<Track>),
	Next,
	Prev,
	SeekRelative(f64),
	TogglePause,
	UrlReady {
		url: String,
		index: usize,
	},
	ArtReady {
		art: Arc<DynamicImage>,
		id: CoverArtId,
	},
}

pub struct Musicbirb {
	api: Arc<SubsonicClient>,
	tx: mpsc::UnboundedSender<CoreMessage>,
	state_rx: watch::Receiver<CoreState>,
}

impl Musicbirb {
	pub fn new(api: SubsonicClient, player: Player) -> Arc<Self> {
		let (tx, rx) = mpsc::unbounded_channel();
		let (state_tx, state_rx) = watch::channel(CoreState::default());
		let api_arc = Arc::new(api);

		let core = Arc::new(Self {
			api: Arc::clone(&api_arc),
			tx: tx.clone(),
			state_rx,
		});

		core.start_actor_loop(player, rx, tx, state_tx, api_arc);
		core
	}

	fn start_actor_loop(
		&self,
		player: Player,
		mut rx: mpsc::UnboundedReceiver<CoreMessage>,
		tx: mpsc::UnboundedSender<CoreMessage>,
		state_tx: watch::Sender<CoreState>,
		api: Arc<SubsonicClient>,
	) {
		tokio::spawn(async move {
			let mut interval = tokio::time::interval(Duration::from_millis(33));
			let mut queue: Vec<Track> = Vec::new();
			let mut queue_position: usize = 0;

			let mut active_index: Option<usize> = None;
			let mut fetching_index: Option<usize> = None;

			let mut art_cache: HashMap<CoverArtId, Arc<DynamicImage>> = HashMap::new();
			let mut current_art_id: Option<CoverArtId> = None;
			let mut current_art: Option<Arc<DynamicImage>> = None;

			loop {
				tokio::select! {
						_ = interval.tick() => {
								let p_state = player.get_state();

								if p_state.status == PlayerStatus::Stopped && active_index == Some(queue_position) {
										if queue_position + 1 < queue.len() {
												queue_position += 1;
										}
								}

								if !queue.is_empty() && Some(queue_position) != active_index && Some(queue_position) != fetching_index {
										fetching_index = Some(queue_position);
										let track_id = queue[queue_position].id.clone();
										let api_clone = Arc::clone(&api);
										let tx_clone = tx.clone();
										let idx = queue_position;

										tokio::spawn(async move {
												if let Ok(url) = api_clone.get_stream_url(&track_id).await {
														let _ = tx_clone.send(CoreMessage::UrlReady { url, index: idx });
												}
										});
								}

								if let Some(track) = queue.get(queue_position) {
										if track.cover_art != current_art_id {
												current_art_id = track.cover_art.clone();
												current_art = None;

												if let Some(art_id) = &current_art_id {
														if let Some(cached) = art_cache.get(art_id) {
																current_art = Some(Arc::clone(cached));
														} else {
																let api_clone = Arc::clone(&api);
																let tx_clone = tx.clone();
																let aid = art_id.clone();

																tokio::spawn(async move {
																		if let Ok(bytes) = api_clone.get_cover_art_bytes(&aid).await {
																				if let Ok(img) = image::load_from_memory(&bytes) {
																						let _ = tx_clone.send(CoreMessage::ArtReady {
																								art: Arc::new(img),
																								id: aid
																						});
																				}
																		}
																});
														}
												}
										}
								}

								let _ = state_tx.send(CoreState {
										queue: queue.clone(),
										queue_position,
										time: p_state.position_secs,
										status: p_state.status,
										current_art: current_art.clone(),
								});
						}

						Some(msg) = rx.recv() => {
								match msg {
										CoreMessage::AddTracks(tracks) => {
												queue.extend(tracks);
										}
										CoreMessage::Next => {
												if queue_position + 1 < queue.len() {
														queue_position += 1;
														active_index = None;
														let _ = player.stop();
												}
										}
										CoreMessage::Prev => {
												if queue_position > 0 {
														queue_position -= 1;
														active_index = None;
														let _ = player.stop();
												} else {
														active_index = None;
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
												if Some(index) == fetching_index && index == queue_position {
														let _ = player.play(&url);
														active_index = Some(index);
														fetching_index = None;
												}
										}
										CoreMessage::ArtReady { art, id } => {
												art_cache.insert(id.clone(), Arc::clone(&art));
												if Some(id) == current_art_id {
														current_art = Some(art);
												}
										}
								}
						}
				}
			}
		});
	}

	pub async fn queue_track(&self, id: &TrackId) -> Result<(), CoreError> {
		let track = self.api.get_track(id).await?;
		self.tx
			.send(CoreMessage::AddTracks(vec![track]))
			.map_err(|_| CoreError::Internal("Core loop dead".into()))?;
		Ok(())
	}

	pub async fn queue_album(&self, id: &AlbumId) -> Result<usize, CoreError> {
		let tracks = self.api.get_album_tracks(id).await?;
		let count = tracks.len();
		self.tx
			.send(CoreMessage::AddTracks(tracks))
			.map_err(|_| CoreError::Internal("Core loop dead".into()))?;
		Ok(count)
	}

	pub async fn queue_playlist(&self, id: &PlaylistId) -> Result<usize, CoreError> {
		let tracks = self.api.get_playlist_tracks(id).await?;
		let count = tracks.len();
		self.tx
			.send(CoreMessage::AddTracks(tracks))
			.map_err(|_| CoreError::Internal("Core loop dead".into()))?;
		Ok(count)
	}

	pub fn next(&self) -> Result<(), CoreError> {
		self.tx
			.send(CoreMessage::Next)
			.map_err(|_| CoreError::Internal("Core loop dead".into()))
	}

	pub fn prev(&self) -> Result<(), CoreError> {
		self.tx
			.send(CoreMessage::Prev)
			.map_err(|_| CoreError::Internal("Core loop dead".into()))
	}

	pub fn seek(&self, seconds: f64) -> Result<(), CoreError> {
		self.tx
			.send(CoreMessage::SeekRelative(seconds))
			.map_err(|_| CoreError::Internal("Core loop dead".into()))
	}

	pub fn toggle_pause(&self) -> Result<(), CoreError> {
		self.tx
			.send(CoreMessage::TogglePause)
			.map_err(|_| CoreError::Internal("Core loop dead".into()))
	}

	pub fn subscribe(&self) -> watch::Receiver<CoreState> {
		self.state_rx.clone()
	}
}
