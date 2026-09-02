// ---------------- [ File: osx-wallpaper-cycler/src/build_wallpaper_cache_assignments_for_desktop_targets.rs ]
crate::ix!();

pub fn build_wallpaper_cache_assignments_for_desktop_targets(
    desktop_paths: &[std::path::PathBuf],
    desktop_remote_ids: &[String],
    assigned_at: std::time::SystemTime,
) -> Vec<WallpaperCacheAssignment> {
    let epoch = assigned_at
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_secs();

    desktop_paths
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            let remote_id = desktop_remote_ids.get(idx).cloned().unwrap_or_default();
            WallpaperCacheAssignmentBuilder::default()
                .target_index(idx + 1)
                .remote_id(remote_id)
                .cache_path(p.clone())
                .assigned_at_epoch_seconds(epoch)
                .build()
                .unwrap()
        })
        .collect()
}
