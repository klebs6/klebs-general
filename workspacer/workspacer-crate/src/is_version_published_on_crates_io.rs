// ---------------- [ File: src/is_version_published_on_crates_io.rs ]
crate::ix!();

/// Checks crates.io to see if `<crate_name>@<crate_version>` is published.
/// If a successful (200) response is returned, we assume it is published.
pub async fn is_crate_version_published_on_crates_io(
    crate_name: &str,
    crate_version: &semver::Version,
) -> Result<bool, WorkspaceError> {
    let url = format!(
        "https://crates.io/api/v1/crates/{}/{}",
        crate_name, crate_version
    );
    debug!("Checking crates.io for {}@{} ...", crate_name, crate_version);

    // Using reqwest or a similar HTTP client. If the user uses `workspacer_3p` or
    // some built-in HTTP utilities, adapt accordingly. For simplicity, show `reqwest`.
    let resp = reqwest::get(&url).await.map_err(|e| {
        CrateError::FailedCratesIoCheck {
            crate_name: crate_name.to_string(),
            crate_version: crate_version.clone(),
            error: Arc::new(e),
        }
    })?;

    Ok(resp.status() == reqwest::StatusCode::OK)
}
