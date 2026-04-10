use moka_query::{GlobalQueryClient, moka_query_proxy};
use std::sync::{Arc, Mutex};

#[moka_query_proxy(namespace = "Retry")]
pub trait RetryTrait: Send + Sync {
	#[query(key = "Test", retries = 3)]
	async fn fetch_retry(&self) -> Result<String, String>;
}

struct RetryProvider {
	attempts: Arc<Mutex<u32>>,
}
#[async_trait::async_trait]
impl RetryTrait for RetryProvider {
	async fn fetch_retry(&self) -> Result<String, String> {
		let mut a = self.attempts.lock().unwrap();
		*a += 1;
		if *a < 3 {
			return Err("Transient Error".to_string());
		}
		Ok("Success after retries".to_string())
	}
}

struct FailProvider {
	attempts: Arc<Mutex<u32>>,
}
#[async_trait::async_trait]
impl RetryTrait for FailProvider {
	async fn fetch_retry(&self) -> Result<String, String> {
		let mut a = self.attempts.lock().unwrap();
		*a += 1;
		Err("Permanent Failure".to_string())
	}
}

#[tokio::test]
async fn test_retry_success_flow() {
	let attempts = Arc::new(Mutex::new(0));
	let global_client = Arc::new(GlobalQueryClient::new());
	let provider = CachedRetryTrait::new(
		Arc::new(RetryProvider {
			attempts: attempts.clone(),
		}),
		global_client,
	);

	let stream = provider.observe_fetch_retry();
	let _ = stream.next().await; // Loading

	// Should eventually yield success after 2 failures
	let state = stream.next().await.unwrap();
	assert_eq!(
		state,
		ObserveFetchRetryState::Data {
			data: "Success after retries".to_string()
		}
	);
	assert_eq!(*attempts.lock().unwrap(), 3);
}

#[tokio::test]
async fn test_retry_exhaustion_flow() {
	let attempts = Arc::new(Mutex::new(0));
	let global_client = Arc::new(GlobalQueryClient::new());
	let provider = CachedRetryTrait::new(
		Arc::new(FailProvider {
			attempts: attempts.clone(),
		}),
		global_client,
	);

	let stream = provider.observe_fetch_retry();
	let _ = stream.next().await; // Loading

	let state = stream.next().await.unwrap();
	assert!(matches!(state, ObserveFetchRetryState::Error { .. }));

	// retries = 3 means 1 initial call + 3 retries = 4 total attempts
	assert_eq!(*attempts.lock().unwrap(), 4);
}

#[derive(Debug, Clone)]
pub enum MyError {
	Fatal,
	Transient,
}

impl std::fmt::Display for MyError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}
impl moka_query::client::MokaRetryable for MyError {
	fn is_transient(&self) -> bool {
		matches!(self, MyError::Transient)
	}
}

#[moka_query_proxy(namespace = "Fatal")]
pub trait FatalTrait: Send + Sync {
	#[query(key = "Test", retries = 5)]
	async fn fetch(&self) -> Result<String, MyError>;
}

#[tokio::test]
async fn test_fatal_error_stops_retrying() {
	let attempts = Arc::new(Mutex::new(0));

	struct FatalProviderImpl {
		attempts: Arc<Mutex<u32>>,
	}
	#[async_trait::async_trait]
	impl FatalTrait for FatalProviderImpl {
		async fn fetch(&self) -> Result<String, MyError> {
			let mut a = self.attempts.lock().unwrap();
			*a += 1;
			Err(MyError::Fatal)
		}
	}

	let global_client = Arc::new(GlobalQueryClient::new());
	let provider = CachedFatalTrait::new(
		Arc::new(FatalProviderImpl {
			attempts: attempts.clone(),
		}),
		global_client,
	);

	let stream = provider.observe_fetch();
	let _ = stream.next().await; // Loading
	let _ = stream.next().await; // Error

	// Even though retries = 5, it should stop after 1 call because error was Fatal
	assert_eq!(*attempts.lock().unwrap(), 1);
}

#[tokio::test]
async fn test_custom_error_transient_retry_flow() {
	let attempts = Arc::new(Mutex::new(0));

	struct TransientProvider {
		attempts: Arc<Mutex<u32>>,
	}
	#[async_trait::async_trait]
	impl FatalTrait for TransientProvider {
		async fn fetch(&self) -> Result<String, MyError> {
			let mut a = self.attempts.lock().unwrap();
			*a += 1;
			if *a < 2 {
				return Err(MyError::Transient);
			}
			Ok("Recovered".to_string())
		}
	}

	let global_client = Arc::new(GlobalQueryClient::new());
	let provider = CachedFatalTrait::new(
		Arc::new(TransientProvider {
			attempts: attempts.clone(),
		}),
		global_client,
	);

	let stream = provider.observe_fetch();
	let _ = stream.next().await; // Loading

	// Should retry once and then succeed
	let state = stream.next().await.unwrap();
	assert_eq!(
		state,
		ObserveFetchState::Data {
			data: "Recovered".to_string()
		}
	);
	assert_eq!(*attempts.lock().unwrap(), 2);
}
