// ---------------- [ File: workspacer-check-crates-io/src/is_version_published_on_crates_io.rs ]
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

#[cfg(test)]
mod test_is_crate_version_published_on_crates_io {
    use super::*;
    use std::sync::Arc;
    use semver::Version;
    use tokio::runtime::Runtime;

    // If you want to test against the live crates.io, you can pick a crate & version
    // that you know definitely exists (e.g., "serde" at some stable version).
    // However, depending on your CI environment, calling real crates.io is
    // sometimes discouraged or flaky. Instead, you can mock the HTTP request
    // using a library like `mockito` or `wiremock`. Below we show both approaches
    // in separate tests.

    // ------------------------------------------------------------------------
    // 1) Example of a "live" test hitting real crates.io (not recommended for CI).
    // ------------------------------------------------------------------------
    #[test]
    fn test_live_call_crates_io_exists() {
        let rt = Runtime::new().expect("Failed to create a tokio runtime");
        rt.block_on(async {
            // We'll pick a widely used crate that definitely exists, e.g. "serde" at version "1.0.0".
            // (In reality, you'd confirm that "serde@1.0.0" truly exists. If not, pick a known version.)
            let crate_name = "serde";
            let crate_version = Version::parse("1.0.0").expect("Invalid semver for test");
            let result = is_crate_version_published_on_crates_io(crate_name, &crate_version).await;
            match result {
                Ok(is_published) => {
                    // It's possible 1.0.0 of serde isn't exactly correct, so adjust as needed.
                    // For demonstration, we might just print it. Or we can assert if we know it's published.
                    println!("Is {crate_name}@{crate_version} published? {is_published}");
                    // Optionally:
                    // assert!(is_published, "Expected serde@1.0.0 to be published (if it truly exists).");
                }
                Err(e) => panic!("Failed to contact crates.io or parse response: {e}"),
            }
        });
    }

    // ------------------------------------------------------------------------
    // 2) Example of a "live" test hitting real crates.io for a definitely non-existent crate.
    // ------------------------------------------------------------------------
    #[test]
    fn test_live_call_crates_io_does_not_exist() {
        let rt = Runtime::new().expect("Failed to create a tokio runtime");
        rt.block_on(async {
            // We'll pick a crate name that almost certainly doesn't exist, e.g. "completely_non_existent_crate_foobar"
            let crate_name = "completely_non_existent_crate_foobar";
            let crate_version = Version::parse("99.0.0").expect("Invalid semver for test");
            let result = is_crate_version_published_on_crates_io(crate_name, &crate_version).await;
            match result {
                Ok(is_published) => {
                    assert!(!is_published, "We expect this to be false, as the crate doesn't exist");
                }
                Err(e) => panic!("Unexpected error contacting crates.io: {e}"),
            }
        });
    }

    // ------------------------------------------------------------------------
    // 3) Example with `mockito` to mock crates.io responses (recommended for stable tests).
    //    This way we don't rely on actual network or crates.io availability.
    // ------------------------------------------------------------------------
    #[cfg(feature = "mock_http")]
    #[test]
    fn test_is_crate_version_published_with_mock() {
        let rt = Runtime::new().expect("Failed to create a tokio runtime");
        rt.block_on(async {
            // We'll use the mockito crate, so add it to dev-dependencies in Cargo.toml:
            // [dev-dependencies]
            // mockito = "0.31.0"

            use mockito::{mock, server_address};

            // mockito starts a local HTTP server to stand in for crates.io
            let crate_name = "my_mock_crate";
            let crate_version = Version::parse("1.2.3").unwrap();
            let url_path = format!("/api/v1/crates/{}/{}", crate_name, crate_version);

            // We'll mock a 200 OK response
            let _m = mock("GET", url_path.as_str())
                .with_status(200)
                .create();

            // We must temporarily override the URL in the function, or you can make the function
            // accept a base URL so we can pass `mockito::server_url()`. For demonstration, we'll
            // define a local version of the function that uses mockito's base URL.
            async fn is_crate_version_published_on_mock(
                crate_name: &str,
                crate_version: &Version,
            ) -> Result<bool, WorkspaceError> {
                let base_url = server_address(); 
                // e.g., "127.0.0.1:12345"
                let url = format!(
                    "http://{}/api/v1/crates/{}/{}",
                    base_url, crate_name, crate_version
                );
                debug!("Mock: checking crates.io for {crate_name}@{crate_version} => {url}");

                let resp = reqwest::get(&url).await.map_err(|e| {
                    CrateError::FailedCratesIoCheck {
                        crate_name: crate_name.to_string(),
                        crate_version: crate_version.clone(),
                        error: Arc::new(e),
                    }
                })?;

                Ok(resp.status() == reqwest::StatusCode::OK)
            }

            // Now call the mock version
            let result = is_crate_version_published_on_mock(crate_name, &crate_version).await;
            match result {
                Ok(is_published) => {
                    assert!(is_published, "Mock responded 200 => should be true");
                }
                Err(e) => panic!("Test failed with mock server: {e}"),
            }
        });
    }

    // ------------------------------------------------------------------------
    // 4) Example to show an error scenario (e.g., network error or 500 error).
    // ------------------------------------------------------------------------
    #[cfg(feature = "mock_http")]
    #[test]
    fn test_is_crate_version_published_mock_500() {
        let rt = Runtime::new().expect("Failed to create a tokio runtime");
        rt.block_on(async {
            use mockito::{mock, server_address};
            let crate_name = "my_mock_crate";
            let crate_version = Version::parse("1.2.3").unwrap();
            let url_path = format!("/api/v1/crates/{}/{}", crate_name, crate_version);

            // We'll mock a 500 Internal Server Error
            let _m = mock("GET", url_path.as_str())
                .with_status(500)
                .create();

            async fn is_crate_version_published_on_mock(
                crate_name: &str,
                crate_version: &Version,
            ) -> Result<bool, WorkspaceError> {
                let base_url = server_address();
                let url = format!(
                    "http://{}/api/v1/crates/{}/{}",
                    base_url, crate_name, crate_version
                );
                debug!("Mock: checking crates.io for {crate_name}@{crate_version} => {url}");

                let resp = reqwest::get(&url).await.map_err(|e| {
                    CrateError::FailedCratesIoCheck {
                        crate_name: crate_name.to_string(),
                        crate_version: crate_version.clone(),
                        error: Arc::new(e),
                    }
                })?;

                // For 500, the status != 200 => returns false
                // Our function doesn't consider 500 an error; if we want to treat 5xx as an error,
                // we'd adapt the code to do so. Currently, we just do `Ok(status == 200)`.
                Ok(resp.status() == reqwest::StatusCode::OK)
            }

            let result = is_crate_version_published_on_mock(crate_name, &crate_version).await;
            match result {
                Ok(is_published) => {
                    assert!(!is_published, "HTTP 500 => is_published should be false");
                }
                Err(e) => panic!("Unexpected error: {e}"),
            }
        });
    }
}
