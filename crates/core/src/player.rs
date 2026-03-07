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

	pub fn play(&self, url: &str) -> Result<(), CoreError> {
		let mpv = self.mpv.lock().unwrap();
		mpv.command("loadfile", &[url])
			.map_err(|e| CoreError::Player(format!("{:?}", e)))?;
		mpv.set_property("pause", false)
			.map_err(|e| CoreError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	pub fn stop(&self) -> Result<(), CoreError> {
		self.mpv
			.lock()
			.unwrap()
			.command("stop", &[])
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
		self.mpv
			.lock()
			.unwrap()
			.command("seek", &[&seconds.to_string(), "relative"])
			.map_err(|e| CoreError::Player(format!("{:?}", e)))?;
		Ok(())
	}

	pub fn get_state(&self) -> PlayerState {
		let mpv = self.mpv.lock().unwrap();
		let idle: bool = mpv.get_property("idle-active").unwrap_or(true);
		let paused: bool = mpv.get_property("pause").unwrap_or(false);
		let time: f64 = mpv.get_property("time-pos").unwrap_or(0.0);

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
		}
	}
}
