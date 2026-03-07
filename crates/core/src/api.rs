use crate::models::Track;
use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use submarine::{auth::AuthBuilder, Client};

pub struct SubsonicClient {
    client: Client,
    http_client: reqwest::Client,
}

impl SubsonicClient {
    pub fn new(url: &str, username: &str, password: &str) -> Result<Self> {
        let auth = AuthBuilder::new(username, env!("CARGO_PKG_VERSION"))
            .client_name("musicbirb")
            .hashed(password);

        let client = Client::new(url, auth);
        let http_client = reqwest::Client::new();

        Ok(Self {
            client,
            http_client,
        })
    }

    pub async fn get_stream_url(&self, track_id: &str) -> Result<String> {
        let url = self
            .client
            .stream_url(
                track_id,
                None,
                None::<String>,
                None,
                None::<String>,
                None,
                None,
            )
            .map_err(|e| anyhow!("Failed to build stream URL: {}", e))?;
        Ok(url.to_string())
    }

    pub async fn get_track(&self, track_id: &str) -> Result<Track> {
        let data = self
            .client
            .get_song(track_id)
            .await
            .map_err(|e| anyhow!("Failed to fetch track: {}", e))?;

        Ok(Track {
            id: data.id,
            title: data.title,
            artist: data.artist.unwrap_or_else(|| "Unknown".to_string()),
            album: data.album.unwrap_or_else(|| "Unknown".to_string()),
            duration_secs: data.duration.unwrap_or(0) as u32,
            cover_art: data.cover_art,
        })
    }

    pub async fn get_album_tracks(&self, album_id: &str) -> Result<Vec<Track>> {
        let album = self
            .client
            .get_album(album_id)
            .await
            .map_err(|e| anyhow!("Failed: {}", e))?;
        Ok(album
            .song
            .into_iter()
            .map(|s| Track {
                id: s.id,
                title: s.title,
                artist: s.artist.unwrap_or_else(|| "Unknown".to_string()),
                album: s.album.unwrap_or_else(|| "Unknown".to_string()),
                duration_secs: s.duration.unwrap_or(0) as u32,
                cover_art: s.cover_art,
            })
            .collect())
    }

    pub async fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<Track>> {
        let playlist = self
            .client
            .get_playlist(playlist_id)
            .await
            .map_err(|e| anyhow!("Failed: {}", e))?;
        Ok(playlist
            .entry
            .into_iter()
            .map(|s| Track {
                id: s.id,
                title: s.title,
                artist: s.artist.unwrap_or_else(|| "Unknown".to_string()),
                album: s.album.unwrap_or_else(|| "Unknown".to_string()),
                duration_secs: s.duration.unwrap_or(0) as u32,
                cover_art: s.cover_art,
            })
            .collect())
    }

    pub async fn get_cover_art_bytes(&self, cover_id: &str) -> Result<Vec<u8>> {
        let url = self
            .client
            .get_cover_art_url(cover_id, Some(600))
            .map_err(|e| anyhow!("{}", e))?;
        let resp = self
            .http_client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| anyhow!("Download error: {}", e))?;

        if resp.status() != StatusCode::OK {
            return Err(anyhow!("Image download failed: {}", resp.status()));
        }

        let bytes = resp.bytes().await.map_err(|e| anyhow!("{}", e))?;
        Ok(bytes.to_vec())
    }
}
