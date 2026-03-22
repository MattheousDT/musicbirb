use super::{JellyfinContext, dto::*};
use crate::error::MusicbirbError;
use crate::models::{Album, Artist, ArtistId, CoverArtId, SearchPreset, SearchQuery, SearchResults, Track};
use crate::providers::SearchProvider;
use std::sync::Arc;

pub struct JellyfinSearch {
	pub ctx: Arc<JellyfinContext>,
}

#[macros::async_ffi]
impl SearchProvider for JellyfinSearch {
	async fn search(&self, query: SearchQuery) -> Result<SearchResults, MusicbirbError> {
		let user_id = self.ctx.get_user_id()?;
		let limit = query.limit.unwrap_or(20);

		if let Some(preset) = query.preset {
			match preset {
				SearchPreset::LastPlayedAlbums => {
					let res: QueryResult<BaseItemDto> = self.ctx.fetch(
						&format!("/Users/{}/Items?IncludeItemTypes=MusicAlbum&SortBy=DatePlayed&SortOrder=Descending&Limit={}&Recursive=true&EnableImages=true", user_id, limit)
					).await?;
					return Ok(SearchResults {
						albums: res.items.into_iter().map(Album::from).collect(),
						tracks: vec![],
						artists: vec![],
					});
				}
				SearchPreset::RecentlyAddedAlbums => {
					let items: Vec<BaseItemDto> = self
						.ctx
						.fetch(&format!(
							"/Users/{}/Items/Latest?IncludeItemTypes=MusicAlbum&Limit={}&EnableImages=true",
							user_id, limit
						))
						.await?;
					return Ok(SearchResults {
						albums: items.into_iter().map(Album::from).collect(),
						tracks: vec![],
						artists: vec![],
					});
				}
				SearchPreset::NewlyReleasedAlbums => {
					let res: QueryResult<BaseItemDto> = self
						.ctx
						.fetch(&format!(
							"/Users/{}/Items?IncludeItemTypes=MusicAlbum&SortBy=ProductionYear&SortOrder=Descending&Limit={}&Recursive=true",
							user_id, limit
						))
						.await?;
					return Ok(SearchResults {
						albums: res.items.into_iter().map(Album::from).collect(),
						tracks: vec![],
						artists: vec![],
					});
				}
			}
		}

		let kw = query.keyword.unwrap_or_default();
		if kw.is_empty() {
			return Ok(SearchResults {
				albums: vec![],
				tracks: vec![],
				artists: vec![],
			});
		}

		let res: QueryResult<BaseItemDto> = self
			.ctx
			.fetch(&format!(
				"/Users/{}/Items?SearchTerm={}&IncludeItemTypes=Audio,MusicAlbum,MusicArtist&Limit={}&Recursive=true",
				user_id, kw, limit
			))
			.await?;

		let mut tracks = vec![];
		let mut albums = vec![];
		let mut artists = vec![];

		for item in res.items {
			if let Some(t) = &item.item_type {
				match t.as_str() {
					"Audio" => tracks.push(Track::from(item)),
					"MusicAlbum" => albums.push(Album::from(item)),
					"MusicArtist" => {
						artists.push(Artist {
							id: ArtistId(item.id.clone()),
							name: item.name.unwrap_or_else(|| "Unknown".to_string()),
							cover_art: Some(CoverArtId(item.id)),
							artist_image_url: None,
						});
					}
					_ => {}
				}
			}
		}

		Ok(SearchResults {
			tracks,
			albums,
			artists,
		})
	}
}
