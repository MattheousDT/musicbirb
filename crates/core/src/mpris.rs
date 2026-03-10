use crate::backend::PlayerStatus;
use crate::models::Track;
use crate::state::CoreMessage;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct MprisManager {
	controls: MediaControls,
}

impl MprisManager {
	pub fn new(tx: mpsc::UnboundedSender<CoreMessage>) -> Option<Self> {
		let hwnd: Option<*mut std::ffi::c_void> = None;
		let config = PlatformConfig {
			dbus_name: "musicbirb",
			display_name: "Musicbirb",
			hwnd,
		};

		let mut controls = MediaControls::new(config).ok()?;
		controls
			.attach(move |event: MediaControlEvent| match event {
				MediaControlEvent::Play | MediaControlEvent::Pause | MediaControlEvent::Toggle => {
					let _ = tx.send(CoreMessage::TogglePause);
				}
				MediaControlEvent::Next => {
					let _ = tx.send(CoreMessage::Next);
				}
				MediaControlEvent::Previous => {
					let _ = tx.send(CoreMessage::Prev);
				}
				_ => {}
			})
			.ok()?;

		Some(Self { controls })
	}

	pub fn update_metadata(&mut self, track: &Track, mpris_art_path: Option<&Path>) {
		let mpris_art_url = mpris_art_path.map(|p| format!("file://{}", p.to_string_lossy()));

		let meta = MediaMetadata {
			title: Some(&track.title),
			artist: Some(&track.artist),
			album: Some(&track.album),
			duration: Some(Duration::from_secs(track.duration_secs as u64)),
			cover_url: mpris_art_url.as_deref(),
			..Default::default()
		};
		let _ = self.controls.set_metadata(meta);
	}

	pub fn set_playback_status(&mut self, status: PlayerStatus) {
		let pb = match status {
			PlayerStatus::Playing => MediaPlayback::Playing { progress: None },
			PlayerStatus::Paused => MediaPlayback::Paused { progress: None },
			PlayerStatus::Stopped => MediaPlayback::Stopped,
		};
		let _ = self.controls.set_playback(pb);
	}

	pub fn sync(
		&mut self,
		track: Option<&Track>,
		status: PlayerStatus,
		art_path: Option<&std::path::Path>,
	) {
		self.set_playback_status(status);
		if let Some(t) = track {
			self.update_metadata(t, art_path);
		}
	}
}
