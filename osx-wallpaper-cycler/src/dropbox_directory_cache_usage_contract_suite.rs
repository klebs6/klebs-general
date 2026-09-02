// ---------------- [ File: osx-wallpaper-cycler/src/dropbox_directory_cache_usage_contract_suite.rs ]
crate::ix!();

#[cfg(test)]
mod dropbox_directory_cache_usage_contract_suite {
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
    fn directory_cache_is_written_and_avoids_rescanning_on_next_cycle() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let cache_dir = tempfile::TempDir::new().unwrap();

            let cfg = WallpaperRotatorConfigBuilder::default()
                .dropbox_app_key("k")
                .dropbox_refresh_token("r")
                .dropbox_roots(vec!["/Wallpapers".to_string()])
                .allowed_extensions(vec!["jpg".to_string()])
                .interval_seconds(1u64)
                .cache_dir(cache_dir.path().to_path_buf())
                .concurrency(2usize)
                .scope(WallpaperRotatorScope::ActiveDesktopOnly)
                .build()
                .unwrap();

            let candidates = vec![
                build_candidate("id:1"),
                build_candidate("id:2"),
                build_candidate("id:3"),
            ];

            let source = std::sync::Arc::new(MockDropboxWallpaperSource::new(
                candidates,
                std::time::Duration::from_millis(1),
            ));
            let controller = std::sync::Arc::new(MockDesktopWallpaperController::new(1));

            perform_wallpaper_rotation_cycle_once(&cfg, source.clone(), controller.clone())
                .await
                .unwrap();

            assert_eq!(source.list_calls_observed(), 1);

            let cache_path = DropboxDirectoryCacheState::cache_file_path(cache_dir.path());
            assert!(tokio::fs::metadata(&cache_path).await.is_ok());

            perform_wallpaper_rotation_cycle_once(&cfg, source.clone(), controller.clone())
                .await
                .unwrap();

            assert_eq!(
                source.list_calls_observed(),
                1,
                "second cycle should reuse cached directory index and avoid rescanning"
            );
        });
    }

    #[traced_test]
    fn forced_rescan_bypasses_directory_cache_and_calls_source_again() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let cache_dir = tempfile::TempDir::new().unwrap();

            let cfg1 = WallpaperRotatorConfigBuilder::default()
                .dropbox_app_key("k")
                .dropbox_refresh_token("r")
                .dropbox_roots(vec!["/Wallpapers".to_string()])
                .allowed_extensions(vec!["jpg".to_string()])
                .interval_seconds(1u64)
                .cache_dir(cache_dir.path().to_path_buf())
                .concurrency(2usize)
                .scope(WallpaperRotatorScope::ActiveDesktopOnly)
                .build()
                .unwrap();

            let cfg2 = WallpaperRotatorConfigBuilder::default()
                .dropbox_app_key("k")
                .dropbox_refresh_token("r")
                .dropbox_roots(vec!["/Wallpapers".to_string()])
                .allowed_extensions(vec!["jpg".to_string()])
                .interval_seconds(1u64)
                .cache_dir(cache_dir.path().to_path_buf())
                .concurrency(2usize)
                .scope(WallpaperRotatorScope::ActiveDesktopOnly)
                .force_dropbox_directory_cache_rescan(true)
                .build()
                .unwrap();

            let candidates = vec![
                build_candidate("id:1"),
                build_candidate("id:2"),
                build_candidate("id:3"),
            ];

            let source = std::sync::Arc::new(MockDropboxWallpaperSource::new(
                candidates,
                std::time::Duration::from_millis(1),
            ));
            let controller = std::sync::Arc::new(MockDesktopWallpaperController::new(1));

            perform_wallpaper_rotation_cycle_once(&cfg1, source.clone(), controller.clone())
                .await
                .unwrap();
            assert_eq!(source.list_calls_observed(), 1);

            perform_wallpaper_rotation_cycle_once(&cfg2, source.clone(), controller.clone())
                .await
                .unwrap();
            assert_eq!(
                source.list_calls_observed(),
                2,
                "forced rescan should bypass cached directory index"
            );
        });
    }
}
