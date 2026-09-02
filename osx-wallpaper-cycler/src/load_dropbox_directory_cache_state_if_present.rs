// ---------------- [ File: osx-wallpaper-cycler/src/load_dropbox_directory_cache_state_if_present.rs ]
crate::ix!();

pub async fn load_dropbox_directory_cache_state_if_present(
    cache_dir: &std::path::Path,
) -> Result<Option<DropboxDirectoryCacheState>, WallpaperRotatorError> {
    let cache_path = DropboxDirectoryCacheState::cache_file_path(cache_dir);

    if tokio::fs::metadata(&cache_path).await.is_err() {
        return Ok(None);
    }

    let bytes = tokio::fs::read(&cache_path).await.map_err(|e| {
        wallpaper_rotator_io_failure(FilesystemOperationKind::ReadFile, cache_path.clone(), e)
    })?;

    match serde_json::from_slice::<DropboxDirectoryCacheState>(&bytes) {
        Ok(v) => Ok(Some(v)),
        Err(e) => {
            tracing::warn!(
                path = %cache_path.display(),
                bytes = bytes.len(),
                error = %e,
                "dropbox directory cache was present but failed to decode; ignoring"
            );
            Ok(None)
        }
    }
}
