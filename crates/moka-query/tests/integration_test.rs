use moka_query::{GlobalQueryClient, moka_query_proxy};
use std::sync::{Arc, Mutex};

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!("moka_query_integration_test");

// 1. Define the provider trait
#[moka_query_proxy(namespace = "Artist")]
pub trait ArtistProvider: Send + Sync {
	#[query(key = "Details({id})")]
	async fn get_artist_details(&self, id: &String) -> Result<String, String>;

	#[mutation(invalidates =["Artist/*"])]
	async fn star_artist(&self, id: &String) -> Result<(), String>;
}

#[moka_query_proxy(namespace = "NoArgs")]
pub trait NoArgsProvider: Send + Sync {
	#[query(key = "FetchAll")]
	async fn fetch_no_args(&self) -> Result<String, String>;
}

struct MockNoArgsProvider;

#[async_trait::async_trait]
impl NoArgsProvider for MockNoArgsProvider {
	async fn fetch_no_args(&self) -> Result<String, String> {
		Ok("All Data".to_string())
	}
}

#[tokio::test]
async fn test_no_args_compilation_and_execution() {
	let global_client = Arc::new(GlobalQueryClient::new());
	let provider = CachedNoArgsProvider::new(Arc::new(MockNoArgsProvider), global_client);

	let stream = provider.observe_fetch_no_args();

	let state = stream.next().await.unwrap();
	assert!(matches!(state, ObserveFetchNoArgsState::Loading));

	let state = stream.next().await.unwrap();
	assert_eq!(
		state,
		ObserveFetchNoArgsState::Data {
			data: "All Data".to_string()
		}
	);
}

#[tokio::test]
async fn test_optimistic_update_success_flow() {
	let call_count = Arc::new(Mutex::new(0));
	let backend_data = Arc::new(Mutex::new("Server V1".to_string()));

	struct FlowProvider {
		count: Arc<Mutex<usize>>,
		data: Arc<Mutex<String>>,
	}
	#[async_trait::async_trait]
	impl NoArgsProvider for FlowProvider {
		async fn fetch_no_args(&self) -> Result<String, String> {
			let mut c = self.count.lock().unwrap();
			*c += 1;
			Ok(self.data.lock().unwrap().clone())
		}
	}

	let global_client = Arc::new(GlobalQueryClient::new());
	let provider = CachedNoArgsProvider::new(
		Arc::new(FlowProvider {
			count: call_count.clone(),
			data: backend_data.clone(),
		}),
		global_client,
	);

	let stream = provider.observe_fetch_no_args();
	let _ = stream.next().await; // Loading
	let _ = stream.next().await; // "Server V1"
	assert_eq!(*call_count.lock().unwrap(), 1);

	// 1. Trigger Optimistic Update
	provider.set_cached_fetch_no_args("Optimistic Guess".to_string()).await;
	assert_eq!(
		stream.next().await.unwrap(),
		ObserveFetchNoArgsState::Data {
			data: "Optimistic Guess".to_string()
		}
	);

	// 2. Simulate Mutation Success (Backend updates to V2)
	{
		*backend_data.lock().unwrap() = "Server V2".to_string();
	}
	provider.moka_invalidate("NoArgs/*".to_string()).await;

	// 3. Stream should re-fetch and overwrite optimistic data with Server V2
	assert_eq!(
		stream.next().await.unwrap(),
		ObserveFetchNoArgsState::Data {
			data: "Server V2".to_string()
		}
	);
	assert_eq!(*call_count.lock().unwrap(), 2);
}

#[tokio::test]
async fn test_optimistic_update_rollback_flow() {
	let backend_data = Arc::new(Mutex::new("Stable Server Data".to_string()));

	struct RollbackProvider {
		data: Arc<Mutex<String>>,
	}
	#[async_trait::async_trait]
	impl NoArgsProvider for RollbackProvider {
		async fn fetch_no_args(&self) -> Result<String, String> {
			Ok(self.data.lock().unwrap().clone())
		}
	}

	let global_client = Arc::new(GlobalQueryClient::new());
	let provider = CachedNoArgsProvider::new(
		Arc::new(RollbackProvider {
			data: backend_data.clone(),
		}),
		global_client,
	);

	let stream = provider.observe_fetch_no_args();
	let _ = stream.next().await; // Loading
	let _ = stream.next().await; // "Stable Server Data"

	// 1. Trigger Optimistic Update
	provider.set_cached_fetch_no_args("Bad Guess".to_string()).await;
	let state = stream.next().await.unwrap();
	assert_eq!(
		state,
		ObserveFetchNoArgsState::Data {
			data: "Bad Guess".to_string()
		}
	);

	// 2. Mutation Fails! We trigger invalidation to "roll back" to server truth
	provider.moka_invalidate("NoArgs/FetchAll".to_string()).await;

	// 3. Stream should yield the original stable data again
	let state = stream.next().await.unwrap();
	assert_eq!(
		state,
		ObserveFetchNoArgsState::Data {
			data: "Stable Server Data".to_string()
		}
	);
}

// 2. Implement a mock backend that counts API calls
struct MockArtistProvider {
	pub call_count: Arc<Mutex<usize>>,
	pub data: Arc<Mutex<String>>,
}

#[async_trait::async_trait]
impl ArtistProvider for MockArtistProvider {
	async fn get_artist_details(&self, id: &String) -> Result<String, String> {
		let mut count = self.call_count.lock().unwrap();
		*count += 1;

		let d = self.data.lock().unwrap().clone();
		Ok(format!("{}: {}", id, d))
	}

	async fn star_artist(&self, _id: &String) -> Result<(), String> {
		let mut d = self.data.lock().unwrap();
		*d = "Starred".to_string();
		Ok(())
	}
}

#[tokio::test]
async fn test_caching_and_invalidation() {
	let call_count = Arc::new(Mutex::new(0));
	let data = Arc::new(Mutex::new("Unstarred".to_string()));

	let raw_provider = MockArtistProvider {
		call_count: call_count.clone(),
		data: data.clone(),
	};

	let global_client = Arc::new(GlobalQueryClient::new());
	let provider = CachedArtistProvider::new(Arc::new(raw_provider), global_client);

	// Observer 1
	let stream1 = provider.observe_get_artist_details("123".to_string());

	// Observer 2 (To test deduplication & cache hits)
	let stream2 = provider.observe_get_artist_details("123".to_string());

	// Both should yield Loading initially, then Data.
	let state = stream1.next().await.unwrap();
	assert!(matches!(state, ObserveGetArtistDetailsState::Loading));

	let state = stream1.next().await.unwrap();
	assert_eq!(
		state,
		ObserveGetArtistDetailsState::Data {
			data: "123: Unstarred".to_string()
		}
	);

	// stream2 hits the cache! It goes straight to Data without triggering an API call.
	let state2 = stream2.next().await.unwrap();
	assert_eq!(
		state2,
		ObserveGetArtistDetailsState::Data {
			data: "123: Unstarred".to_string()
		}
	);

	// We verify the backend was only invoked once! (Deduplication)
	assert_eq!(*call_count.lock().unwrap(), 1);

	// Mutation triggers invalidation!
	provider.star_artist(&"123".to_string()).await.unwrap();

	// Since "Artist/*" was invalidated, BOTH streams will receive a fresh Data packet with the new state.
	let state_updated1 = stream1.next().await.unwrap();
	assert_eq!(
		state_updated1,
		ObserveGetArtistDetailsState::Data {
			data: "123: Starred".to_string()
		}
	);

	let state_updated2 = stream2.next().await.unwrap();
	assert_eq!(
		state_updated2,
		ObserveGetArtistDetailsState::Data {
			data: "123: Starred".to_string()
		}
	);

	// The backend should have been fetched exactly 1 more time for the refetch
	assert_eq!(*call_count.lock().unwrap(), 2);
}

#[tokio::test]
async fn test_mutation_stream_flow() {
	let call_count = Arc::new(Mutex::new(0));
	let data = Arc::new(Mutex::new("Unstarred".to_string()));

	let raw_provider = MockArtistProvider {
		call_count: call_count.clone(),
		data: data.clone(),
	};

	let global_client = Arc::new(GlobalQueryClient::new());
	let provider = CachedArtistProvider::new(Arc::new(raw_provider), global_client);

	let stream = provider.execute_star_artist("123".to_string());

	// 1. Initial execution yields Loading
	let state = stream.next().await.unwrap();
	assert_eq!(state, MutateStarArtistState::Loading);

	// 2. Awaits completion and yields Data (No interior struct data since the return type was Unit)
	let state = stream.next().await.unwrap();
	assert_eq!(state, MutateStarArtistState::Data);

	// 3. Verify backend mutation executed correctly
	assert_eq!(*data.lock().unwrap(), "Starred");
}
