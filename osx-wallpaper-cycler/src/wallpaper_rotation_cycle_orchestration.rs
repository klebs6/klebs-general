// ---------------- [ File: osx-wallpaper-cycler/src/wallpaper_rotation_cycle_orchestration.rs ]
crate::ix!();

#[cfg(test)]
mod wallpaper_rotation_cycle_orchestration_suite {
    use super::*;

    fn build_candidate(id: String, name: String, path_lower: String) -> DropboxWallpaperCandidate {
        DropboxWallpaperCandidateBuilder::default()
            .id(id)
            .name(name)
            .path_lower(path_lower)
            .build()
            .unwrap()
    }

    #[traced_test]
    fn cycle_downloads_one_per_desktop_when_enough_unique_candidates() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let cache_dir = tempfile::TempDir::new().unwrap();

            let cfg = WallpaperRotatorConfigBuilder::default()
                .dropbox_app_key("k")
                .dropbox_refresh_token("r")
                .dropbox_roots(vec!["/Wallpapers".to_string()])
                .allowed_extensions(vec!["jpg".to_string(), "png".to_string()])
                .interval_seconds(1u64)
                .cache_dir(cache_dir.path().to_path_buf())
                .concurrency(4usize)
                .scope(WallpaperRotatorScope::AllDesktops)
                .build()
                .unwrap();

            let mut candidates: Vec<DropboxWallpaperCandidate> = Vec::new();
            for i in 0..10 {
                candidates.push(build_candidate(
                    format!("id:{i}"),
                    format!("w{i}.jpg"),
                    format!("/wallpapers/w{i}.jpg"),
                ));
            }

            let source = std::sync::Arc::new(MockDropboxWallpaperSource::new(
                candidates,
                std::time::Duration::from_millis(5),
            ));
            let controller = std::sync::Arc::new(MockDesktopWallpaperController::new(5));

            perform_wallpaper_rotation_cycle_once(&cfg, source.clone(), controller.clone())
                .await
                .unwrap();

            let download_ids = source.download_ids().await;
            assert_eq!(download_ids.len(), 5);

            let applied = controller.applied_paths();
            assert_eq!(applied.len(), 5);

            let state = load_wallpaper_cache_state_if_present(cache_dir.path())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(state.assignments().len(), 5);
            assert_eq!(*state.schema_version(), 1u32);
        });
    }

    #[traced_test]
    fn cycle_filters_extensions_and_errors_when_none_eligible() {
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
                .scope(WallpaperRotatorScope::AllDesktops)
                .build()
                .unwrap();

            let candidates = vec![
                build_candidate(
                    "id:1".to_string(),
                    "a.png".to_string(),
                    "/wallpapers/a.png".to_string(),
                ),
                build_candidate(
                    "id:2".to_string(),
                    "b.gif".to_string(),
                    "/wallpapers/b.gif".to_string(),
                ),
            ];

            let source = std::sync::Arc::new(MockDropboxWallpaperSource::new(
                candidates,
                std::time::Duration::from_millis(1),
            ));
            let controller = std::sync::Arc::new(MockDesktopWallpaperController::new(5));

            let err = perform_wallpaper_rotation_cycle_once(&cfg, source, controller)
                .await
                .err()
                .unwrap();

            match err {
                WallpaperRotatorError::NoEligibleWallpapers { .. } => {}
                _ => panic!("unexpected error variant"),
            }
        });
    }

    #[traced_test]
    fn cycle_desktop_count_zero_falls_back_to_one_target() {
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
                .concurrency(4usize)
                .scope(WallpaperRotatorScope::AllDesktops)
                .build()
                .unwrap();

            let candidates = vec![
                build_candidate(
                    "id:1".to_string(),
                    "a.jpg".to_string(),
                    "/wallpapers/a.jpg".to_string(),
                ),
                build_candidate(
                    "id:2".to_string(),
                    "b.jpg".to_string(),
                    "/wallpapers/b.jpg".to_string(),
                ),
            ];

            let source = std::sync::Arc::new(MockDropboxWallpaperSource::new(
                candidates,
                std::time::Duration::from_millis(1),
            ));
            let controller = std::sync::Arc::new(MockDesktopWallpaperController::new(0));

            perform_wallpaper_rotation_cycle_once(&cfg, source.clone(), controller.clone())
                .await
                .unwrap();

            let downloads = source.download_ids().await;
            assert_eq!(downloads.len(), 1);

            let applied = controller.applied_paths();
            assert_eq!(applied.len(), 1);
        });
    }

    #[traced_test]
    fn cycle_reuses_images_when_not_enough_unique_candidates_for_all_desktops() {
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
                .concurrency(4usize)
                .scope(WallpaperRotatorScope::AllDesktops)
                .build()
                .unwrap();

            let candidates = vec![
                build_candidate(
                    "id:1".to_string(),
                    "a.jpg".to_string(),
                    "/wallpapers/a.jpg".to_string(),
                ),
                build_candidate(
                    "id:2".to_string(),
                    "b.jpg".to_string(),
                    "/wallpapers/b.jpg".to_string(),
                ),
            ];

            let source = std::sync::Arc::new(MockDropboxWallpaperSource::new(
                candidates,
                std::time::Duration::from_millis(1),
            ));
            let controller = std::sync::Arc::new(MockDesktopWallpaperController::new(5));

            perform_wallpaper_rotation_cycle_once(&cfg, source.clone(), controller.clone())
                .await
                .unwrap();

            let downloads = source.download_ids().await;
            assert_eq!(downloads.len(), 2);

            let applied = controller.applied_paths();
            assert_eq!(applied.len(), 5);

            let unique_applied: std::collections::HashSet<String> =
                applied.iter().map(|p| p.to_string_lossy().to_string()).collect();
            assert!(unique_applied.len() <= 2);

            let state = load_wallpaper_cache_state_if_present(cache_dir.path())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(state.assignments().len(), 5);
        });
    }

    #[traced_test]
    fn cycle_respects_configured_concurrency_limit() {
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
                .scope(WallpaperRotatorScope::AllDesktops)
                .build()
                .unwrap();

            let mut candidates: Vec<DropboxWallpaperCandidate> = Vec::new();
            for i in 0..8 {
                candidates.push(build_candidate(
                    format!("id:{i}"),
                    format!("w{i}.jpg"),
                    format!("/wallpapers/w{i}.jpg"),
                ));
            }

            let source = std::sync::Arc::new(MockDropboxWallpaperSource::new(
                candidates,
                std::time::Duration::from_millis(25),
            ));
            let controller = std::sync::Arc::new(MockDesktopWallpaperController::new(6));

            perform_wallpaper_rotation_cycle_once(&cfg, source.clone(), controller.clone())
                .await
                .unwrap();

            let max = source.max_concurrency_observed();
            assert!(max <= 2);
        });
    }

    #[traced_test]
    fn cycle_uses_cache_hits_and_avoids_re_download_when_files_exist() {
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
                .concurrency(4usize)
                .scope(WallpaperRotatorScope::AllDesktops)
                .build()
                .unwrap();

            let mut candidates: Vec<DropboxWallpaperCandidate> = Vec::new();
            for i in 0..6 {
                candidates.push(build_candidate(
                    format!("id:{i}"),
                    format!("w{i}.jpg"),
                    format!("/wallpapers/w{i}.jpg"),
                ));
            }

            for c in candidates.iter() {
                let p = compute_wallpaper_cache_path_for_remote(cache_dir.path(), c);
                tokio::fs::write(&p, b"PREEXISTING").await.unwrap();
            }

            let source = std::sync::Arc::new(MockDropboxWallpaperSource::new(
                candidates,
                std::time::Duration::from_millis(1),
            ));
            let controller = std::sync::Arc::new(MockDesktopWallpaperController::new(5));

            perform_wallpaper_rotation_cycle_once(&cfg, source.clone(), controller.clone())
                .await
                .unwrap();

            let downloads = source.download_ids().await;
            assert_eq!(downloads.len(), 0);
        });
    }

    #[traced_test]
    fn cycle_cleans_up_old_cached_files_after_successful_new_assignment() {
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
                .concurrency(4usize)
                .scope(WallpaperRotatorScope::AllDesktops)
                .build()
                .unwrap();

            let cfg2 = WallpaperRotatorConfigBuilder::default()
                .dropbox_app_key("k")
                .dropbox_refresh_token("r")
                .dropbox_roots(vec!["/Wallpapers".to_string()])
                .allowed_extensions(vec!["jpg".to_string()])
                .interval_seconds(1u64)
                .cache_dir(cache_dir.path().to_path_buf())
                .concurrency(4usize)
                .scope(WallpaperRotatorScope::AllDesktops)
                .force_dropbox_directory_cache_rescan(true)
                .build()
                .unwrap();

            let source1 = std::sync::Arc::new(MockDropboxWallpaperSource::new(
                vec![
                    build_candidate(
                        "id:old1".to_string(),
                        "old1.jpg".to_string(),
                        "/wallpapers/old1.jpg".to_string(),
                    ),
                    build_candidate(
                        "id:old2".to_string(),
                        "old2.jpg".to_string(),
                        "/wallpapers/old2.jpg".to_string(),
                    ),
                ],
                std::time::Duration::from_millis(1),
            ));
            let controller = std::sync::Arc::new(MockDesktopWallpaperController::new(2));

            perform_wallpaper_rotation_cycle_once(&cfg1, source1.clone(), controller.clone())
                .await
                .unwrap();

            let old_paths: Vec<std::path::PathBuf> = {
                let state = load_wallpaper_cache_state_if_present(cache_dir.path())
                    .await
                    .unwrap()
                    .unwrap();
                state
                    .assignments()
                    .iter()
                    .map(|a| a.cache_path().clone())
                    .collect()
            };

            assert!(tokio::fs::metadata(&old_paths[0]).await.is_ok());
            assert!(tokio::fs::metadata(&old_paths[1]).await.is_ok());

            let source2 = std::sync::Arc::new(MockDropboxWallpaperSource::new(
                vec![
                    build_candidate(
                        "id:new1".to_string(),
                        "new1.jpg".to_string(),
                        "/wallpapers/new1.jpg".to_string(),
                    ),
                    build_candidate(
                        "id:new2".to_string(),
                        "new2.jpg".to_string(),
                        "/wallpapers/new2.jpg".to_string(),
                    ),
                ],
                std::time::Duration::from_millis(1),
            ));

            perform_wallpaper_rotation_cycle_once(&cfg2, source2.clone(), controller.clone())
                .await
                .unwrap();

            for p in old_paths.iter() {
                assert!(tokio::fs::metadata(p).await.is_err());
            }

            let state = load_wallpaper_cache_state_if_present(cache_dir.path())
                .await
                .unwrap()
                .unwrap();

            let ids: std::collections::HashSet<String> = state
                .assignments()
                .iter()
                .map(|a| a.remote_id().to_string())
                .collect();

            assert!(ids.contains("id:new1"));
            assert!(ids.contains("id:new2"));
        });
    }
}
