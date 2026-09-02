// ---------------- [ File: osx-wallpaper-cycler/src/wallpaper_cache_state.rs ]
crate::ix!();

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, getset::Getters, derive_builder::Builder)]
#[getset(get = "pub")]
#[builder(setter(into))]
pub struct WallpaperCacheState {
    schema_version: u32,
    assignments: Vec<WallpaperCacheAssignment>,
}

impl WallpaperCacheState {
    pub fn state_file_path(cache_dir: &std::path::Path) -> std::path::PathBuf {
        cache_dir.join("state.json")
    }
}
