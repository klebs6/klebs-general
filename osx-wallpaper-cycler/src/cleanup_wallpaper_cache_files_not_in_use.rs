// ---------------- [ File: osx-wallpaper-cycler/src/cleanup_wallpaper_cache_files_not_in_use.rs ]
crate::ix!();

pub async fn cleanup_wallpaper_cache_files_not_in_use(
    cache_dir: &std::path::Path,
    prior_state: Option<&WallpaperCacheState>,
    new_assignments: &[WallpaperCacheAssignment],
) -> Result<(), WallpaperRotatorError> {
    if new_assignments.len() <= 1 {
        tracing::warn!(
            cache_dir = %cache_dir.display(),
            new_assignments = new_assignments.len(),
            "skipping cache cleanup because only a single target assignment is present; this avoids deleting wallpapers still in use by other Spaces"
        );
        return Ok(());
    }

    let mut keep: std::collections::HashSet<std::path::PathBuf> = std::collections::HashSet::new();
    keep.insert(WallpaperCacheState::state_file_path(cache_dir));

    for a in new_assignments.iter() {
        keep.insert(a.cache_path().clone());
    }

    let Some(prior) = prior_state else {
        return Ok(());
    };

    let mut attempted: std::collections::HashSet<std::path::PathBuf> = std::collections::HashSet::new();

    for old in prior.assignments().iter() {
        let path = old.cache_path().clone();

        if keep.contains(&path) {
            continue;
        }
        if !attempted.insert(path.clone()) {
            continue;
        }

        match tokio::fs::remove_file(&path).await {
            Ok(_) => {
                tracing::info!(path = %path.display(), "removed old cached wallpaper");
            }
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "failed to remove old cached wallpaper");
            }
        }
    }

    Ok(())
}

