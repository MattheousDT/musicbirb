pub mod client;
pub mod state;

// Re-export for macro use
pub use async_stream;

// Re-export everything for easy access
pub use client::{QueryClient, Retryable};
pub use moka_query_macros::query_group;
pub use state::QueryState;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!("moka_query");
