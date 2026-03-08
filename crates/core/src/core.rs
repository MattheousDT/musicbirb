use crate::actor::CoreActor;
use crate::api::SubsonicClient;
use crate::error::CoreError;
use crate::models::{AlbumId, PlaylistId, TrackId};
use crate::player::Player;
use crate::state::{CoreMessage, CoreState};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

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

		let actor = CoreActor::new();
		let api_clone = Arc::clone(&api_arc);

		tokio::spawn(async move {
			actor.run(rx, tx, state_tx, api_clone, player).await;
		});

		core
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
