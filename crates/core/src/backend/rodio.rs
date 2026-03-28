use super::{AudioBackend, BackendEvent, PlayerState, PlayerStatus};
use crate::MusicbirbError;
use rodio::{DeviceSinkBuilder, Player};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio::sync::mpsc;

type HttpStream = stream_download::StreamDownload<stream_download::storage::temp::TempStorageProvider>;

/// Wraps our decoders securely avoiding dynamic dispatch complexity where not required.
pub enum AudioDecoder {
	Http(rodio::Decoder<HttpStream>),
	File(rodio::Decoder<std::fs::File>),
}

/// Commands for our internal background actor
enum RodioCommand {
	Play,
	Pause,
	TogglePause,
	Stop,
	Add(String),
	Insert(String, i64),
	RemoveIndex(i64),
	ClearPlaylist,
	LoadTrack {
		index: i64,
		is_preload: bool,
	},
	Seek(f64),
	SeekRelative(f64),
	SetVolume(f64),
	UpdateEventSender(mpsc::UnboundedSender<BackendEvent>),
	TrackLoaded {
		index: i64,
		decoder: AudioDecoder,
		is_preload: bool,
	},
	TrackLoadFailed {
		index: i64,
		is_preload: bool,
	},
}

#[derive(Debug, Clone)]
pub struct RodioState {
	playlist: Vec<String>,
	current_index: i64,
	queued_count: usize,
	last_preloaded_index: i64,
	status: PlayerStatus,
	volume: f64,
	track_start_pos: f64,
	seek_offset: f64,
}

pub struct RodioBackend {
	cmd_tx: mpsc::UnboundedSender<RodioCommand>,
	state: Arc<RwLock<RodioState>>,
	player: Arc<Player>,
}

impl RodioBackend {
	pub fn new() -> Result<Self, MusicbirbError> {
		let handle = DeviceSinkBuilder::open_default_sink()
			.map_err(|e| MusicbirbError::Player(format!("Rodio init failed: {:?}", e)))?;
		let player = Arc::new(Player::connect_new(handle.mixer()));

		let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<RodioCommand>();

		let state = Arc::new(RwLock::new(RodioState {
			playlist: Vec::new(),
			current_index: -1,
			queued_count: 0,
			last_preloaded_index: -1,
			status: PlayerStatus::Stopped,
			volume: 100.0,
			track_start_pos: 0.0,
			seek_offset: 0.0,
		}));

		let state_clone = Arc::clone(&state);
		let cmd_tx_clone = cmd_tx.clone();
		let player_loop = Arc::clone(&player);

		tokio::spawn(async move {
			let _handle = handle; // Required to keep the audio OS-stream alive!
			let mut event_tx: Option<mpsc::UnboundedSender<BackendEvent>> = None;
			let mut expected_load_index: i64 = -1;

			let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));

			loop {
				tokio::select! {
					cmd_opt = cmd_rx.recv() => {
						let Some(cmd) = cmd_opt else { break };
						match cmd {
							RodioCommand::UpdateEventSender(tx) => {
								event_tx = Some(tx);
							}
							RodioCommand::Play => {
								let mut s = state_clone.write().unwrap();
								if s.status == PlayerStatus::Paused {
									player_loop.play();
									s.status = PlayerStatus::Playing;
									if let Some(tx) = &event_tx {
										let _ = tx.send(BackendEvent::StatusUpdate(PlayerStatus::Playing));
									}
								} else if s.status == PlayerStatus::Stopped {
									if s.current_index >= 0 && (s.current_index as usize) < s.playlist.len() {
										let _ = cmd_tx_clone.send(RodioCommand::LoadTrack { index: s.current_index, is_preload: false });
									}
								}
							}
							RodioCommand::Pause => {
								player_loop.pause();
								let mut s = state_clone.write().unwrap();
								if s.status != PlayerStatus::Stopped {
									s.status = PlayerStatus::Paused;
									if let Some(tx) = &event_tx {
										let _ = tx.send(BackendEvent::StatusUpdate(PlayerStatus::Paused));
									}
								}
							}
							RodioCommand::TogglePause => {
								let mut s = state_clone.write().unwrap();
								if s.status == PlayerStatus::Playing {
									player_loop.pause();
									s.status = PlayerStatus::Paused;
									if let Some(tx) = &event_tx {
										let _ = tx.send(BackendEvent::StatusUpdate(PlayerStatus::Paused));
									}
								} else if s.status == PlayerStatus::Paused {
									player_loop.play();
									s.status = PlayerStatus::Playing;
									if let Some(tx) = &event_tx {
										let _ = tx.send(BackendEvent::StatusUpdate(PlayerStatus::Playing));
									}
								} else if s.status == PlayerStatus::Stopped {
									let _ = cmd_tx_clone.send(RodioCommand::Play);
								}
							}
							RodioCommand::Stop => {
								player_loop.clear();
								let mut s = state_clone.write().unwrap();
								s.status = PlayerStatus::Stopped;
								s.queued_count = 0;
								expected_load_index = -1;
								if let Some(tx) = &event_tx {
									let _ = tx.send(BackendEvent::StatusUpdate(PlayerStatus::Stopped));
								}
							}
							RodioCommand::Add(url) => {
								let mut s = state_clone.write().unwrap();
								s.playlist.push(url);
							}
							RodioCommand::Insert(url, index) => {
								let mut s = state_clone.write().unwrap();
								let idx = index as usize;
								if idx <= s.playlist.len() {
									s.playlist.insert(idx, url);
									if s.current_index >= index {
										s.current_index += 1;
										s.last_preloaded_index += 1;
									}
									if expected_load_index >= index {
										expected_load_index += 1;
									}
								} else {
									s.playlist.push(url);
								}
							}
							RodioCommand::RemoveIndex(index) => {
								let mut s = state_clone.write().unwrap();
								let idx = index as usize;
								if idx < s.playlist.len() {
									s.playlist.remove(idx);
									if s.current_index == index {
										let _ = cmd_tx_clone.send(RodioCommand::Stop);
									} else if s.current_index > index {
										s.current_index -= 1;
										s.last_preloaded_index -= 1;
									}
									if expected_load_index > index {
										expected_load_index -= 1;
									}
								}
							}
							RodioCommand::ClearPlaylist => {
								let mut s = state_clone.write().unwrap();
								s.playlist.clear();
								player_loop.clear();
								s.status = PlayerStatus::Stopped;
								s.current_index = -1;
								s.queued_count = 0;
								s.last_preloaded_index = -1;
								s.track_start_pos = 0.0;
								s.seek_offset = 0.0;
								expected_load_index = -1;
								if let Some(tx) = &event_tx {
									let _ = tx.send(BackendEvent::StatusUpdate(PlayerStatus::Stopped));
								}
							}
							RodioCommand::LoadTrack { index, is_preload } => {
								if !is_preload {
									expected_load_index = index;
									let mut s = state_clone.write().unwrap();
									s.status = PlayerStatus::Buffering;
									s.current_index = index;
									s.queued_count = 0;
									s.last_preloaded_index = index;
									s.seek_offset = 0.0;
									if let Some(tx) = &event_tx {
										let _ = tx.send(BackendEvent::StatusUpdate(PlayerStatus::Buffering));
									}
								}

								let url = {
									let s = state_clone.read().unwrap();
									if index >= 0 && (index as usize) < s.playlist.len() {
										Some(s.playlist[index as usize].clone())
									} else {
										None
									}
								};

								if let Some(url) = url {
									let tx_clone = cmd_tx_clone.clone();
									tokio::spawn(async move {
										if url.starts_with("http://") || url.starts_with("https://") {
											let url_parsed = match url.parse::<reqwest::Url>() {
												Ok(u) => u,
												Err(_) => {
													let _ = tx_clone.send(RodioCommand::TrackLoadFailed { index, is_preload });
													return;
												}
											};

											// Head request to reliably get Content-Length for Symphonia byte seeking.
											let content_length = if let Ok(res) = reqwest::Client::new().head(url_parsed.clone()).send().await {
												res.headers()
													.get(reqwest::header::CONTENT_LENGTH)
													.and_then(|v| v.to_str().ok())
													.and_then(|s| s.parse::<u64>().ok())
											} else {
												None
											};

											let reader = match stream_download::StreamDownload::new_http(
												url_parsed,
												stream_download::storage::temp::TempStorageProvider::new(),
												stream_download::Settings::default()
											).await {
												Ok(r) => r,
												Err(_) => {
													let _ = tx_clone.send(RodioCommand::TrackLoadFailed { index, is_preload });
													return;
												}
											};

											let decoder_res = tokio::task::spawn_blocking(move || {
												let mut builder = rodio::Decoder::builder()
													.with_data(reader)
													.with_seekable(true);

												if let Some(l) = content_length {
													builder = builder.with_byte_len(l);
												}

												builder.build().map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Decode error"))
											}).await.unwrap();

											match decoder_res {
												Ok(decoder) => {
													let _ = tx_clone.send(RodioCommand::TrackLoaded { index, decoder: AudioDecoder::Http(decoder), is_preload });
												}
												Err(_) => {
													let _ = tx_clone.send(RodioCommand::TrackLoadFailed { index, is_preload });
												}
											}
										} else {
											let url_path = url.strip_prefix("file://").unwrap_or(&url).to_string();
											let decoder_res = tokio::task::spawn_blocking(move || {
												let file = std::fs::File::open(url_path).map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;
												let len_opt = file.metadata().ok().map(|m| m.len());
												let mut builder = rodio::Decoder::builder()
													.with_data(file)
													.with_seekable(true);

												if let Some(l) = len_opt {
													builder = builder.with_byte_len(l);
												}

												builder.build().map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Decode error"))
											}).await.unwrap();

											match decoder_res {
												Ok(decoder) => {
													let _ = tx_clone.send(RodioCommand::TrackLoaded { index, decoder: AudioDecoder::File(decoder), is_preload });
												}
												Err(_) => {
													let _ = tx_clone.send(RodioCommand::TrackLoadFailed { index, is_preload });
												}
											}
										}
									});
								} else if !is_preload {
									player_loop.clear();
									let mut s = state_clone.write().unwrap();
									s.status = PlayerStatus::Stopped;
									if let Some(tx) = &event_tx {
										let _ = tx.send(BackendEvent::StatusUpdate(PlayerStatus::Stopped));
									}
								}
							}
							RodioCommand::Seek(seconds) => {
								let sec = if seconds < 0.0 { 0.0 } else { seconds };
								// Removed artificial volume muting as setting mixer volume instantly creates
								// pops/discontinuities. Seek directly.
								if player_loop.try_seek(std::time::Duration::from_secs_f64(sec)).is_ok() {
									let mut s = state_clone.write().unwrap();
									s.seek_offset = sec;
									s.track_start_pos = player_loop.get_pos().as_secs_f64();
								}
							}
							RodioCommand::SeekRelative(seconds) => {
								let current_pos = {
									let s = state_clone.read().unwrap();
									s.seek_offset + (player_loop.get_pos().as_secs_f64() - s.track_start_pos)
								};

								let mut new_pos = current_pos + seconds;
								if new_pos < 0.0 { new_pos = 0.0; }

								if player_loop.try_seek(std::time::Duration::from_secs_f64(new_pos)).is_ok() {
									let mut s = state_clone.write().unwrap();
									s.seek_offset = new_pos;
									s.track_start_pos = player_loop.get_pos().as_secs_f64();
								}
							}
							RodioCommand::SetVolume(volume) => {
								let rodio_vol = (volume / 100.0).clamp(0.0, 1.0) as f32;
								player_loop.set_volume(rodio_vol);
								let mut s = state_clone.write().unwrap();
								s.volume = volume;
							}
							RodioCommand::TrackLoaded { index, decoder, is_preload } => {
								if is_preload {
									let mut s = state_clone.write().unwrap();
									if index == s.last_preloaded_index && s.status != PlayerStatus::Stopped {
										match decoder {
											AudioDecoder::Http(d) => player_loop.append(d),
											AudioDecoder::File(d) => player_loop.append(d),
										}
										s.queued_count += 1;
									}
								} else {
									if index == expected_load_index {
										player_loop.clear();
										match decoder {
											AudioDecoder::Http(d) => player_loop.append(d),
											AudioDecoder::File(d) => player_loop.append(d),
										}

										let mut s = state_clone.write().unwrap();
										s.queued_count = 1;
										s.track_start_pos = player_loop.get_pos().as_secs_f64();
										s.seek_offset = 0.0;

										if let Some(tx) = &event_tx {
											let _ = tx.send(BackendEvent::TrackStarted);
										}

										// If user clicked play, or it was playing prior to buffering
										if s.status != PlayerStatus::Paused {
											player_loop.play();
											s.status = PlayerStatus::Playing;
											if let Some(tx) = &event_tx {
												let _ = tx.send(BackendEvent::StatusUpdate(PlayerStatus::Playing));
											}
										}
									}
								}
							}
							RodioCommand::TrackLoadFailed { index, is_preload } => {
								if !is_preload && index == expected_load_index {
									let mut s = state_clone.write().unwrap();
									s.status = PlayerStatus::Stopped;
									s.queued_count = 0;
									if let Some(tx) = &event_tx {
										let _ = tx.send(BackendEvent::StatusUpdate(PlayerStatus::Stopped));
									}
								}
							}
						}
					}
					_ = interval.tick() => {
						let mut s = state_clone.write().unwrap();

						if s.status == PlayerStatus::Playing {
							let q_len = player_loop.len();

							if q_len == 0 && s.queued_count > 0 {
								// Queue unexpectedly ran empty or tracks finished too fast
								if let Some(tx) = &event_tx {
									let _ = tx.send(BackendEvent::EndOfTrack);
								}
								s.current_index += 1;
								s.queued_count = 0;

								let next_index = s.current_index;
								let has_next = next_index >= 0 && (next_index as usize) < s.playlist.len();

								if has_next {
									let _ = cmd_tx_clone.send(RodioCommand::LoadTrack { index: next_index, is_preload: false });
								} else {
									s.status = PlayerStatus::Stopped;
									if let Some(tx) = &event_tx {
										let _ = tx.send(BackendEvent::StatusUpdate(PlayerStatus::Stopped));
									}
								}
							} else if q_len > 0 {
								// Track gapless transitions
								if q_len < s.queued_count {
									let finished_tracks = s.queued_count - q_len;
									for i in 0..finished_tracks {
										s.current_index += 1;
										// Snapshot accurately for position math, whether sink internally resets to 0 or not
										s.track_start_pos = player_loop.get_pos().as_secs_f64();
										s.seek_offset = 0.0;

										if let Some(tx) = &event_tx {
											let _ = tx.send(BackendEvent::EndOfTrack);
											// Only emit track started if we are transitioning into an actively playing track
											if q_len > 0 || i < finished_tracks - 1 {
												let _ = tx.send(BackendEvent::TrackStarted);
											}
										}
									}
									s.queued_count = q_len;
								}

								// Preload next track for gapless playback when we only have 1 active item
								if q_len == 1 {
									let next_index = s.current_index + 1;
									if next_index >= 0 && (next_index as usize) < s.playlist.len() && next_index != s.last_preloaded_index {
										s.last_preloaded_index = next_index;
										let _ = cmd_tx_clone.send(RodioCommand::LoadTrack { index: next_index, is_preload: true });
									}
								}

								if let Some(tx) = &event_tx {
									let pos = s.seek_offset + (player_loop.get_pos().as_secs_f64() - s.track_start_pos);
									let _ = tx.send(BackendEvent::PositionCorrection {
										seconds: pos.max(0.0),
										timestamp: Instant::now(),
									});
								}
							}
						}
					}
				}
			}
		});

		Ok(Self { cmd_tx, state, player })
	}

	fn send_cmd(&self, cmd: RodioCommand) -> Result<(), MusicbirbError> {
		self.cmd_tx
			.send(cmd)
			.map_err(|_| MusicbirbError::Player("Rodio backend task died".into()))
	}
}

#[macros::async_ffi]
impl AudioBackend for RodioBackend {
	fn set_event_sender(&self, tx: mpsc::UnboundedSender<BackendEvent>) {
		let _ = self.send_cmd(RodioCommand::UpdateEventSender(tx));
	}

	async fn play(&self) -> Result<(), MusicbirbError> {
		self.send_cmd(RodioCommand::Play)
	}

	async fn pause(&self) -> Result<(), MusicbirbError> {
		self.send_cmd(RodioCommand::Pause)
	}

	async fn toggle_pause(&self) -> Result<(), MusicbirbError> {
		self.send_cmd(RodioCommand::TogglePause)
	}

	async fn stop(&self) -> Result<(), MusicbirbError> {
		self.send_cmd(RodioCommand::Stop)
	}

	async fn add(&self, url: &str) -> Result<(), MusicbirbError> {
		self.send_cmd(RodioCommand::Add(url.to_string()))
	}

	async fn insert(&self, url: &str, index: i64) -> Result<(), MusicbirbError> {
		self.send_cmd(RodioCommand::Insert(url.to_string(), index))
	}

	async fn remove_index(&self, index: i64) -> Result<(), MusicbirbError> {
		self.send_cmd(RodioCommand::RemoveIndex(index))
	}

	async fn clear_playlist(&self) -> Result<(), MusicbirbError> {
		self.send_cmd(RodioCommand::ClearPlaylist)
	}

	async fn play_index(&self, index: i64) -> Result<(), MusicbirbError> {
		self.send_cmd(RodioCommand::LoadTrack {
			index,
			is_preload: false,
		})
	}

	async fn seek(&self, seconds: f64) -> Result<(), MusicbirbError> {
		self.send_cmd(RodioCommand::Seek(seconds))
	}

	async fn seek_relative(&self, seconds: f64) -> Result<(), MusicbirbError> {
		self.send_cmd(RodioCommand::SeekRelative(seconds))
	}

	async fn set_volume(&self, volume: f64) -> Result<(), MusicbirbError> {
		self.send_cmd(RodioCommand::SetVolume(volume))
	}

	async fn get_volume(&self) -> Result<f64, MusicbirbError> {
		let s = self.state.read().unwrap();
		Ok(s.volume)
	}

	fn get_state(&self) -> PlayerState {
		let s = self.state.read().unwrap();
		let pos = s.seek_offset + (self.player.get_pos().as_secs_f64() - s.track_start_pos);
		PlayerState {
			position_secs: pos.max(0.0),
			status: s.status,
			playlist_index: s.current_index,
			playlist_count: s.playlist.len() as i64,
			timestamp: Instant::now(),
		}
	}
}
