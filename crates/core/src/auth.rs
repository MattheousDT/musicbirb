use crate::{MusicbirbError, Provider, run_async};
use std::sync::Arc;

#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AuthCredential {
	Password(String),
	Token(String),
}

#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
#[derive(Clone, Debug, PartialEq)]
pub enum AuthStep {
	UserPass,
	BrowserAuth {
		auth_url: String,
		display_code: String,
		polling_id: String,
	},
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct AuthResult {
	pub provider: Arc<dyn Provider>,
	pub credential: AuthCredential,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Object))]
pub struct Authenticator;

#[cfg_attr(feature = "ffi", uniffi::export)]
impl Authenticator {
	#[cfg_attr(feature = "ffi", uniffi::constructor)]
	pub fn new() -> Arc<Self> {
		Arc::new(Self)
	}

	pub fn get_supported_providers(&self) -> Vec<String> {
		vec!["subsonic".into(), "jellyfin".into(), "plex".into()]
	}

	pub fn credential_to_json(&self, cred: AuthCredential) -> String {
		serde_json::to_string(&cred).unwrap_or_default()
	}

	pub fn credential_from_json(&self, json: String) -> Option<AuthCredential> {
		serde_json::from_str(&json).ok()
	}

	pub async fn init_auth(&self, provider: String, _server_url: String) -> Result<AuthStep, MusicbirbError> {
		run_async!(async move {
			match provider.as_str() {
				"subsonic" | "jellyfin" => Ok(AuthStep::UserPass),
				"plex" => {
					// TODO: Implement Plex PIN generation via https://plex.tv/api/v2/pins
					Err(MusicbirbError::Internal("Plex OAuth flow not yet implemented".into()))
				}
				_ => Err(MusicbirbError::Internal("Unknown provider".into())),
			}
		})
	}

	pub async fn login_with_password(
		&self,
		provider: String,
		server_url: String,
		username: String,
		password: String,
	) -> Result<AuthResult, MusicbirbError> {
		run_async!(async move {
			match provider.as_str() {
				#[cfg(feature = "subsonic")]
				"subsonic" => {
					let p: Arc<dyn Provider> = Arc::new(crate::providers::subsonic::SubsonicProvider::new(
						&server_url,
						&username,
						&password,
					)?);
					p.ping().await?;
					Ok(AuthResult {
						provider: p,
						credential: AuthCredential::Password(password),
					})
				}
				#[cfg(feature = "jellyfin")]
				"jellyfin" => {
					let mut ctx = crate::providers::jellyfin::JellyfinContext::new(&server_url);
					let auth = ctx.login(&username, &password).await?;
					let token = auth.access_token.clone();

					let p: Arc<dyn Provider> = Arc::new(crate::providers::jellyfin::JellyfinProvider::new(ctx));
					Ok(AuthResult {
						provider: p,
						credential: AuthCredential::Token(token),
					})
				}
				_ => Err(MusicbirbError::Internal(format!(
					"Provider '{}' does not support password login",
					provider
				))),
			}
		})
	}

	pub async fn poll_browser_auth(
		&self,
		provider: String,
		_server_url: String,
		_polling_id: String,
	) -> Result<AuthResult, MusicbirbError> {
		run_async!(async move {
			match provider.as_str() {
				"plex" => {
					// TODO: Implement polling https://plex.tv/api/v2/pins/{polling_id}
					Err(MusicbirbError::Internal("Plex polling not yet implemented".into()))
				}
				_ => Err(MusicbirbError::Internal(
					"Provider does not support browser auth".into(),
				)),
			}
		})
	}

	pub async fn connect_with_credential(
		&self,
		provider: String,
		server_url: String,
		_username: String,
		credential: AuthCredential,
	) -> Result<Arc<dyn Provider>, MusicbirbError> {
		run_async!(async move {
			match provider.as_str() {
				#[cfg(feature = "subsonic")]
				"subsonic" => {
					if let AuthCredential::Password(pass) = credential {
						let p: Arc<dyn Provider> = Arc::new(crate::providers::subsonic::SubsonicProvider::new(
							&server_url,
							&_username,
							&pass,
						)?);
						p.ping().await?;
						Ok(p)
					} else {
						Err(MusicbirbError::Auth("Subsonic requires a password credential".into()))
					}
				}
				#[cfg(feature = "jellyfin")]
				"jellyfin" => {
					if let AuthCredential::Token(token) = credential {
						let mut ctx = crate::providers::jellyfin::JellyfinContext::new(&server_url);
						ctx.set_token(token);

						ctx.fetch_me().await?;

						let p: Arc<dyn Provider> = Arc::new(crate::providers::jellyfin::JellyfinProvider::new(ctx));
						Ok(p)
					} else {
						Err(MusicbirbError::Auth("Jellyfin requires a token credential".into()))
					}
				}
				_ => Err(MusicbirbError::Internal("Unknown provider".into())),
			}
		})
	}
}
