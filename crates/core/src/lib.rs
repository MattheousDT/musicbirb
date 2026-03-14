#[cfg(feature = "ffi")]
uniffi::setup_scaffolding!("musicbirb");

pub mod actor;
pub mod api;
pub mod art_cache;
pub mod backend;
pub mod core;
pub mod error;
pub mod models;
pub mod scrobble;
pub mod state;

pub use crate::core::Musicbirb;
pub use api::subsonic::SubsonicClient;
pub use backend::*;
pub use error::MusicbirbError;
pub use models::{AlbumId, CoverArtId, PlaylistId, Track, TrackId};
pub use state::{CoreMessage, CoreState};

#[cfg(feature = "os-media-controls")]
pub mod mpris;
