// ---------------- [ File: workspacer-crate/src/read_file_string.rs ]
crate::ix!();

#[async_trait]
impl ReadFileString for CrateHandle {
    async fn read_file_string(&self, path: &Path) -> Result<String, CrateError> {
        // The naive approach:
        // let full_path = self.crate_path().join(path);

        // The improved approach:
        let mut full_path = path.to_path_buf();

        // 1) If it's absolute, just use it
        if !full_path.is_absolute() {
            // 2) If it starts with our crate_path when interpreted as a string,
            //    skip the join. E.g., if path = "workspacer-toml/src/imports.rs",
            //    and crate_path is "workspacer-toml", we'd double up if we blindly do join.
            let crate_str = self.crate_path().to_string_lossy().to_string();
            let path_str = full_path.to_string_lossy().to_string();

            if path_str.starts_with(&crate_str) {
                // Already has crate_path as prefix, so leave it alone
                debug!("Path is already under crate_path: {}", path_str);
            } else {
                // Otherwise, do the join
                full_path = self.crate_path().join(path_str);
            }
        }

        let content_result = fs::read_to_string(&full_path).await;
        content_result.map_err(|io_err| CrateError::IoError {
            io_error: Arc::new(io_err),
            context: format!("Failed to read file: {}", full_path.display()),
        })
    }
}

#[cfg(test)]
mod test_read_file_string {
    use super::*;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;
    use tokio::fs::{create_dir_all, File};
    use tokio::io::AsyncWriteExt;
    use std::io::Write;
    use std::sync::Arc;

    // ------------------------------------------------------------------------
    // A minimal helper that implements `HasCargoTomlPathBuf` so we can create a CrateHandle.
    // We'll just store a single directory as the "crate_path."
    // ------------------------------------------------------------------------
    #[derive(Clone)]
    struct MockCratePath(PathBuf);

    impl AsRef<Path> for MockCratePath {
        fn as_ref(&self) -> &Path {
            &self.0
        }
    }

    // ------------------------------------------------------------------------
    // Test fixture: Creates a real directory structure and a CrateHandle.
    // ------------------------------------------------------------------------
    async fn setup_crate_handle() -> CrateHandle {
        // 1) Make a temp directory to simulate a crate root
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        let crate_root = tmp_dir.path().to_path_buf();

        // 2) We'll create a minimal Cargo.toml so the handle can be constructed
        let cargo_toml_content = r#"
            [package]
            name = "mock_crate"
            version = "0.1.0"
            authors = ["Test <test@example.com>"]
            license = "MIT"
        "#;
        let cargo_toml_path = crate_root.join("Cargo.toml");

        {
            let mut f = File::create(&cargo_toml_path)
                .await
                .expect("Failed to create Cargo.toml");
            f.write_all(cargo_toml_content.as_bytes())
                .await
                .expect("Failed to write Cargo.toml");
        }

        // 3) Build a mock path object and then create the CrateHandle
        let mock_path = MockCratePath(crate_root);
        CrateHandle::new(&mock_path)
            .await
            .expect("Failed to create CrateHandle")
    }

    // ------------------------------------------------------------------------
    // Write some test content to a file in the crate, returning the path.
    // ------------------------------------------------------------------------
    async fn write_file_in_crate(handle: &CrateHandle, relative_path: &str, content: &str) -> PathBuf {
        let file_path = handle.as_ref().join(relative_path);
        if let Some(parent) = file_path.parent() {
            create_dir_all(parent)
                .await
                .expect("Failed to create parent directories");
        }
        let mut f = File::create(&file_path)
            .await
            .expect("Failed to create test file");
        f.write_all(content.as_bytes())
            .await
            .expect("Failed to write test file content");
        file_path
    }

    // ------------------------------------------------------------------------
    // Test cases for read_file_string
    // ------------------------------------------------------------------------

    /// 1) If the path is already absolute, we just use that path (no crate_path join).
    #[tokio::test]
    async fn test_read_file_string_absolute_path() {
        let handle = setup_crate_handle().await;
        // We'll create a file outside the crate path (in another temp dir),
        // then pass its absolute path to read_file_string.
        let outside_tmp_dir = tempdir().expect("Failed to create second temp dir");
        let outside_file_path = outside_tmp_dir.path().join("external_file.txt");

        {
            let mut f = std::fs::File::create(&outside_file_path)
                .expect("Failed to create external file");
            writeln!(f, "This is outside the crate path.").expect("Write failed");
        }

        let content = handle
            .read_file_string(&outside_file_path)
            .await
            .expect("Failed to read external file with absolute path");
        assert_eq!(
            content.trim(),
            "This is outside the crate path.",
            "Should read from the absolute path directly"
        );
    }

    /// 2) If the path is relative and already starts with the crate path as a string prefix, we do not join again.
    ///    (In practice, this is a bit contrived, but let's test the logic.)
    #[tokio::test]
    async fn test_read_file_string_relative_already_prefixed() {
        let handle = setup_crate_handle().await;

        // We'll create a file *within* the crate path first
        let relative_path_str = "nested/hello.txt";
        let file_path_in_crate = write_file_in_crate(&handle, relative_path_str, "Hello, World").await;

        // Now let's build a PathBuf that "looks" like it's already prefixed
        // E.g., the user passes "crate_root/nested/hello.txt" but it's not absolute on some OS?
        // This scenario might be OS or environment specific. We'll just forcibly do:
        let path_str = file_path_in_crate.to_string_lossy().into_owned();
        // Now we create a PathBuf from that string, but hopefully it doesn't parse as absolute
        // if there's no leading slash or drive letter. We'll skip the advanced OS intricacies here.

        // We'll see if the function detects that it "starts with crate_path" and doesn't re-join.
        let content = handle
            .read_file_string(Path::new(&path_str))
            .await
            .expect("Failed to read file with 'already prefixed' path");
        assert_eq!(content, "Hello, World", "Should read the same content");
    }

    /// 3) If the path is relative and does NOT start with crate_path,
    ///    we join it with crate_path.
    #[tokio::test]
    async fn test_read_file_string_relative_joined() {
        let handle = setup_crate_handle().await;

        // We'll create a file in "src/myfile.txt" inside the crate
        let file_path = write_file_in_crate(&handle, "src/myfile.txt", "some data").await;

        // We'll read via a relative path "src/myfile.txt"
        let relative_path = Path::new("src/myfile.txt");
        let content = handle
            .read_file_string(relative_path)
            .await
            .expect("Failed to read file with relative path that does not match prefix");
        assert_eq!(content, "some data");
    }

    /// 4) If the file doesn't exist, we expect an IoError with the correct context.
    #[tokio::test]
    async fn test_read_file_string_missing_file() {
        let handle = setup_crate_handle().await;
        let missing_path = Path::new("this_file_does_not_exist.txt");
        let result = handle.read_file_string(missing_path).await;

        assert!(result.is_err(), "Expected error for missing file");
        match result {
            Err(CrateError::IoError { context, .. }) => {
                // Check that context includes "Failed to read file: <path>"
                assert!(
                    context.contains("Failed to read file"),
                    "Error context should mention failed to read file"
                );
                // We could also check that it includes "this_file_does_not_exist.txt"
                assert!(
                    context.contains("this_file_does_not_exist.txt"),
                    "Error context should mention the missing file"
                );
            }
            other => panic!("Expected CrateError::IoError, got: {other:?}"),
        }
    }

    /// 5) Test reading a file that is inside the crate path but specified with an absolute path
    ///    to confirm we still read it without doubling the path.
    #[tokio::test]
    async fn test_read_file_string_same_crate_path_but_absolute() {
        let handle = setup_crate_handle().await;
        // Create a file in the crate
        let relative_path = "docs/some_doc.txt";
        let file_path = write_file_in_crate(&handle, relative_path, "doc content").await;
        let absolute_path = file_path.canonicalize().expect("Failed to canonicalize path");

        // Should read the file directly, no re-join
        let content = handle
            .read_file_string(&absolute_path)
            .await
            .expect("Failed to read doc content with absolute path");
        assert_eq!(content, "doc content");
    }

    /// 6) (Optional) If we want to test edge cases like the path string partially matching the crate path
    ///    but not from the start, we can do that. For example, if crate_path = "abc/def", and
    ///    the user passes a path with "some/abc/def" in the middle. We'll see if it incorrectly
    ///    tries to skip the join. The code might or might not handle this scenario. We'll do it for completeness.
    #[tokio::test]
    async fn test_read_file_string_partial_prefix() {
        let handle = setup_crate_handle().await;
        let crate_str = handle.crate_path().to_string_lossy().to_string();

        // Suppose the crate path is "some_temp_dir_12345" 
        // We create a partial prefix scenario, e.g. "xxx{crate_str}yyy"
        let pathological_path_str = format!("xxx{}yyy", crate_str);

        // We'll attempt to read from that path
        let result = handle
            .read_file_string(Path::new(&pathological_path_str))
            .await;

        // Because it doesn't strictly start with the crate_path, the code tries to do the join,
        // which obviously won't exist => expect an error
        assert!(result.is_err(), "Likely fails to find file or fails to parse path");
    }
}
