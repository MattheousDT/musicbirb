use crate::actor::CoreActor;
use crate::api::subsonic::SubsonicClient;
use crate::backend::AudioBackend;
use crate::error::MusicbirbError;
use crate::models::{Album, AlbumId, Playlist, PlaylistId, TrackId};
use crate::state::{CoreMessage, CoreState};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

pub struct Musicbirb {
	pub api: Arc<SubsonicClient>,
	tx: mpsc::UnboundedSender<CoreMessage>,
	state_rx: watch::Receiver<CoreState>,
}

impl Musicbirb {
	pub fn new(api: SubsonicClient, player: Arc<dyn AudioBackend>) -> Arc<Self> {
		Self::with_paths(api, player, None, None)
	}

	pub fn with_paths(
		api: SubsonicClient,
		player: Arc<dyn AudioBackend>,
		// I shouldn't need these but Android paths are brokey in rust
		// Prop-drill baby drill!
		data_dir: Option<PathBuf>,
		cache_dir: Option<PathBuf>,
	) -> Arc<Self> {
		let (tx, rx) = mpsc::unbounded_channel();
		let (state_tx, state_rx) = watch::channel(CoreState::default());
		let api_arc = Arc::new(api);

		let core = Arc::new(Self {
			api: Arc::clone(&api_arc),
			tx: tx.clone(),
			state_rx,
		});

		let actor = CoreActor::new(data_dir, cache_dir);
		let api_clone = Arc::clone(&api_arc);
		let tx_clone = tx.clone();

		tokio::spawn(async move {
			actor.run(rx, tx_clone, state_tx, api_clone, player).await;
		});

		core
	}

	pub fn shutdown(&self) {
		let _ = self.tx.send(CoreMessage::Shutdown);
	}

	pub async fn queue_track(&self, id: &TrackId) -> Result<(), MusicbirbError> {
		let track = self.api.get_track(id).await?;
		self.tx
			.send(CoreMessage::AddTracks(vec![track]))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
		Ok(())
	}

	pub async fn queue_album(&self, id: &AlbumId) -> Result<usize, MusicbirbError> {
		let tracks = self.api.get_album_tracks(id).await?;
		let count = tracks.len();
		self.tx
			.send(CoreMessage::AddTracks(tracks))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
		Ok(count)
	}

	pub async fn queue_playlist(&self, id: &PlaylistId) -> Result<usize, MusicbirbError> {
		let tracks = self.api.get_playlist_tracks(id).await?;
		let count = tracks.len();
		self.tx
			.send(CoreMessage::AddTracks(tracks))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))?;
		Ok(count)
	}

	pub async fn get_last_played_albums(&self) -> Result<Vec<Album>, MusicbirbError> {
		self.api.get_last_played_albums().await
	}

	pub async fn get_recently_added_albums(&self) -> Result<Vec<Album>, MusicbirbError> {
		self.api.get_recently_added_albums().await
	}

	pub async fn get_newly_released_albums(&self) -> Result<Vec<Album>, MusicbirbError> {
		self.api.get_newly_released_albums().await
	}

	pub async fn get_playlists(&self) -> Result<Vec<Playlist>, MusicbirbError> {
		self.api.get_playlists().await
	}

	pub fn next(&self) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::Next)
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))
	}

	pub fn prev(&self) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::Prev)
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))
	}

	pub fn play_index(&self, index: usize) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::PlayIndex(index))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))
	}

	pub fn seek(&self, seconds: f64) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::SeekRelative(seconds))
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))
	}

	pub fn toggle_pause(&self) -> Result<(), MusicbirbError> {
		self.tx
			.send(CoreMessage::TogglePause)
			.map_err(|_| MusicbirbError::Internal("Core loop dead".into()))
	}

	pub fn subscribe(&self) -> watch::Receiver<CoreState> {
		self.state_rx.clone()
	}
}
