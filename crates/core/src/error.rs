use thiserror::Error;

#[derive(Error, Debug)]
pub enum MusicbirbError {
	#[error("Subsonic API error: {0}")]
	Api(String),

	#[error("Network error: {0}")]
	Network(#[from] reqwest::Error),

	#[error("Player error: {0}")]
	Player(String),

	#[error("Internal core error: {0}")]
	Internal(String),
}
