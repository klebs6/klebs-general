// ---------------- [ File: osx-wallpaper-cycler/src/load_wallpaper_cache_state_if_present.rs ]
crate::ix!();

pub async fn load_wallpaper_cache_state_if_present(
    cache_dir: &std::path::Path,
) -> Result<Option<WallpaperCacheState>, WallpaperRotatorError> {
    let state_path = WallpaperCacheState::state_file_path(cache_dir);

    if tokio::fs::metadata(&state_path).await.is_err() {
        return Ok(None);
    }

    let bytes = tokio::fs::read(&state_path).await.map_err(|e| {
        wallpaper_rotator_io_failure(FilesystemOperationKind::ReadFile, state_path.clone(), e)
    })?;

    let state: WallpaperCacheState = serde_json::from_slice(&bytes).map_err(|_| WallpaperRotatorError::CacheStateCorrupt {
        path: state_path.clone(),
    })?;

    Ok(Some(state))
}
