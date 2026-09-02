// ---------------- [ File: osx-wallpaper-cycler/src/persist_dropbox_directory_cache_state.rs ]
crate::ix!();

pub async fn persist_dropbox_directory_cache_state(
    cache_dir: &std::path::Path,
    state: &DropboxDirectoryCacheState,
) -> Result<(), WallpaperRotatorError> {
    let cache_path = DropboxDirectoryCacheState::cache_file_path(cache_dir);

    let bytes = serde_json::to_vec_pretty(state)
        .map_err(|e| WallpaperRotatorError::JsonEncodeFailure { source: e })?;

    tokio::fs::write(&cache_path, &bytes).await.map_err(|e| {
        wallpaper_rotator_io_failure(FilesystemOperationKind::WriteFile, cache_path.clone(), e)
    })?;

    Ok(())
}
