// ---------------- [ File: osx-wallpaper-cycler/src/ensure_wallpaper_cache_dir_ready.rs ]
crate::ix!();

pub async fn ensure_wallpaper_cache_dir_ready(
    cache_dir: &std::path::Path,
) -> Result<(), WallpaperRotatorError> {
    tokio::fs::create_dir_all(cache_dir).await.map_err(|e| {
        wallpaper_rotator_io_failure(FilesystemOperationKind::CreateDirAll, cache_dir.to_path_buf(), e)
    })?;
    Ok(())
}
