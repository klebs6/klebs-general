// ---------------- [ File: osx-wallpaper-cycler/src/mock_desktop_wallpaper_source.rs ]
crate::ix!();

#[cfg(test)]
#[derive(Debug)]
pub struct MockDropboxWallpaperSource {
    candidates: Vec<DropboxWallpaperCandidate>,
    downloads: tokio::sync::Mutex<Vec<String>>,
    list_calls: std::sync::atomic::AtomicUsize,
    concurrent_now: std::sync::atomic::AtomicUsize,
    max_concurrent: std::sync::atomic::AtomicUsize,
    per_download_delay: std::time::Duration,
}

#[cfg(test)]
impl MockDropboxWallpaperSource {
    pub(crate) fn new(
        candidates: Vec<DropboxWallpaperCandidate>,
        per_download_delay: std::time::Duration,
    ) -> Self {
        Self {
            candidates,
            downloads: tokio::sync::Mutex::new(Vec::new()),
            list_calls: std::sync::atomic::AtomicUsize::new(0),
            concurrent_now: std::sync::atomic::AtomicUsize::new(0),
            max_concurrent: std::sync::atomic::AtomicUsize::new(0),
            per_download_delay,
        }
    }

    pub(crate) async fn download_ids(&self) -> Vec<String> {
        self.downloads.lock().await.clone()
    }

    pub(crate) fn max_concurrency_observed(&self) -> usize {
        self.max_concurrent
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    pub(crate) fn list_calls_observed(&self) -> usize {
        self.list_calls.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl DropboxWallpaperSource for MockDropboxWallpaperSource {
    async fn list_wallpaper_candidates_for_roots(
        &self,
        _roots: &[String],
    ) -> Result<Vec<DropboxWallpaperCandidate>, WallpaperRotatorError> {
        self.list_calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(self.candidates.clone())
    }

    async fn download_remote_wallpaper_to_path(
        &self,
        remote: &DropboxWallpaperCandidate,
        destination_path: &std::path::Path,
    ) -> Result<(), WallpaperRotatorError> {
        let now = self
            .concurrent_now
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1;

        loop {
            let prev = self
                .max_concurrent
                .load(std::sync::atomic::Ordering::SeqCst);
            if now <= prev {
                break;
            }
            if self
                .max_concurrent
                .compare_exchange(
                    prev,
                    now,
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                )
                .is_ok()
            {
                break;
            }
        }

        tokio::time::sleep(self.per_download_delay).await;

        if let Some(parent) = destination_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                wallpaper_rotator_io_failure(
                    FilesystemOperationKind::CreateDirAll,
                    parent.to_path_buf(),
                    e,
                )
            })?;
        }

        let payload = format!("BYTES:{}", remote.id());
        tokio::fs::write(destination_path, payload.as_bytes())
            .await
            .map_err(|e| {
                wallpaper_rotator_io_failure(
                    FilesystemOperationKind::WriteFile,
                    destination_path.to_path_buf(),
                    e,
                )
            })?;

        self.downloads.lock().await.push(remote.id().to_string());
        self.concurrent_now
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}
