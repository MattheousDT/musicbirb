use super::{AudioBackend, PlayerState, PlayerStatus};
use crate::MusicbirbError;
use libmpv::Mpv;
use std::sync::Mutex;

pub struct MpvBackend {
	mpv: Mutex<Mpv>,
}

impl MpvBackend {
	pub fn new() -> Result<Self, MusicbirbError> {
		let mpv =
			Mpv::new().map_err(|e| MusicbirbError::Player(format!("Init failed: {:?}", e)))?;
		mpv.set_property("vo", "null")
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(Self {
			mpv: Mutex::new(mpv),
		})
	}
}

impl AudioBackend for MpvBackend {
	fn play(&self) -> Result<(), MusicbirbError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.set_property("pause", false)
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	fn pause(&self) -> Result<(), MusicbirbError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.set_property("pause", true)
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	fn toggle_pause(&self) -> Result<(), MusicbirbError> {
		let mpv = self.mpv.lock().unwrap();
		let paused: bool = mpv.get_property("pause").unwrap_or(false);
		mpv.set_property("pause", !paused)
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	fn stop(&self) -> Result<(), MusicbirbError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.command("stop", &[])
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	fn add(&self, url: &str) -> Result<(), MusicbirbError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.command("loadfile", &[url, "append"])
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	fn insert(&self, url: &str, index: i64) -> Result<(), MusicbirbError> {
		let mpv = self.mpv.lock().unwrap();
		// MPV doesn't have a direct `insert` command, so we append and move it
		mpv.command("loadfile", &[url, "append"])
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;

		let count: i64 = mpv.get_property("playlist-count").unwrap_or(0);
		if count > 0 {
			mpv.command(
				"playlist-move",
				&[&(count - 1).to_string(), &index.to_string()],
			)
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		}
		Ok(())
	}

	fn remove_index(&self, index: i64) -> Result<(), MusicbirbError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.command("playlist-remove", &[&index.to_string()])
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	fn clear_playlist(&self) -> Result<(), MusicbirbError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.command("playlist-clear", &[])
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	fn play_index(&self, index: i64) -> Result<(), MusicbirbError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.set_property("playlist-pos", index)
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	fn seek_relative(&self, seconds: f64) -> Result<(), MusicbirbError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.command("seek", &[&seconds.to_string(), "relative"])
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	fn seek_absolute(&self, seconds: f64) -> Result<(), MusicbirbError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.command("seek", &[&seconds.to_string(), "absolute"])
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	fn set_volume(&self, volume: f64) -> Result<(), MusicbirbError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.set_property("volume", volume)
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	fn get_volume(&self) -> Result<f64, MusicbirbError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.get_property("volume")
			.map_err(|e| MusicbirbError::Player(format!("{:?}", e)))
	}

	fn get_state(&self) -> PlayerState {
		let mpv = self.mpv.lock().unwrap();
		let idle: bool = mpv.get_property("idle-active").unwrap_or(true);
		let paused: bool = mpv.get_property("pause").unwrap_or(false);
		let time: f64 = mpv.get_property("time-pos").unwrap_or(0.0);
		let playlist_pos: i64 = mpv.get_property("playlist-pos").unwrap_or(-1);
		let playlist_count: i64 = mpv.get_property("playlist-count").unwrap_or(0);

		let status = if idle {
			PlayerStatus::Stopped
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
		}
	}
}
