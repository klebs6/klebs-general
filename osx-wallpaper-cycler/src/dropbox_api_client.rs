// ---------------- [ File: osx-wallpaper-cycler/src/dropbox_api_client.rs ]
crate::ix!();

#[derive(Debug, getset::Getters)]
#[getset(get = "pub")]
pub struct DropboxApiClient {
    http: reqwest::Client,
    endpoints: DropboxApiClientEndpoints,
    app_key: String,
    app_secret: Option<String>,
    refresh_token: String,
    token_state: tokio::sync::Mutex<Option<DropboxAccessTokenState>>,
}

impl DropboxApiClient {
    pub fn new(
        app_key: String,
        app_secret: Option<String>,
        refresh_token: String,
    ) -> Result<Self, WallpaperRotatorError> {
        Self::new_with_endpoints(
            app_key,
            app_secret,
            refresh_token,
            DropboxApiClientEndpoints::default_dropbox(),
        )
    }

    pub fn new_with_endpoints(
        app_key: String,
        app_secret: Option<String>,
        refresh_token: String,
        endpoints: DropboxApiClientEndpoints,
    ) -> Result<Self, WallpaperRotatorError> {
        let http = reqwest::Client::builder()
            .user_agent("dropbox-wallpaper-rotator/1.0")
            .build()
            .map_err(|e| WallpaperRotatorError::HttpFailure { source: e })?;

        Ok(Self {
            http,
            endpoints,
            app_key,
            app_secret,
            refresh_token,
            token_state: tokio::sync::Mutex::new(None),
        })
    }

    pub(crate) async fn ensure_access_token(&self) -> Result<String, WallpaperRotatorError> {
        {
            let guard = self.token_state.lock().await;
            if let Some(state) = guard.as_ref() {
                let now = std::time::Instant::now();
                let expires_at = *state.expires_at();
                if expires_at > now + std::time::Duration::from_secs(60) {
                    tracing::trace!("reusing cached access token");
                    return Ok(state.token().clone());
                }
            }
        }

        let endpoint = DropboxEndpointKind::OAuthToken;

        let mut form: Vec<(&str, String)> = vec![
            ("grant_type", "refresh_token".to_string()),
            ("refresh_token", self.refresh_token.clone()),
            ("client_id", self.app_key.clone()),
        ];
        if let Some(secret) = self.app_secret.as_ref() {
            form.push(("client_secret", secret.clone()));
        }

        tracing::debug!(endpoint = %endpoint, "refreshing access token");

        let url = format!("{}/oauth2/token", self.endpoints.oauth_base_url());
        let resp = self
            .http
            .post(url)
            .form(&form)
            .send()
            .await
            .map_err(|e| WallpaperRotatorError::HttpFailure { source: e })?;

        let request_id = extract_dropbox_request_id_from_headers(resp.headers());
        let status = resp.status();

        if !status.is_success() {
            let body = parse_json_body_or_none(resp).await;
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

        let v: serde_json::Value = serde_json::from_slice(body_bytes.as_ref())
            .map_err(|e| WallpaperRotatorError::JsonDecodeFailure { source: e })?;

        let access_token = v
            .get("access_token")
            .and_then(|x| x.as_str())
            .ok_or_else(|| {
                wallpaper_rotator_dropbox_protocol_violation(
                    endpoint,
                    DropboxProtocolViolationDetail::MissingAccessToken,
                )
            })?
            .to_string();

        let expires_in = v
            .get("expires_in")
            .and_then(|x| x.as_u64())
            .unwrap_or(4 * 60 * 60);

        let expires_at = std::time::Instant::now() + std::time::Duration::from_secs(expires_in);

        let mut guard = self.token_state.lock().await;
        *guard = Some(DropboxAccessTokenState::new(access_token.clone(), expires_at));

        tracing::info!(endpoint = %endpoint, expires_in, "refreshed access token");
        Ok(access_token)
    }

    pub(crate) async fn list_candidates_recursive_for_root(
        &self,
        root: &str,
    ) -> Result<Vec<DropboxWallpaperCandidate>, WallpaperRotatorError> {
        let token = self.ensure_access_token().await?;

        let mut cursor: Option<String> = None;
        let mut has_more: bool = true;
        let mut raw_entries: Vec<serde_json::Value> = Vec::new();

        while has_more {
            let (endpoint_kind, url, request_json) = if let Some(cur) = cursor.as_ref() {
                (
                    DropboxEndpointKind::ListFolderContinue,
                    format!("{}/2/files/list_folder/continue", self.endpoints.api_base_url()),
                    serde_json::json!({ "cursor": cur }),
                )
            } else {
                (
                    DropboxEndpointKind::ListFolder,
                    format!("{}/2/files/list_folder", self.endpoints.api_base_url()),
                    serde_json::json!({
                        "path": root,
                        "recursive": true,
                        "include_deleted": false,
                        "include_non_downloadable_files": false
                    }),
                )
            };

            let payload = serde_json::to_vec(&request_json)
                .map_err(|e| WallpaperRotatorError::JsonEncodeFailure { source: e })?;

            let root_trimmed = root.trim();
            tracing::debug!(
                endpoint = %endpoint_kind,
                root = %root,
                root_len = root.len(),
                root_is_empty = root.is_empty(),
                root_starts_with_slash = root.starts_with('/'),
                root_trimmed = %root_trimmed,
                root_trimmed_len = root_trimmed.len(),
                root_trimmed_is_empty = root_trimmed.is_empty(),
                root_trimmed_starts_with_slash = root_trimmed.starts_with('/'),
                cursor_present = cursor.is_some(),
                "listing folder page"
            );

            let resp = self
                .http
                .post(url.clone())
                .bearer_auth(&token)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(payload)
                .send()
                .await
                .map_err(|e| WallpaperRotatorError::HttpFailure { source: e })?;

            let request_id = extract_dropbox_request_id_from_headers(resp.headers());
            let status = resp.status();

            if !status.is_success() {
                let error_body = parse_json_body_or_none(resp).await;

                tracing::error!(
                    endpoint = %endpoint_kind,
                    url = %url,
                    root = %root,
                    root_len = root.len(),
                    root_is_empty = root.is_empty(),
                    root_starts_with_slash = root.starts_with('/'),
                    root_trimmed = %root_trimmed,
                    root_trimmed_len = root_trimmed.len(),
                    root_trimmed_is_empty = root_trimmed.is_empty(),
                    root_trimmed_starts_with_slash = root_trimmed.starts_with('/'),
                    cursor_present = cursor.is_some(),
                    status = %status,
                    request_id = request_id.as_deref(),
                    request_json = ?request_json,
                    error_body = ?error_body,
                    "dropbox list_folder failed"
                );

                let should_probe = endpoint_kind == DropboxEndpointKind::ListFolder
                    && cursor.is_none()
                    && status == reqwest::StatusCode::BAD_REQUEST
                    && tracing::enabled!(tracing::Level::DEBUG)
                    && should_run_dropbox_list_folder_failure_probe_once();

                tracing::info!("should_execute_list_folder_probe={}",should_probe);

                if should_probe {
                    debug_log_dropbox_list_folder_probe_for_root(self, &token, root).await;
                } else {
                    tracing::debug!(
                        endpoint = %endpoint_kind,
                        status = %status,
                        cursor_present = cursor.is_some(),
                        "dropbox list_folder debug probe not triggered for this failure"
                    );
                }

                return Err(wallpaper_rotator_dropbox_api_failure(
                    endpoint_kind,
                    status,
                    request_id,
                    error_body,
                ));
            }

            let body_bytes = resp
                .bytes()
                .await
                .map_err(|e| WallpaperRotatorError::HttpFailure { source: e })?;

            let v: serde_json::Value = serde_json::from_slice(body_bytes.as_ref())
                .map_err(|e| WallpaperRotatorError::JsonDecodeFailure { source: e })?;

            let entries = v
                .get("entries")
                .and_then(|x| x.as_array())
                .ok_or_else(|| {
                    wallpaper_rotator_dropbox_protocol_violation(
                        endpoint_kind,
                        DropboxProtocolViolationDetail::MissingEntries,
                    )
                })?;

            for e in entries.iter() {
                raw_entries.push(e.clone());
            }

            let new_cursor = v
                .get("cursor")
                .and_then(|x| x.as_str())
                .ok_or_else(|| {
                    wallpaper_rotator_dropbox_protocol_violation(
                        endpoint_kind,
                        DropboxProtocolViolationDetail::MissingCursor,
                    )
                })?
                .to_string();

            has_more = v.get("has_more").and_then(|x| x.as_bool()).unwrap_or(false);
            cursor = Some(new_cursor);
        }

        let mut candidates: Vec<DropboxWallpaperCandidate> = Vec::new();
        for entry in raw_entries.iter() {
            let tag = entry.get(".tag").and_then(|x| x.as_str()).unwrap_or("");
            if tag != "file" {
                continue;
            }

            let id = entry
                .get("id")
                .and_then(|x| x.as_str())
                .ok_or_else(|| {
                    wallpaper_rotator_dropbox_protocol_violation(
                        DropboxEndpointKind::ListFolder,
                        DropboxProtocolViolationDetail::MissingId,
                    )
                })?
                .to_string();

            let name = entry
                .get("name")
                .and_then(|x| x.as_str())
                .ok_or_else(|| {
                    wallpaper_rotator_dropbox_protocol_violation(
                        DropboxEndpointKind::ListFolder,
                        DropboxProtocolViolationDetail::MissingName,
                    )
                })?
                .to_string();

            let path_lower = entry
                .get("path_lower")
                .and_then(|x| x.as_str())
                .ok_or_else(|| {
                    wallpaper_rotator_dropbox_protocol_violation(
                        DropboxEndpointKind::ListFolder,
                        DropboxProtocolViolationDetail::MissingPathLower,
                    )
                })?
                .to_string();

            let candidate = DropboxWallpaperCandidateBuilder::default()
                .id(id)
                .name(name)
                .path_lower(path_lower)
                .build()
                .map_err(|_| {
                    wallpaper_rotator_dropbox_protocol_violation(
                        DropboxEndpointKind::ListFolder,
                        DropboxProtocolViolationDetail::InvalidJsonShape,
                    )
                })?;

            candidates.push(candidate);
        }

        tracing::info!(
            endpoint = %DropboxEndpointKind::ListFolder,
            root = %root,
            file_candidates = candidates.len(),
            "listed wallpaper candidates"
        );

        Ok(candidates)
    }
}
