// ---------------- [ File: osx-wallpaper-cycler/src/dropbox_list_folder_failure_probe.rs ]
crate::ix!();

pub fn should_run_dropbox_list_folder_failure_probe_once() -> bool {
    use std::sync::atomic::{AtomicBool, Ordering};

    static DID_PROBE: AtomicBool = AtomicBool::new(false);
    !DID_PROBE.swap(true, Ordering::SeqCst)
}

pub fn build_dropbox_list_folder_probe_paths_for_root(provided_root: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut push_unique = |s: String| {
        if seen.insert(s.clone()) {
            out.push(s);
        }
    };

    let raw = provided_root;
    let trimmed = raw.trim();
    let trimmed_no_trailing = trimmed.trim_end_matches('/');

    push_unique("".to_string());
    push_unique("/".to_string());
    push_unique("/Apps".to_string());

    push_unique(raw.to_string());
    push_unique(trimmed.to_string());
    push_unique(trimmed_no_trailing.to_string());

    if trimmed == "/" || trimmed_no_trailing == "/" {
        push_unique("".to_string());
    }

    if !trimmed.is_empty() && !trimmed.starts_with('/') {
        push_unique(format!("/{}", trimmed));
    }

    if trimmed.starts_with('/') {
        if let Some(without) = trimmed.strip_prefix('/') {
            push_unique(without.to_string());
        }
    }

    if trimmed_no_trailing.starts_with('/') {
        if let Some(without) = trimmed_no_trailing.strip_prefix('/') {
            push_unique(without.to_string());
        }
    }

    out.retain(|p| {
        let s = p.as_str();
        !(s.is_empty() && raw != "" && trimmed != "" && trimmed_no_trailing != "")
    });

    out
}

pub async fn debug_list_dropbox_folder_entries_shallow(
    client: &DropboxApiClient,
    token: &str,
    probe_path: &str,
    limit: usize,
) -> Result<(), WallpaperRotatorError> {
    let endpoint = DropboxEndpointKind::ListFolder;

    let url = format!(
        "{}/2/files/list_folder",
        client.endpoints().api_base_url()
    );

    let request_json = serde_json::json!({
        "path": probe_path,
        "recursive": false,
        "include_deleted": false,
        "limit": limit,
    });

    let payload = serde_json::to_vec(&request_json)
        .map_err(|e| WallpaperRotatorError::JsonEncodeFailure { source: e })?;

    tracing::debug!(
        endpoint = %endpoint,
        probe_path = %probe_path,
        payload_bytes = payload.len(),
        "dropbox debug probe: sending list_folder request"
    );

    let resp = client
        .http()
        .post(url)
        .bearer_auth(token)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(payload)
        .send()
        .await
        .map_err(|e| WallpaperRotatorError::HttpFailure { source: e })?;

    let request_id = extract_dropbox_request_id_from_headers(resp.headers());
    let status = resp.status();

    if !status.is_success() {
        let body = parse_json_body_or_none(resp).await;

        tracing::warn!(
            endpoint = %endpoint,
            probe_path = %probe_path,
            status = %status,
            request_id = request_id.as_deref(),
            error_body = ?body,
            request_json = ?request_json,
            "dropbox debug probe: list_folder returned non-success"
        );

        return Err(wallpaper_rotator_dropbox_api_failure(
            endpoint, status, request_id, body,
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
        .cloned()
        .unwrap_or_default();

    let has_more = v.get("has_more").and_then(|x| x.as_bool()).unwrap_or(false);
    let cursor_present = v.get("cursor").and_then(|x| x.as_str()).is_some();

    let mut folder_count: usize = 0;
    let mut file_count: usize = 0;
    let mut other_count: usize = 0;

    let mut folder_samples: Vec<String> = Vec::new();
    let mut file_samples: Vec<String> = Vec::new();
    let mut other_samples: Vec<String> = Vec::new();

    for e in entries.iter() {
        let tag = e.get(".tag").and_then(|x| x.as_str()).unwrap_or("");
        match tag {
            "folder" => folder_count += 1,
            "file" => file_count += 1,
            _ => other_count += 1,
        }
    }

    for e in entries.iter().take(limit) {
        let tag = e.get(".tag").and_then(|x| x.as_str()).unwrap_or("");
        let name = e
            .get("name")
            .and_then(|x| x.as_str())
            .unwrap_or("<missing-name>");
        let path_lower = e
            .get("path_lower")
            .and_then(|x| x.as_str())
            .unwrap_or("<missing-path_lower>");
        let id = e.get("id").and_then(|x| x.as_str()).unwrap_or("");

        let line = if id.is_empty() {
            format!("{tag} name={name} path_lower={path_lower}")
        } else {
            format!("{tag} name={name} path_lower={path_lower} id={id}")
        };

        match tag {
            "folder" => folder_samples.push(line),
            "file" => file_samples.push(line),
            _ => other_samples.push(line),
        }
    }

    tracing::info!(
        endpoint = %endpoint,
        probe_path = %probe_path,
        entries_returned = entries.len(),
        has_more,
        cursor_present,
        folders = folder_count,
        files = file_count,
        other = other_count,
        "dropbox debug probe: list_folder succeeded"
    );

    tracing::debug!(
        probe_path = %probe_path,
        folder_samples = ?folder_samples,
        file_samples = ?file_samples,
        other_samples = ?other_samples,
        "dropbox debug probe: sample entries"
    );

    Ok(())
}

pub async fn debug_log_dropbox_list_folder_probe_for_root(
    client: &DropboxApiClient,
    token: &str,
    provided_root: &str,
) {
    const PROBE_LIMIT: usize = 25;
    const MAX_PROBE_PATHS: usize = 12;

    let probe_paths = build_dropbox_list_folder_probe_paths_for_root(provided_root);

    tracing::info!(
        provided_root = %provided_root,
        probe_paths = ?probe_paths,
        probe_limit = PROBE_LIMIT,
        "dropbox list_folder debug probe: starting"
    );

    for probe_path in probe_paths.into_iter().take(MAX_PROBE_PATHS) {
        match debug_list_dropbox_folder_entries_shallow(client, token, &probe_path, PROBE_LIMIT).await {
            Ok(()) => {
                tracing::info!(
                    probe_path = %probe_path,
                    "dropbox list_folder debug probe: probe path succeeded"
                );
            }
            Err(e) => {
                tracing::warn!(
                    probe_path = %probe_path,
                    error = %e,
                    "dropbox list_folder debug probe: probe path failed"
                );
            }
        }
    }

    tracing::info!(
        provided_root = %provided_root,
        "dropbox list_folder debug probe: complete"
    );
}
