#[cfg(feature = "ffi")]
uniffi::setup_scaffolding!("musicbirb_ffi");
#[cfg(feature = "ffi")]
use lazy_static::lazy_static;
#[cfg(feature = "ffi")]
lazy_static! {
	pub static ref RUNTIME: tokio::runtime::Runtime =
		tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
}
#[cfg(feature = "ffi")]
pub mod ffi;

pub mod actor;
pub mod art_cache;
pub mod backend;
pub mod core;
pub mod error;
pub mod models;
pub mod providers;
pub mod scrobble;
pub mod state;

pub use crate::core::Musicbirb;
pub use backend::*;
pub use error::MusicbirbError;
pub use models::*;
pub use providers::Provider;
pub use state::{CoreMessage, CoreState};

#[cfg(feature = "os-media-controls")]
pub mod mpris;

#[cfg(feature = "subsonic")]
pub use providers::subsonic::SubsonicProvider;
