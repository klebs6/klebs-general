// ---------------- [ File: src/get_file_size.rs ]
crate::ix!();

#[async_trait]
pub trait GetFileSize {

    async fn file_size(&self) -> Result<u64, FileError>;
}

#[async_trait]
impl<T> GetFileSize for T
where
    T: AsRef<Path> + Send + Sync,
{
    async fn file_size(&self) -> Result<u64, FileError> {
        Ok(tokio::fs::metadata(self.as_ref())
            .await
            .map_err(|e| FileError::GetMetadataError { io: e.into() })?
            .len())
    }
}

#[cfg(test)]
mod test_get_file_size {
    use super::*;
    use std::io::Write as IoWrite;
    use std::path::{Path, PathBuf};
    use tempfile::{tempdir, NamedTempFile};
    use tokio::fs::{File, create_dir};
    use tokio::io::AsyncWriteExt;

    /// Verify that `file_size()` returns 0 for an empty file.
    #[tokio::test]
    async fn test_file_size_empty_file_returns_0() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_path_buf();

        let size = file_path.file_size().await;
        assert!(size.is_ok(), "Expected Ok for empty file metadata");
        assert_eq!(size.unwrap(), 0, "Empty file should have size 0");
    }

    /// Verify that `file_size()` returns correct file size for a file with content.
    #[tokio::test]
    async fn test_file_size_existing_file_with_content() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_path_buf();

        // Write some content
        let content = b"Hello, world!";
        temp_file
            .write_all(content)
            .expect("Failed to write content to temp file");

        let size = file_path.file_size().await;
        assert!(size.is_ok(), "Expected Ok for file with content");
        assert_eq!(
            size.unwrap() as usize,
            content.len(),
            "File size should match number of bytes written"
        );
    }

    /// Verify that `file_size()` returns an error for a non-existent file.
    #[tokio::test]
    async fn test_file_size_non_existent_file_returns_error() {
        let non_existent_path = PathBuf::from("this_file_does_not_exist.xyz");
        let result = non_existent_path.file_size().await;
        assert!(result.is_err(), "Expected an error for non-existent file");
        match result {
            Err(FileError::GetMetadataError { .. }) => {
                // This is the expected variant
            }
            _ => panic!("Expected FileError::GetMetadataError for non-existent file"),
        }
    }

    /// Verify behavior when calling `file_size()` on a directory.
    /// Generally, metadata for directories is valid, and it returns a size (which may be 0 or OS-specific).
    #[tokio::test]
    async fn test_file_size_directory() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let dir_path = temp_dir.path().to_path_buf();

        let size = dir_path.file_size().await;
        // Some filesystems set directory sizes to 0, others may set it to a block size, etc.
        // So we won't test for an exact numeric size. We'll just check it's Ok and >= 0.
        assert!(size.is_ok(), "Getting metadata for a directory should succeed on most platforms");
        let size_val = size.unwrap();
        assert!(
            size_val >= 0,
            "Directory size ({} bytes) should be non-negative",
            size_val
        );
    }

    /// (Optional) Test with a nested directory structure. Typically, the size might remain 0 or reflect
    /// an internal structure, depending on the filesystem. 
    #[tokio::test]
    async fn test_file_size_nested_directory() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let nested_dir = temp_dir.path().join("nested");
        create_dir(&nested_dir).await.expect("Failed to create nested dir");

        let size = nested_dir.file_size().await;
        assert!(size.is_ok(), "Getting metadata for nested directory should succeed");
    }

    /// (Optional) Large file test. We create a file with a certain size and verify it.
    /// For demonstration, we'll just do a slightly larger buffer (e.g., 1 MB). Adjust as needed.
    #[tokio::test]
    async fn test_file_size_large_file() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_path_buf();

        // Write 1 MB of data
        let data_size = 1_000_000usize;
        let data = vec![0u8; data_size];
        temp_file
            .write_all(&data)
            .expect("Failed to write large data to temp file");

        let size = file_path.file_size().await;
        assert!(size.is_ok(), "Expected Ok for large file metadata");
        assert_eq!(
            size.unwrap() as usize,
            data_size,
            "File size should match 1 MB of data"
        );
    }
}
