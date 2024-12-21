#![allow(dead_code)]
use std::borrow::Cow;
use httpmock::prelude::*;
use tempfile::tempdir;
use tracing::*;
use tracing_setup::*;
use tokio::io::AsyncReadExt;
use file_downloader::*; // replace with your crate name/path if needed

struct NoMd5Downloader {
    link: String,
    ext:  &'static str,
}

impl FileDownloader for NoMd5Downloader {

    fn download_link(&self) -> &str {
        &self.link
    }

    // Return None so we skip MD5 checks
    fn md5_download_link(&self) -> Option<Cow<'_, str>> {
        None
    }
}

#[tokio::test]
async fn test_obtain_pbf_no_md5() {
    configure_tracing();

    let server = MockServer::start_async().await;

    let pbf_content = b"FAKEOSMPBF";
    server.mock(|when, then| {
        when.method(GET)
            .path("/planet.osm.pbf");
        then.status(200)
            .body(pbf_content);
    });

    let pbf_url = server.url("/planet.osm.pbf");
    info!("Using PBF download link: {pbf_url}");

    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf(); // Keep the tempdir alive
    info!("Temporary directory: {}", dir_path.display());

    let path = NoMd5Downloader {
        link: pbf_url,
        ext: ".osm.pbf",
    }
    .find_file_locally_or_download(&dir_path)
    .await
    .expect("Download should succeed without MD5 checks");

    info!("Returned file path: {}", path.display());

    assert!(
        tokio::fs::metadata(&path).await.is_ok(),
        "File does not exist where expected: {}",
        path.display()
    );

    assert_eq!(
        path.file_name().unwrap().to_str().unwrap(),
        "planet.osm.pbf"
    );

    let mut file = tokio::fs::File::open(&path).await.unwrap();
    let mut downloaded_content = Vec::new();
    file.read_to_end(&mut downloaded_content).await.unwrap();
    assert_eq!(downloaded_content, pbf_content);
}
