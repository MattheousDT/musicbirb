use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuthRequest<'a> {
	pub username: &'a str,
	pub pw: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuthResponse {
	pub access_token: String,
	pub user: UserDto,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserDto {
	pub id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct QueryResult<T> {
	pub items: Vec<T>,
	pub total_record_count: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BaseItemDto {
	pub id: String,
	pub name: Option<String>,
	#[serde(rename = "Type")]
	pub item_type: Option<String>,
	pub overview: Option<String>,
	pub child_count: Option<u32>,
	pub run_time_ticks: Option<u64>,
	pub production_year: Option<u32>,
	pub album: Option<String>,
	pub album_id: Option<String>,
	pub artists: Option<Vec<String>>,
	pub artist_items: Option<Vec<NameIdPair>>,
	pub index_number: Option<u32>,
	pub parent_index_number: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NameIdPair {
	pub name: String,
	pub id: String,
}
