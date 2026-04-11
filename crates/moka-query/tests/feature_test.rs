use moka_query::{QueryClient, query_group};
use std::sync::Arc;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!("moka_query_feature_test");

#[query_group(namespace = "FeatureTest")]
pub trait FeatureTrait: Send + Sync {
	#[query(key = "Test")]
	async fn test_method(&self) -> Result<String, String>;
}

struct FeatureProvider;
#[async_trait::async_trait]
impl FeatureTrait for FeatureProvider {
	async fn test_method(&self) -> Result<String, String> {
		Ok("Works".to_string())
	}
}

#[tokio::test]
async fn test_feature_forwarding_compilation() {
	let client = Arc::new(QueryClient::new());
	let provider = CachedFeatureTrait::new(Arc::new(FeatureProvider), client);

	// This method is generated natively by the macro.
	// If the uniffi feature was forwarded correctly, the internal stream struct
	// will have the expected inherent methods regardless of the consumer's flags.
	let stream = provider.observe_test_method();
	let state = stream.current_cached_state();
	assert!(state.is_none());
}
