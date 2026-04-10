use moka_query::{GlobalQueryClient, moka_query_proxy};
use std::sync::Arc;
use tokio::time::{Duration, sleep};

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!("moka_query_basic_example");

#[moka_query_proxy(namespace = "Users")]
pub trait UserProvider: Send + Sync {
	#[query(key = "Profile({id})")]
	async fn fetch_user(&self, id: &String) -> Result<String, String>;

	#[mutation(invalidates = ["Users/*"])]
	async fn update_user_name(&self, id: &String, new_name: String) -> Result<(), String>;
}

pub struct ApiProvider;

#[async_trait::async_trait]
impl UserProvider for ApiProvider {
	async fn fetch_user(&self, id: &String) -> Result<String, String> {
		println!("  [API] Fetching backend user {}...", id);
		sleep(Duration::from_millis(500)).await;
		Ok(format!("UserData_{}", id))
	}

	async fn update_user_name(&self, id: &String, _new_name: String) -> Result<(), String> {
		println!("  [API] Updating user {}...", id);
		sleep(Duration::from_millis(200)).await;
		Ok(())
	}
}

#[tokio::main]
async fn main() {
	let client = Arc::new(GlobalQueryClient::new());
	let provider = CachedUserProvider::new(Arc::new(ApiProvider), client.clone());

	println!("Observer starting...");
	let stream = provider.observe_fetch_user("John".to_string());

	// Initial fetch
	tokio::spawn(async move {
		while let Some(state) = stream.next().await {
			match state {
				ObserveFetchUserState::Loading => println!("UI State: Loading..."),
				ObserveFetchUserState::Data { data } => println!("UI State: Render Data -> {}", data),
				ObserveFetchUserState::Error { message } => println!("UI State: Render Error -> {}", message),
			}
		}
	});

	sleep(Duration::from_secs(2)).await;

	println!("\nTriggering Mutation...");
	// Update user (which triggers invalidation of "Users/*")
	provider
		.update_user_name(&"John".to_string(), "Johnny".to_string())
		.await
		.unwrap();

	// Wait to see the UI stream automatically refresh
	sleep(Duration::from_secs(2)).await;
}
