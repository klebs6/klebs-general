// ---------------- [ File: osx-wallpaper-cycler/src/wallpaper_rotator_cache_contract_suite.rs ]
crate::ix!();

#[cfg(test)]
mod wallpaper_rotator_cache_contract_suite {
    use super::*;

    fn build_candidate(id: &str, name: &str, path_lower: &str) -> DropboxWallpaperCandidate {
        DropboxWallpaperCandidateBuilder::default()
            .id(id)
            .name(name)
            .path_lower(path_lower)
            .build()
            .unwrap()
    }

    fn build_assignment(
        target_index: usize,
        remote_id: &str,
        cache_path: std::path::PathBuf,
        assigned_at_epoch_seconds: u64,
    ) -> WallpaperCacheAssignment {
        WallpaperCacheAssignmentBuilder::default()
            .target_index(target_index)
            .remote_id(remote_id)
            .cache_path(cache_path)
            .assigned_at_epoch_seconds(assigned_at_epoch_seconds)
            .build()
            .unwrap()
    }

    #[traced_test]
    fn cache_state_file_path_is_under_cache_dir() {
        let dir = std::path::PathBuf::from("/tmp/cache");
        let p = WallpaperCacheState::state_file_path(&dir);
        assert_eq!(p, dir.join("state.json"));
    }

    #[traced_test]
    fn compute_cache_path_is_stable_and_uses_extension_or_fallback() {
        let cache_dir = std::path::PathBuf::from("/tmp/cache");
        let c1 = build_candidate("id:abc", "pic.heic", "/x/pic.heic");
        let c2 = build_candidate("id:abc", "pic", "/x/pic");

        let p1a = compute_wallpaper_cache_path_for_remote(&cache_dir, &c1);
        let p1b = compute_wallpaper_cache_path_for_remote(&cache_dir, &c1);
        assert_eq!(p1a, p1b);
        assert!(p1a.to_string_lossy().ends_with(".heic"));

        let p2 = compute_wallpaper_cache_path_for_remote(&cache_dir, &c2);
        assert!(p2.to_string_lossy().ends_with(".img"));
    }

    #[traced_test]
    fn cache_state_persist_and_load_round_trips() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let dir = tempfile::TempDir::new().unwrap();
            let cache_dir = dir.path();

            ensure_wallpaper_cache_dir_ready(cache_dir).await.unwrap();

            let assignment = build_assignment(1, "id:1", cache_dir.join("a.jpg"), 123);
            let state = build_wallpaper_cache_state(vec![assignment]);

            persist_wallpaper_cache_state(cache_dir, &state).await.unwrap();
            let loaded = load_wallpaper_cache_state_if_present(cache_dir)
                .await
                .unwrap()
                .unwrap();

            assert_eq!(loaded.schema_version(), &1u32);
            assert_eq!(loaded.assignments().len(), 1);
            assert_eq!(loaded.assignments()[0].remote_id(), "id:1");
        });
    }

    #[traced_test]
    fn cache_cleanup_removes_old_files_and_keeps_current_and_state() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let dir = tempfile::TempDir::new().unwrap();
            let cache_dir = dir.path().to_path_buf();

            ensure_wallpaper_cache_dir_ready(&cache_dir).await.unwrap();

            let old_keep = cache_dir.join("keep.jpg");
            let old_remove = cache_dir.join("remove.jpg");
            let other_assigned = cache_dir.join("other.jpg");

            tokio::fs::write(&old_keep, b"k").await.unwrap();
            tokio::fs::write(&old_remove, b"r").await.unwrap();
            tokio::fs::write(&other_assigned, b"o").await.unwrap();

            let prior = WallpaperCacheStateBuilder::default()
                .schema_version(1u32)
                .assignments(vec![
                    build_assignment(1, "id:keep", old_keep.clone(), 1),
                    build_assignment(2, "id:remove", old_remove.clone(), 1),
                ])
                .build()
                .unwrap();

            let new_assignments = vec![
                build_assignment(1, "id:keep", old_keep.clone(), 2),
                build_assignment(2, "id:other", other_assigned.clone(), 2),
            ];

            cleanup_wallpaper_cache_files_not_in_use(&cache_dir, Some(&prior), &new_assignments)
                .await
                .unwrap();

            assert!(tokio::fs::metadata(&old_keep).await.is_ok());
            assert!(tokio::fs::metadata(&old_remove).await.is_err());
            assert!(tokio::fs::metadata(&other_assigned).await.is_ok());
        });
    }

    #[traced_test]
    fn assignment_builder_creates_target_indices_and_epoch_seconds() {
        let paths = vec![
            std::path::PathBuf::from("/tmp/a.jpg"),
            std::path::PathBuf::from("/tmp/b.jpg"),
        ];
        let ids = vec!["id:a".to_string(), "id:b".to_string()];
        let assignments = build_wallpaper_cache_assignments_for_desktop_targets(
            &paths,
            &ids,
            std::time::SystemTime::UNIX_EPOCH,
        );

        assert_eq!(assignments.len(), 2);
        assert_eq!(assignments[0].target_index(), &1usize);
        assert_eq!(assignments[1].target_index(), &2usize);
        assert_eq!(assignments[0].assigned_at_epoch_seconds(), &0u64);
    }
}
