// ---------------- [ File: osx-wallpaper-cycler/src/dropbox_wallpaper_source.rs ]
crate::ix!();

#[async_trait::async_trait]
pub trait DropboxWallpaperSource: Send + Sync {
    async fn list_wallpaper_candidates_for_roots(
        &self,
        roots: &[String],
    ) -> Result<Vec<DropboxWallpaperCandidate>, WallpaperRotatorError>;

    async fn download_remote_wallpaper_to_path(
        &self,
        remote: &DropboxWallpaperCandidate,
        destination_path: &std::path::Path,
    ) -> Result<(), WallpaperRotatorError>;
}

#[async_trait::async_trait]
impl DropboxWallpaperSource for DropboxApiClient {
    async fn list_wallpaper_candidates_for_roots(
        &self,
        roots: &[String],
    ) -> Result<Vec<DropboxWallpaperCandidate>, WallpaperRotatorError> {
        let mut all: Vec<DropboxWallpaperCandidate> = Vec::new();
        for root in roots.iter() {
            let mut per_root = self.list_candidates_recursive_for_root(root).await?;
            all.append(&mut per_root);
        }
        Ok(all)
    }

    async fn download_remote_wallpaper_to_path(
        &self,
        remote: &DropboxWallpaperCandidate,
        destination_path: &std::path::Path,
    ) -> Result<(), WallpaperRotatorError> {
        let token = self.ensure_access_token().await?;
        let endpoint = DropboxEndpointKind::Download;

        let arg = serde_json::json!({ "path": remote.path_lower() }).to_string();
        let tmp_path = destination_path.with_extension("partial");

        if let Some(parent) = tmp_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                wallpaper_rotator_io_failure(
                    FilesystemOperationKind::CreateDirAll,
                    parent.to_path_buf(),
                    e,
                )
            })?;
        }

        tracing::debug!(
            endpoint = %endpoint,
            remote_id = %remote.id(),
            remote_path = %remote.path_lower(),
            dest = %destination_path.display(),
            "starting download"
        );

        let url = format!("{}/2/files/download", self.endpoints().content_base_url());
        let resp = self
            .http()
            .post(url)
            .bearer_auth(token)
            .header("Dropbox-API-Arg", arg)
            .send()
            .await
            .map_err(|e| WallpaperRotatorError::HttpFailure { source: e })?;

        let request_id = extract_dropbox_request_id_from_headers(resp.headers());
        let status = resp.status();

        if !status.is_success() {
            let body = parse_json_body_or_none(resp).await;
            tracing::warn!(
                endpoint = %endpoint,
                status = %status,
                request_id = request_id.as_deref(),
                body_present = body.is_some(),
                "download failed"
            );
            return Err(wallpaper_rotator_dropbox_api_failure(
                endpoint,
                status,
                request_id,
                body,
            ));
        }

        let body_bytes = resp
            .bytes()
            .await
            .map_err(|e| WallpaperRotatorError::HttpFailure { source: e })?;

        let total_bytes: usize = body_bytes.len();

        let mut file = tokio::fs::File::create(&tmp_path).await.map_err(|e| {
            wallpaper_rotator_io_failure(FilesystemOperationKind::CreateFile, tmp_path.clone(), e)
        })?;

        use tokio::io::AsyncWriteExt;

        file.write_all(body_bytes.as_ref()).await.map_err(|e| {
            wallpaper_rotator_io_failure(FilesystemOperationKind::WriteFile, tmp_path.clone(), e)
        })?;

        file.flush().await.map_err(|e| {
            wallpaper_rotator_io_failure(FilesystemOperationKind::FlushFile, tmp_path.clone(), e)
        })?;
        drop(file);

        tokio::fs::rename(&tmp_path, destination_path).await.map_err(|e| {
            wallpaper_rotator_io_failure(
                FilesystemOperationKind::Rename,
                destination_path.to_path_buf(),
                e,
            )
        })?;

        tracing::info!(
            endpoint = %endpoint,
            remote_id = %remote.id(),
            bytes = total_bytes,
            dest = %destination_path.display(),
            "download complete"
        );

        Ok(())
    }
}
