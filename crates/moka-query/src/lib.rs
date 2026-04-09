pub mod client;
pub mod state;

// Re-export everything for easy access
pub use client::GlobalQueryClient;
pub use moka_query_macros::moka_query_proxy;
pub use state::QueryState;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!("moka_query");
