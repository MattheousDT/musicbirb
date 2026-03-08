use crate::models::CoverArtId;
use image::DynamicImage;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub struct ArtCache {
	dir: PathBuf,
}

impl ArtCache {
	pub fn new() -> Self {
		let proj_dirs = directories::ProjectDirs::from("com", "musicbirb", "musicbirb");
		let cache_dir = proj_dirs.unwrap().cache_dir().to_path_buf();
		let artwork_dir = cache_dir.join("artwork");

		let _ = fs::create_dir_all(&artwork_dir);

		Self { dir: artwork_dir }
	}

	pub fn get_path(&self, id: &CoverArtId) -> PathBuf {
		self.dir.join(format!("{}.png", id.0))
	}

	pub fn is_cached(&self, id: &CoverArtId) -> bool {
		self.get_path(id).exists()
	}

	pub fn load_image(&self, id: &CoverArtId) -> Option<Arc<DynamicImage>> {
		let path = self.get_path(id);
		let bytes = fs::read(path).ok()?;
		let img = image::load_from_memory(&bytes).ok()?;
		Some(Arc::new(img))
	}

	pub fn save_and_load(&self, id: &CoverArtId, bytes: &[u8]) -> Option<Arc<DynamicImage>> {
		let path = self.get_path(id);
		fs::write(path, bytes).ok()?;
		let img = image::load_from_memory(bytes).ok()?;
		Some(Arc::new(img))
	}
}
