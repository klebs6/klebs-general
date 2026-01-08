// ---------------- [ File: osx-wallpaper-cycler/src/wallpaper_cache_assignment.rs ]
crate::ix!();

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, getset::Getters, derive_builder::Builder)]
#[getset(get = "pub")]
#[builder(setter(into))]
pub struct WallpaperCacheAssignment {
    target_index: usize,
    remote_id: String,
    cache_path: std::path::PathBuf,
    assigned_at_epoch_seconds: u64,
}
