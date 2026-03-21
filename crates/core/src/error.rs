use thiserror::Error;

#[cfg_attr(feature = "ffi", derive(uniffi::Error))]
#[derive(Error, Debug)]
pub enum MusicbirbError {
	#[error("Subsonic API error: {0}")]
	Api(String),

	#[error("Network error: {0}")]
	Network(String),

	#[error("Player error: {0}")]
	Player(String),

	#[error("Internal core error: {0}")]
	Internal(String),

	#[error("Authentication error: {0}")]
	Auth(String),
}
