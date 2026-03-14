use super::{AudioBackend, BackendEvent, PlayerState, PlayerStatus};
use crate::MusicbirbError;
use async_trait::async_trait;
use libmpv::{
	Format, Mpv,
	events::{Event, PropertyData},
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

pub struct MpvBackend {
	mpv: Arc<Mpv>,
}

impl MpvBackend {
	pub fn new() -> Result<Self, MusicbirbError> {
		let mpv =
			Mpv::new().map_err(|e| MusicbirbError::Player(format!("Init failed: {:?}", e)))?;
		mpv.set_property("vo", "null")
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		// Make MPV go into idle state at the end of the playlist rather than quitting or pausing at EOF
		mpv.set_property("idle", "yes")
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		mpv.set_property("gapless-audio", "yes")
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;

		Ok(Self { mpv: Arc::new(mpv) })
	}
}

#[async_trait]
impl AudioBackend for MpvBackend {
	fn set_event_sender(&self, tx: mpsc::UnboundedSender<BackendEvent>) {
		let mpv_handle = Arc::clone(&self.mpv);

		std::thread::spawn(move || {
			let mut ev_ctx = mpv_handle.create_event_context();

			let _ = ev_ctx.observe_property("pause", Format::Flag, 1);
			let _ = ev_ctx.observe_property("time-pos", Format::Double, 2);
			let _ = ev_ctx.observe_property("idle-active", Format::Flag, 3);
			let _ = ev_ctx.observe_property("playlist-pos", Format::Int64, 4);

			loop {
				let ev = ev_ctx.wait_event(0.1);

				if tx.is_closed() {
					break;
				}

				match ev {
					Some(Ok(Event::StartFile)) => {
						let _ = tx.send(BackendEvent::TrackStarted);
					}
					Some(Ok(Event::PropertyChange { name, change, .. })) => match name {
						"pause" => {
							if let PropertyData::Flag(p) = change {
								let status = if p {
									PlayerStatus::Paused
								} else {
									PlayerStatus::Playing
								};
								let _ = tx.send(BackendEvent::StatusUpdate(status));
							}
						}
						"idle-active" => {
							if let PropertyData::Flag(idle) = change {
								if idle {
									let _ =
										tx.send(BackendEvent::StatusUpdate(PlayerStatus::Stopped));
								}
							}
						}
						"playlist-pos" => {
							if let PropertyData::Int64(pos) = change {
								if pos >= 0 {
									let _ = tx.send(BackendEvent::TrackStarted);
								}
							}
						}
						"time-pos" => {
							if let PropertyData::Double(pos) = change {
								let _ = tx.send(BackendEvent::PositionCorrection {
									seconds: pos,
									timestamp: Instant::now(),
								});
							}
						}
						_ => {}
					},
					Some(Ok(Event::EndFile(_))) => {
						let _ = tx.send(BackendEvent::EndOfTrack);
					}
					Some(Ok(Event::Shutdown)) => break,
					Some(Err(_)) => break,
					None => continue,
					_ => {}
				}
			}
		});
	}

	async fn play(&self) -> Result<(), MusicbirbError> {
		self.mpv
			.set_property("pause", false)
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	async fn pause(&self) -> Result<(), MusicbirbError> {
		self.mpv
			.set_property("pause", true)
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	async fn toggle_pause(&self) -> Result<(), MusicbirbError> {
		let paused: bool = self.mpv.get_property("pause").unwrap_or(false);
		self.mpv
			.set_property("pause", !paused)
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	async fn stop(&self) -> Result<(), MusicbirbError> {
		self.mpv
			.command("stop", &[])
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	async fn add(&self, url: &str) -> Result<(), MusicbirbError> {
		self.mpv
			.command("loadfile", &[url, "append"])
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	async fn insert(&self, url: &str, index: i64) -> Result<(), MusicbirbError> {
		self.mpv
			.command("loadfile", &[url, "append"])
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;

		let count: i64 = self.mpv.get_property("playlist-count").unwrap_or(0);
		if count > 0 {
			self.mpv
				.command(
					"playlist-move",
					&[&(count - 1).to_string(), &index.to_string()],
				)
				.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		}
		Ok(())
	}

	async fn remove_index(&self, index: i64) -> Result<(), MusicbirbError> {
		self.mpv
			.command("playlist-remove", &[&index.to_string()])
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	async fn clear_playlist(&self) -> Result<(), MusicbirbError> {
		self.mpv
			.command("playlist-clear", &[])
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	async fn play_index(&self, index: i64) -> Result<(), MusicbirbError> {
		self.mpv
			.set_property("playlist-pos", index)
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	async fn seek_relative(&self, seconds: f64) -> Result<(), MusicbirbError> {
		self.mpv
			.command("seek", &[&seconds.to_string(), "relative"])
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	async fn seek_absolute(&self, seconds: f64) -> Result<(), MusicbirbError> {
		self.mpv
			.command("seek", &[&seconds.to_string(), "absolute"])
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	async fn set_volume(&self, volume: f64) -> Result<(), MusicbirbError> {
		self.mpv
			.set_property("volume", volume)
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	async fn get_volume(&self) -> Result<f64, MusicbirbError> {
		self.mpv
			.get_property("volume")
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))
	}

	fn get_state(&self) -> PlayerState {
		let idle: bool = self.mpv.get_property("idle-active").unwrap_or(true);
		let paused: bool = self.mpv.get_property("pause").unwrap_or(false);
		let core_idle: bool = self.mpv.get_property("core-idle").unwrap_or(false);
		let seeking: bool = self.mpv.get_property("seeking").unwrap_or(false);
		let time: f64 = self.mpv.get_property("time-pos").unwrap_or(0.0);
		let playlist_pos: i64 = self.mpv.get_property("playlist-pos").unwrap_or(-1);
		let playlist_count: i64 = self.mpv.get_property("playlist-count").unwrap_or(0);

		let status = if idle {
			PlayerStatus::Stopped
		} else if core_idle || seeking {
			PlayerStatus::Buffering
		} else if paused {
			PlayerStatus::Paused
		} else {
			PlayerStatus::Playing
		};

		PlayerState {
			position_secs: time,
			status,
			playlist_index: playlist_pos,
			playlist_count,
			timestamp: Instant::now(),
		}
	}
}
