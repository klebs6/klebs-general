// ---------------- [ File: osx-wallpaper-cycler/src/persist_wallpaper_cache_state.rs ]
crate::ix!();

pub async fn persist_wallpaper_cache_state(
    cache_dir: &std::path::Path,
    state: &WallpaperCacheState,
) -> Result<(), WallpaperRotatorError> {
    let state_path = WallpaperCacheState::state_file_path(cache_dir);
    let bytes = serde_json::to_vec_pretty(state)
        .map_err(|e| WallpaperRotatorError::JsonEncodeFailure { source: e })?;

    tokio::fs::write(&state_path, &bytes).await.map_err(|e| {
        wallpaper_rotator_io_failure(FilesystemOperationKind::WriteFile, state_path.clone(), e)
    })?;

    Ok(())
}
