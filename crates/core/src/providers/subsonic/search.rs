use super::SubsonicContext;
use crate::error::MusicbirbError;
use crate::models::{Album, Artist, SearchPreset, SearchQuery, SearchResults, Track};
use crate::providers::SearchProvider;
use std::sync::Arc;

pub struct SubsonicSearch {
	pub ctx: Arc<SubsonicContext>,
}

#[macros::async_ffi]
impl SearchProvider for SubsonicSearch {
	async fn search(&self, query: SearchQuery) -> Result<SearchResults, MusicbirbError> {
		let limit_str = query.limit.map(|l| l.to_string()).unwrap_or_else(|| "20".to_string());
		let offset_str = query.offset.map(|o| o.to_string()).unwrap_or_else(|| "0".to_string());

		if let Some(preset) = query.preset {
			let preset_type = match preset {
				SearchPreset::LastPlayedAlbums => "recent",
				SearchPreset::RecentlyAddedAlbums => "newest",
				SearchPreset::NewlyReleasedAlbums => "byYear",
			};

			let mut params = vec![("type", preset_type), ("size", &limit_str), ("offset", &offset_str)];

			if preset == SearchPreset::NewlyReleasedAlbums {
				params.push(("fromYear", "2999"));
				params.push(("toYear", "0"));
			}

			let res = self.ctx.get_rest_response("getAlbumList2", &params).await?;
			let albums = res.album_list2.map(|list| list.album).unwrap_or_default();

			return Ok(SearchResults {
				albums: albums.into_iter().map(Album::from).collect(),
				tracks: vec![],
				artists: vec![],
			});
		}

		let kw = query.keyword.unwrap_or_default();
		if kw.is_empty() {
			return Ok(SearchResults {
				albums: vec![],
				tracks: vec![],
				artists: vec![],
			});
		}

		let div_limit = ((query.limit.unwrap_or(20) as f32) / 3.0).ceil() as i32;
		let div_limit_str = div_limit.to_string();

		let res = self
			.ctx
			.get_rest_response(
				"search3",
				&[
					("query", &kw),
					("artistCount", &div_limit_str),
					("artistOffset", &offset_str),
					("albumCount", &div_limit_str),
					("albumOffset", &offset_str),
					("songCount", &div_limit_str),
					("songOffset", &offset_str),
				],
			)
			.await?;

		// Since both SearchResult2 and SearchResult3 now use the same SearchResult struct type,
		// they can be combined using .or() correctly.
		let search_res = res.search_result3.or(res.search_result2);

		if let Some(s) = search_res {
			Ok(SearchResults {
				tracks: s.song.into_iter().map(Track::from).collect(),
				albums: s.album.into_iter().map(Album::from).collect(),
				artists: s.artist.into_iter().map(Artist::from).collect(),
			})
		} else {
			Ok(SearchResults {
				albums: vec![],
				tracks: vec![],
				artists: vec![],
			})
		}
	}
}
