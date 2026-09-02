// ---------------- [ File: osx-wallpaper-cycler/src/perform_wallpaper_rotation_cycle_once.rs ]
crate::ix!();

pub async fn perform_wallpaper_rotation_cycle_once(
    cfg: &WallpaperRotatorConfig,
    source: std::sync::Arc<dyn DropboxWallpaperSource>,
    controller: std::sync::Arc<dyn DesktopWallpaperController>,
) -> Result<(), WallpaperRotatorError> {
    cfg.validate_or_error()?;

    let cache_dir = cfg.effective_cache_dir()?;
    ensure_wallpaper_cache_dir_ready(&cache_dir).await?;

    let prior_state = load_wallpaper_cache_state_if_present(&cache_dir).await?;

    let allowed = cfg.normalized_allowed_extensions_set();
    let scanned_roots = cfg.dropbox_roots().clone();
    let allowed_vec: Vec<String> = allowed.iter().cloned().collect();

    tracing::info!(
        roots = ?cfg.dropbox_roots(),
        allowed_extensions = ?allowed_vec,
        scope = %cfg.scope(),
        cache_dir = %cache_dir.display(),
        "starting wallpaper rotation cycle"
    );

    let mut loaded_from_directory_cache: bool = false;
    let mut candidates: Vec<DropboxWallpaperCandidate> = Vec::new();

    if *cfg.force_dropbox_directory_cache_rescan() {
        tracing::info!(
            roots = ?cfg.dropbox_roots(),
            "dropbox directory cache rescan forced by config"
        );
    } else {
        match load_dropbox_directory_cache_state_if_present(&cache_dir).await {
            Ok(Some(state)) => {
                let schema_ok = *state.schema_version() == 1u32;
                let roots_ok = state.is_compatible_with_roots(cfg.dropbox_roots());

                if schema_ok && roots_ok {
                    candidates = state.entries().clone();
                    loaded_from_directory_cache = true;

                    tracing::info!(
                        entries = candidates.len(),
                        cached_at_epoch_seconds = state.cached_at_epoch_seconds(),
                        "using cached dropbox directory index"
                    );
                } else {
                    tracing::info!(
                        schema_ok,
                        roots_ok,
                        cached_schema_version = state.schema_version(),
                        cached_roots = ?state.roots(),
                        current_roots = ?cfg.dropbox_roots(),
                        "dropbox directory cache is not compatible with current config; rescanning"
                    );
                }
            }
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    path = %DropboxDirectoryCacheState::cache_file_path(&cache_dir).display(),
                    "failed to load dropbox directory cache; rescanning"
                );
            }
        }
    }

    if !loaded_from_directory_cache {
        candidates = source
            .list_wallpaper_candidates_for_roots(cfg.dropbox_roots())
            .await?;

        tracing::info!(
            candidates = candidates.len(),
            roots = ?cfg.dropbox_roots(),
            "scanned dropbox roots for wallpaper candidates"
        );
    }

    let mut dedup: std::collections::HashMap<String, DropboxWallpaperCandidate> =
        std::collections::HashMap::new();
    for c in candidates.drain(..) {
        dedup.entry(c.id().to_string()).or_insert(c);
    }
    let mut candidates: Vec<DropboxWallpaperCandidate> = dedup.into_values().collect();

    if !loaded_from_directory_cache {
        let state = build_dropbox_directory_cache_state(
            cfg.dropbox_roots().clone(),
            candidates.clone(),
            std::time::SystemTime::now(),
        );

        match persist_dropbox_directory_cache_state(&cache_dir, &state).await {
            Ok(()) => {
                tracing::debug!(
                    path = %DropboxDirectoryCacheState::cache_file_path(&cache_dir).display(),
                    entries = state.entries().len(),
                    "persisted dropbox directory cache"
                );
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    path = %DropboxDirectoryCacheState::cache_file_path(&cache_dir).display(),
                    "failed to persist dropbox directory cache"
                );
            }
        }
    }

    candidates.retain(|c| {
        if let Some(ext) = c.file_extension_lowercase() {
            allowed.contains(&ext)
        } else {
            false
        }
    });

    if candidates.is_empty() {
        return Err(WallpaperRotatorError::NoEligibleWallpapers {
            scanned_roots,
            allowed_extensions: allowed_vec,
        });
    }

    let raw_desktop_count = match cfg.scope() {
        WallpaperRotatorScope::AllDesktops => controller.query_desktop_count()?,
        WallpaperRotatorScope::ActiveDesktopOnly => 1,
    };
    let target_count = if raw_desktop_count == 0 { 1 } else { raw_desktop_count };

    tracing::debug!(
        raw_desktop_count,
        target_count,
        candidates = candidates.len(),
        "computed wallpaper target count"
    );

    let selected = select_distinct_wallpapers_for_targets(&candidates, target_count);

    if selected.is_empty() {
        return Err(WallpaperRotatorError::NoEligibleWallpapers {
            scanned_roots,
            allowed_extensions: allowed_vec,
        });
    }

    if selected.len() < target_count {
        tracing::warn!(
            target_count,
            selected = selected.len(),
            "not enough unique wallpapers; desktop assignment will reuse images"
        );
    }

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(*cfg.concurrency()));
    let mut joinset: tokio::task::JoinSet<
        Result<(usize, DropboxWallpaperCandidate, std::path::PathBuf), WallpaperRotatorError>,
    > = tokio::task::JoinSet::new();

    for (idx, remote) in selected.iter().cloned().enumerate() {
        let sem = semaphore.clone();
        let src = source.clone();
        let dest = compute_wallpaper_cache_path_for_remote(&cache_dir, &remote);

        joinset.spawn(async move {
            let _permit = sem
                .acquire_owned()
                .await
                .map_err(|_| WallpaperRotatorError::ConcurrencyPermitUnavailable)?;

            if tokio::fs::metadata(&dest).await.is_ok() {
                tracing::info!(
                    remote_id = %remote.id(),
                    path = %dest.display(),
                    "cache hit"
                );
                return Ok((idx, remote, dest));
            }

            src.download_remote_wallpaper_to_path(&remote, &dest).await?;
            Ok((idx, remote, dest))
        });
    }

    let mut download_results: Vec<(usize, DropboxWallpaperCandidate, std::path::PathBuf)> =
        Vec::new();
    while let Some(joined) = joinset.join_next().await {
        match joined {
            Ok(Ok(v)) => download_results.push(v),
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(WallpaperRotatorError::JoinFailure),
        }
    }

    download_results.sort_by_key(|(idx, _, _)| *idx);

    let mut selected_remote_ids: Vec<String> = Vec::new();
    let mut selected_cache_paths: Vec<std::path::PathBuf> = Vec::new();

    for (_, remote, path) in download_results.into_iter() {
        selected_remote_ids.push(remote.id().to_string());
        selected_cache_paths.push(path);
    }

    if selected_cache_paths.is_empty() {
        return Err(WallpaperRotatorError::NoEligibleWallpapers {
            scanned_roots,
            allowed_extensions: allowed_vec,
        });
    }

    let desktop_image_paths: Vec<std::path::PathBuf> = (0..target_count)
        .map(|i| selected_cache_paths[i % selected_cache_paths.len()].clone())
        .collect();

    let desktop_remote_ids: Vec<String> = (0..target_count)
        .map(|i| selected_remote_ids[i % selected_remote_ids.len()].clone())
        .collect();

    controller.apply_wallpapers_to_desktops(*cfg.scope(), &desktop_image_paths)?;

    let assignments = build_wallpaper_cache_assignments_for_desktop_targets(
        &desktop_image_paths,
        &desktop_remote_ids,
        std::time::SystemTime::now(),
    );
    let new_state = build_wallpaper_cache_state(assignments.clone());

    cleanup_wallpaper_cache_files_not_in_use(&cache_dir, prior_state.as_ref(), &assignments).await?;
    persist_wallpaper_cache_state(&cache_dir, &new_state).await?;

    tracing::info!(
        cache_dir = %cache_dir.display(),
        assigned_desktops = assignments.len(),
        unique_cached_files = selected_cache_paths.len(),
        "wallpaper rotation cycle complete"
    );

    Ok(())
}
