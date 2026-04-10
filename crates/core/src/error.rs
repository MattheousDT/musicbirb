use moka_query::client::MokaRetryable;
use thiserror::Error;

#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
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

impl MokaRetryable for MusicbirbError {
	fn is_transient(&self) -> bool {
		match self {
			// Network errors are usually transient
			MusicbirbError::Network(_) => true,
			// Internal or specific API errors (like 404) might not be
			_ => false,
		}
	}
}
