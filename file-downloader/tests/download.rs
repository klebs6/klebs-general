use httpmock::prelude::*;
use file_downloader::download_file;
use tokio::io::AsyncReadExt;
use tempfile::tempdir;

#[tokio::test]
async fn test_download_file_ok() {
    let server = MockServer::start_async().await;
    let content = b"OSM PBF CONTENT";
    let _mock = server.mock(|when, then| {
        when.method(GET).path("/test.pbf");
        then.status(200)
            .header("Content-Length", &content.len().to_string())
            .body(content);
    });

    let dir = tempdir().unwrap();
    let target_path = dir.path().join("test.pbf");

    download_file(&format!("{}test.pbf", server.url("/")), &target_path)
        .await
        .unwrap();

    let mut file = tokio::fs::File::open(&target_path).await.unwrap();
    let mut downloaded_content = Vec::new();
    file.read_to_end(&mut downloaded_content).await.unwrap();
    assert_eq!(downloaded_content, content);
}

#[tokio::test]
async fn test_download_file_404() {
    let server = MockServer::start_async().await;
    let _mock = server.mock(|when, then| {
        when.method(GET).path("/nonexistent.pbf");
        then.status(404);
    });

    let dir = tempdir().unwrap();
    let target_path = dir.path().join("nonexistent.pbf");

    let result = download_file(&format!("{}nonexistent.pbf", server.url("/")), &target_path).await;
    assert!(result.is_err());
    // You can match specific error variants if needed.
}
