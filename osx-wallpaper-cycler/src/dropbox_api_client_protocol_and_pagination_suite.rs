// ---------------- [ File: osx-wallpaper-cycler/src/dropbox_api_client_protocol_and_pagination_suite.rs ]
crate::ix!();

#[cfg(test)]
mod dropbox_api_client_protocol_and_pagination_suite {
    use super::*;

    #[traced_test]
    fn oauth_token_is_cached_across_calls_until_near_expiry() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let server = httpmock::MockServer::start();

            let oauth_mock = server.mock(|when, then| {
                when.method(httpmock::Method::POST).path("/oauth2/token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::json!({
                        "access_token": "test_token",
                        "expires_in": 3600,
                        "token_type": "bearer"
                    }));
            });

            let list_mock = server.mock(|when, then| {
                when.method(httpmock::Method::POST)
                    .path("/2/files/list_folder")
                    .header("authorization", "Bearer test_token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::json!({
                        "entries": [],
                        "cursor": "c0",
                        "has_more": false
                    }));
            });

            let endpoints = DropboxApiClientEndpointsBuilder::default()
                .api_base_url(server.base_url())
                .content_base_url(server.base_url())
                .oauth_base_url(server.base_url())
                .build()
                .unwrap();

            let client = DropboxApiClient::new_with_endpoints(
                "app_key".to_string(),
                None,
                "refresh".to_string(),
                endpoints,
            )
            .unwrap();

            let _ = client
                .list_wallpaper_candidates_for_roots(&vec!["/Wallpapers".to_string()])
                .await
                .unwrap();
            let _ = client
                .list_wallpaper_candidates_for_roots(&vec!["/Wallpapers".to_string()])
                .await
                .unwrap();

            assert_eq!(oauth_mock.calls(), 1);
            assert_eq!(list_mock.calls(), 2);
        });
    }

    #[traced_test]
    fn list_folder_pagination_is_followed_and_files_are_extracted() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let server = httpmock::MockServer::start();

            server.mock(|when, then| {
                when.method(httpmock::Method::POST).path("/oauth2/token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::json!({
                        "access_token": "test_token",
                        "expires_in": 3600
                    }));
            });

            server.mock(|when, then| {
                when.method(httpmock::Method::POST)
                    .path("/2/files/list_folder")
                    .header("authorization", "Bearer test_token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::json!({
                        "entries": [
                            { ".tag": "folder", "name": "sub", "id": "id:folder", "path_lower": "/wallpapers/sub" },
                            { ".tag": "file", "name": "a.jpg", "id": "id:a", "path_lower": "/wallpapers/a.jpg" }
                        ],
                        "cursor": "c1",
                        "has_more": true
                    }));
            });

            server.mock(|when, then| {
                when.method(httpmock::Method::POST)
                    .path("/2/files/list_folder/continue")
                    .header("authorization", "Bearer test_token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::json!({
                        "entries": [
                            { ".tag": "file", "name": "b.png", "id": "id:b", "path_lower": "/wallpapers/b.png" }
                        ],
                        "cursor": "c2",
                        "has_more": false
                    }));
            });

            let endpoints = DropboxApiClientEndpointsBuilder::default()
                .api_base_url(server.base_url())
                .content_base_url(server.base_url())
                .oauth_base_url(server.base_url())
                .build()
                .unwrap();

            let client = DropboxApiClient::new_with_endpoints(
                "app_key".to_string(),
                None,
                "refresh".to_string(),
                endpoints,
            )
            .unwrap();

            let files = client
                .list_wallpaper_candidates_for_roots(&vec!["/Wallpapers".to_string()])
                .await
                .unwrap();

            assert_eq!(files.len(), 2);
            let ids: std::collections::HashSet<String> =
                files.iter().map(|f| f.id().to_string()).collect();
            assert!(ids.contains("id:a"));
            assert!(ids.contains("id:b"));
        });
    }

    #[traced_test]
    fn list_folder_missing_entries_is_protocol_violation() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let server = httpmock::MockServer::start();

            server.mock(|when, then| {
                when.method(httpmock::Method::POST).path("/oauth2/token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::json!({
                        "access_token": "test_token",
                        "expires_in": 3600
                    }));
            });

            server.mock(|when, then| {
                when.method(httpmock::Method::POST)
                    .path("/2/files/list_folder")
                    .header("authorization", "Bearer test_token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::json!({
                        "cursor": "c1",
                        "has_more": false
                    }));
            });

            let endpoints = DropboxApiClientEndpointsBuilder::default()
                .api_base_url(server.base_url())
                .content_base_url(server.base_url())
                .oauth_base_url(server.base_url())
                .build()
                .unwrap();

            let client = DropboxApiClient::new_with_endpoints(
                "app_key".to_string(),
                None,
                "refresh".to_string(),
                endpoints,
            )
            .unwrap();

            let err = client
                .list_wallpaper_candidates_for_roots(&vec!["/Wallpapers".to_string()])
                .await
                .err()
                .unwrap();

            match err {
                WallpaperRotatorError::DropboxProtocolViolation { detail, .. } => {
                    assert_eq!(detail, DropboxProtocolViolationDetail::MissingEntries);
                }
                _ => panic!("unexpected error variant"),
            }
        });
    }

    #[traced_test]
    fn download_writes_file_content_and_renames_atomically() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let server = httpmock::MockServer::start();

            server.mock(|when, then| {
                when.method(httpmock::Method::POST).path("/oauth2/token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::json!({
                        "access_token": "test_token",
                        "expires_in": 3600
                    }));
            });

            server.mock(|when, then| {
                when.method(httpmock::Method::POST)
                    .path("/2/files/download")
                    .header("authorization", "Bearer test_token");
                then.status(200).body("IMAGE_BYTES");
            });

            let endpoints = DropboxApiClientEndpointsBuilder::default()
                .api_base_url(server.base_url())
                .content_base_url(server.base_url())
                .oauth_base_url(server.base_url())
                .build()
                .unwrap();

            let client = DropboxApiClient::new_with_endpoints(
                "app_key".to_string(),
                None,
                "refresh".to_string(),
                endpoints,
            )
            .unwrap();

            let dir = tempfile::TempDir::new().unwrap();
            let dest = dir.path().join("x.jpg");

            let remote = DropboxWallpaperCandidateBuilder::default()
                .id("id:x")
                .name("x.jpg")
                .path_lower("/wallpapers/x.jpg")
                .build()
                .unwrap();

            client
                .download_remote_wallpaper_to_path(&remote, &dest)
                .await
                .unwrap();

            let bytes = tokio::fs::read(&dest).await.unwrap();
            assert_eq!(bytes, b"IMAGE_BYTES");
        });
    }

    #[traced_test]
    fn download_non_success_status_maps_to_dropbox_api_failure_with_request_id() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let server = httpmock::MockServer::start();

            server.mock(|when, then| {
                when.method(httpmock::Method::POST).path("/oauth2/token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::json!({
                        "access_token": "test_token",
                        "expires_in": 3600
                    }));
            });

            server.mock(|when, then| {
                when.method(httpmock::Method::POST)
                    .path("/2/files/download")
                    .header("authorization", "Bearer test_token");
                then.status(409)
                    .header("x-dropbox-request-id", "RID123")
                    .header("content-type", "application/json")
                    .json_body(serde_json::json!({
                        "error_summary": "conflict",
                        "error": { ".tag": "path" }
                    }));
            });

            let endpoints = DropboxApiClientEndpointsBuilder::default()
                .api_base_url(server.base_url())
                .content_base_url(server.base_url())
                .oauth_base_url(server.base_url())
                .build()
                .unwrap();

            let client = DropboxApiClient::new_with_endpoints(
                "app_key".to_string(),
                None,
                "refresh".to_string(),
                endpoints,
            )
            .unwrap();

            let dir = tempfile::TempDir::new().unwrap();
            let dest = dir.path().join("x.jpg");

            let remote = DropboxWallpaperCandidateBuilder::default()
                .id("id:x")
                .name("x.jpg")
                .path_lower("/wallpapers/x.jpg")
                .build()
                .unwrap();

            let err = client
                .download_remote_wallpaper_to_path(&remote, &dest)
                .await
                .err()
                .unwrap();

            match err {
                WallpaperRotatorError::DropboxApiFailure {
                    request_id, status, ..
                } => {
                    assert_eq!(status, reqwest::StatusCode::CONFLICT);
                    assert_eq!(request_id.as_deref(), Some("RID123"));
                }
                _ => panic!("unexpected error variant"),
            }
        });
    }
}
