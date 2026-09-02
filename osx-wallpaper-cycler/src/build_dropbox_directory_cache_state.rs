// ---------------- [ File: osx-wallpaper-cycler/src/build_dropbox_directory_cache_state.rs ]
crate::ix!();

pub fn build_dropbox_directory_cache_state(
    roots: Vec<String>,
    entries: Vec<DropboxWallpaperCandidate>,
    cached_at: std::time::SystemTime,
) -> DropboxDirectoryCacheState {
    let epoch = cached_at
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_secs();

    DropboxDirectoryCacheStateBuilder::default()
        .schema_version(1u32)
        .roots(roots)
        .entries(entries)
        .cached_at_epoch_seconds(epoch)
        .build()
        .unwrap()
}

#[cfg(test)]
mod build_dropbox_directory_cache_state_contract_suite {
    use super::*;

    fn build_candidate(id: &str) -> DropboxWallpaperCandidate {
        DropboxWallpaperCandidateBuilder::default()
            .id(id)
            .name(format!("{id}.jpg"))
            .path_lower(format!("/wallpapers/{id}.jpg"))
            .build()
            .unwrap()
    }

    #[traced_test]
    fn builder_sets_schema_and_epoch_seconds_with_unix_epoch_fallback() {
        let state = build_dropbox_directory_cache_state(
            vec!["/Wallpapers".to_string()],
            vec![build_candidate("id:1")],
            std::time::SystemTime::UNIX_EPOCH,
        );

        assert_eq!(*state.schema_version(), 1u32);
        assert_eq!(*state.cached_at_epoch_seconds(), 0u64);
        assert_eq!(state.roots().len(), 1);
        assert_eq!(state.entries().len(), 1);
    }
}
