use file_downloader::*;
use tokio::io::AsyncWriteExt;
use tempfile::tempdir;

#[tokio::test]
async fn test_compute_md5_known_content() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut file = tokio::fs::File::create(&file_path).await.unwrap();
    file.write_all(b"hello world").await.unwrap();
    drop(file);

    let computed = compute_md5(&file_path).await.unwrap();
    // MD5 of "hello world" is "5eb63bbbe01eeed093cb22bb8f5acdc3"
    assert_eq!(computed, "5eb63bbbe01eeed093cb22bb8f5acdc3");
}

#[tokio::test]
async fn test_verify_md5_checksum_match() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut file = tokio::fs::File::create(&file_path).await.unwrap();
    file.write_all(b"hello world").await.unwrap();
    drop(file);

    let result = verify_md5_checksum(&file_path, "5eb63bbbe01eeed093cb22bb8f5acdc3").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_verify_md5_checksum_mismatch() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut file = tokio::fs::File::create(&file_path).await.unwrap();
    file.write_all(b"hello world").await.unwrap();
    drop(file);

    let result = verify_md5_checksum(&file_path, "wronghash").await;
    assert!(result.is_err());
    if let Err(e) = result {
        match e {
            Md5ChecksumVerificationError::ChecksumMismatch { expected, actual } => {
                assert_eq!(expected, "wronghash");
                assert_eq!(actual, "5eb63bbbe01eeed093cb22bb8f5acdc3");
            },
            _ => panic!("Unexpected error type"),
        }
    }
}
