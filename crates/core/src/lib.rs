pub mod actor;
pub mod api;
pub mod art_cache;
pub mod core;
pub mod error;
pub mod models;
pub mod player;
pub mod scrobble;
pub mod state;

#[cfg(feature = "os-media-controls")]
pub mod mpris;

pub use api::SubsonicClient;
pub use core::Musicbirb;
pub use error::CoreError;
pub use models::{AlbumId, CoverArtId, PlaylistId, Track, TrackId};
pub use player::{Player, PlayerState, PlayerStatus};
pub use state::{CoreMessage, CoreState};
