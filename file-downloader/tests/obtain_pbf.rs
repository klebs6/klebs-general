use httpmock::prelude::*;
use file_downloader::*;
use tokio::io::AsyncReadExt;
use tempfile::tempdir;

#[tokio::test]
async fn test_obtain_pbf_success() {
    let server = MockServer::start_async().await;

    // The mock PBF file content
    let pbf_content = b"FAKEOSMPBF";
    server.mock(|when, then| {
        when.method(GET).path("/planet.osm.pbf");
        then.status(200)
            .body(pbf_content);
    });

    // MD5 for "FAKEOSMPBF" is 1a02c0b285ebcc9a3df52b145e69afe1
    let correct_md5 = "4dad4532fb93e5ba3a846fca746280c9";
    server.mock(|when, then| {
        when.method(GET).path("/planet.osm.pbf.md5");
        then.status(200)
            .body(format!("{}  planet.osm.pbf\n", correct_md5));
    });

    let dir = tempdir().unwrap();

    // Construct valid absolute URLs
    let pbf_url = server.url("/planet.osm.pbf");
    let md5_url = server.url("/planet.osm.pbf.md5");

    // Download & verify MD5
    let path = find_file_locally_or_download_into(&pbf_url, Some(&md5_url), dir.path())
    .await
        .unwrap();

    // Confirm the file matches our mock data
    let mut file = tokio::fs::File::open(&path).await.unwrap();
    let mut downloaded_content = Vec::new();
    file.read_to_end(&mut downloaded_content).await.unwrap();

    assert_eq!(downloaded_content, pbf_content);
}

#[tokio::test]
async fn test_obtain_pbf_md5_mismatch() {
    let server = MockServer::start_async().await;

    let pbf_content = b"FAKEOSMPBF";
    server.mock(|when, then| {
        when.method(GET).path("/planet.osm.pbf");
        then.status(200)
            .body(pbf_content);
    });

    // A deliberately "wrong" MD5
    let mismatch_md5 = "WRONGMD5";
    server.mock(|when, then| {
        when.method(GET).path("/planet.osm.pbf.md5");
        then.status(200)
            .body(format!("{}  planet.osm.pbf\n", mismatch_md5));
    });

    let dir = tempdir().unwrap();

    let pbf_url = server.url("/planet.osm.pbf");
    let md5_url = server.url("/planet.osm.pbf.md5");

    // We expect this call to fail
    let result = find_file_locally_or_download_into(&pbf_url, Some(&md5_url), dir.path()).await;
    assert!(result.is_err());

    // Verify it's specifically an Md5ChecksumVerificationError
    if let Err(e) = result {
        match e {
            DownloadError::Md5ChecksumVerificationError(_) => {
                // This is what we want
            }
            _ => panic!("Unexpected error variant"),
        }
    }
}
