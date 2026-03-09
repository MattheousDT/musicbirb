use crate::error::CoreError;
use libmpv::Mpv;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStatus {
	Stopped,
	Playing,
	Paused,
}

#[derive(Debug, Clone)]
pub struct PlayerState {
	pub position_secs: f64,
	pub status: PlayerStatus,
	pub playlist_index: i64,
	pub playlist_count: i64,
}

pub struct Player {
	mpv: Mutex<Mpv>,
}

impl Player {
	pub fn new() -> Result<Self, CoreError> {
		let mpv = Mpv::new().map_err(|e| CoreError::Player(format!("Init failed: {:?}", e)))?;
		mpv.set_property("vo", "null")
			.map_err(|e| CoreError::Player(format!("{:?}", e)))?;
		Ok(Self {
			mpv: Mutex::new(mpv),
		})
	}

	pub fn play(&self, url: &str, replace: bool) -> Result<(), CoreError> {
		let mpv = self.mpv.lock().unwrap();
		let mode = if replace { "replace" } else { "append" };
		mpv.command("loadfile", &[url, mode])
			.map_err(|e| CoreError::Player(format!("{:?}", e)))?;

		if replace {
			mpv.set_property("pause", false)
				.map_err(|e| CoreError::Player(format!("{:?}", e)))?;
		}
		Ok(())
	}

	pub fn stop(&self) -> Result<(), CoreError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.command("stop", &[])
			.map_err(|e| CoreError::Player(format!("{:?}", e)))?;
		mpv.command("playlist-clear", &[])
			.map_err(|e| CoreError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	pub fn remove_index(&self, index: i64) -> Result<(), CoreError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.command("playlist-remove", &[&index.to_string()])
			.map_err(|e| CoreError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	pub fn toggle_pause(&self) -> Result<(), CoreError> {
		let mpv = self.mpv.lock().unwrap();
		let paused: bool = mpv.get_property("pause").unwrap_or(false);
		mpv.set_property("pause", !paused)
			.map_err(|e| CoreError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	pub fn seek_relative(&self, seconds: f64) -> Result<(), CoreError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.command("seek", &[&seconds.to_string(), "relative"])
			.map_err(|e| CoreError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	pub fn get_state(&self) -> PlayerState {
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
