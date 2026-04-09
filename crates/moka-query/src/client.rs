use crate::state::QueryState;
use moka::future::Cache;
use std::any::Any;
use std::collections::HashSet;
use std::future::Future;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

/// Global Client that orchestrates caching, deduplication, and reactivity.
pub struct GlobalQueryClient {
	cache: Cache<String, Arc<dyn Any + Send + Sync>>,
	tx: broadcast::Sender<String>,
	active_keys: Arc<RwLock<HashSet<String>>>,
}

impl GlobalQueryClient {
	pub fn new() -> Self {
		let (tx, _) = broadcast::channel(1024);
		Self {
			cache: Cache::builder().build(),
			tx,
			active_keys: Arc::new(RwLock::new(HashSet::new())),
		}
	}

	fn match_pattern(pattern: &str, key: &str) -> bool {
		if let Some(prefix) = pattern.strip_suffix("/*") {
			key.starts_with(prefix)
		} else {
			pattern == key
		}
	}

	pub async fn invalidate_pattern(&self, pattern: &str) {
		let pattern_str = pattern.to_string();
		let keys_to_remove: Vec<String> = {
			let keys = self.active_keys.read().unwrap();
			keys.iter()
				.filter(|k| Self::match_pattern(&pattern_str, k))
				.cloned()
				.collect()
		};

		for k in keys_to_remove {
			self.cache.invalidate(&k).await;
		}
		let _ = self.tx.send(pattern_str);
	}

	/// Subscribes to a stream of state updates for a specific cache key.
    /// Changed to be synchronous since it just builds and returns the stream immediately!
	pub fn observe<T, F, Fut, E>(
		self: Arc<Self>,
		key: String,
		fetcher: F,
	) -> impl futures::Stream<Item = QueryState<T>>
	where
		T: Any + Send + Sync + Clone + 'static,
		F: Fn() -> Fut + Send + Sync + 'static,
		Fut: Future<Output = Result<T, E>> + Send + 'static,
		E: std::fmt::Display,
	{
		self.active_keys.write().unwrap().insert(key.clone());

		async_stream::stream! {
			let mut rx = self.tx.subscribe();
			let mut has_emitted_data = false;

			loop {
				let has_cache = self.cache.get(&key).await.is_some();
				if !has_cache && !has_emitted_data {
					yield QueryState::Loading;
				}

				let fetch_key = key.clone();
				let fetch_result = self.cache.try_get_with(fetch_key, async {
					match fetcher().await {
						Ok(res) => Ok(Arc::new(res) as Arc<dyn Any + Send + Sync>),
						Err(e) => Err(e.to_string()),
					}
				}).await;

				match fetch_result {
					Ok(data) => {
						if let Ok(typed_data) = data.downcast::<T>() {
							has_emitted_data = true;
							yield QueryState::Data((*typed_data).clone());
						}
					}
					Err(err) => yield QueryState::Error(err.to_string())
				}

				loop {
					match rx.recv().await {
						Ok(pattern) => {
							if Self::match_pattern(&pattern, &key) { break; }
						}
						Err(_) => break,
					}
				}
			}
		}
	}
}

impl Default for GlobalQueryClient {
	fn default() -> Self { Self::new() }
}
