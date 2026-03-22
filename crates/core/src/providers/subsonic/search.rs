use super::SubsonicContext;
use crate::error::MusicbirbError;
use crate::models::{Album, Artist, SearchPreset, SearchQuery, SearchResults, Track};
use crate::providers::SearchProvider;
use async_trait::async_trait;
use std::sync::Arc;
use submarine::api::get_album_list::Order;

pub struct SubsonicSearch {
	pub ctx: Arc<SubsonicContext>,
}

#[async_trait]
impl SearchProvider for SubsonicSearch {
	async fn search(&self, query: SearchQuery) -> Result<SearchResults, MusicbirbError> {
		if let Some(preset) = query.preset {
			let limit = query.limit.map(|l| l as usize).or(Some(20));
			let offset = query.offset.map(|o| o as usize);
			match preset {
				SearchPreset::LastPlayedAlbums => {
					let list = self
						.ctx
						.client
						.get_album_list2(Order::Recent, limit, offset, None::<String>)
						.await
						.map_err(|e| MusicbirbError::Api(format!("Failed: {}", e)))?;
					return Ok(SearchResults {
						albums: list.into_iter().map(Album::from).collect(),
						tracks: vec![],
						artists: vec![],
					});
				}
				SearchPreset::RecentlyAddedAlbums => {
					let list = self
						.ctx
						.client
						.get_album_list2(Order::Newest, limit, offset, None::<String>)
						.await
						.map_err(|e| MusicbirbError::Api(format!("Failed: {}", e)))?;
					return Ok(SearchResults {
						albums: list.into_iter().map(Album::from).collect(),
						tracks: vec![],
						artists: vec![],
					});
				}
				SearchPreset::NewlyReleasedAlbums => {
					let list = self
						.ctx
						.client
						.get_album_list2_by_year(Some(9999), Some(0), limit, offset, None::<String>)
						.await
						.map_err(|e| MusicbirbError::Api(format!("Failed: {}", e)))?;
					return Ok(SearchResults {
						albums: list.into_iter().map(Album::from).collect(),
						tracks: vec![],
						artists: vec![],
					});
				}
			}
		}
		let limit = query.limit.or(Some(20));
		let offset = query.offset;
		let kw = query.keyword.unwrap_or_default();
		if kw.is_empty() {
			return Ok(SearchResults {
				albums: vec![],
				tracks: vec![],
				artists: vec![],
			});
		}

		let res = self
			.ctx
			.client
			.search3(&kw, limit, offset, limit, offset, limit, offset, None::<String>)
			.await
			.map_err(|e| MusicbirbError::Api(format!("Search failed: {}", e)))?;

		Ok(SearchResults {
			tracks: res.song.into_iter().map(Track::from).collect(),
			albums: res.album.into_iter().map(Album::from).collect(),
			artists: res.artist.into_iter().map(Artist::from).collect(),
		})
	}
}
