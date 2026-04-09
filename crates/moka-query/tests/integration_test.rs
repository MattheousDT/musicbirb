use moka_query::{GlobalQueryClient, moka_query_proxy};
use std::sync::{Arc, Mutex};

// 1. Define the provider trait
#[moka_query_proxy(namespace = "Artist")]
pub trait ArtistProvider: Send + Sync {
	#[query(key = "Details({id})")]
	async fn get_artist_details(&self, id: &String) -> Result<String, String>;

	#[mutation(invalidates =["Artist/*"])]
	async fn star_artist(&self, id: &String) -> Result<(), String>;
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
	assert_eq!(state, ObserveGetArtistDetailsState::Data { data: "123: Unstarred".to_string() });

	// stream2 hits the cache! It goes straight to Data without triggering an API call.
	let state2 = stream2.next().await.unwrap();
	assert_eq!(state2, ObserveGetArtistDetailsState::Data { data: "123: Unstarred".to_string() });

	// We verify the backend was only invoked once! (Deduplication)
	assert_eq!(*call_count.lock().unwrap(), 1);

	// Mutation triggers invalidation!
	provider.star_artist(&"123".to_string()).await.unwrap();

	// Since "Artist/*" was invalidated, BOTH streams will receive a fresh Data packet with the new state.
	let state_updated1 = stream1.next().await.unwrap();
	assert_eq!(state_updated1, ObserveGetArtistDetailsState::Data { data: "123: Starred".to_string() });

	let state_updated2 = stream2.next().await.unwrap();
	assert_eq!(state_updated2, ObserveGetArtistDetailsState::Data { data: "123: Starred".to_string() });

	// The backend should have been fetched exactly 1 more time for the refetch
	assert_eq!(*call_count.lock().unwrap(), 2);
}
