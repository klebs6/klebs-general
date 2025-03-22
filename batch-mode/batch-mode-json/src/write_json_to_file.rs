// ---------------- [ File: src/write_json_to_file.rs ]
crate::ix!();

/// Writes serialized JSON content to a file asynchronously.
///
/// # Arguments
/// * `target_path` - A reference to the path where the file will be created/overwritten.
/// * `serialized_json` - The JSON content as a string to write.
///
/// # Returns
/// * `Result<(), io::Error>` - `Ok(())` if successful, or an `io::Error` otherwise.
pub async fn write_to_file(
    target_path: impl AsRef<Path>,
    serialized_json: &str
) -> Result<(), io::Error> 
{
    info!("writing some json content to file: {:?}", target_path.as_ref());

    // Create or overwrite the target file
    let mut target_file = File::create(&target_path).await?;

    // Write the JSON content
    target_file.write_all(serialized_json.as_bytes()).await?;

    // Ensure all data is written and flushed
    target_file.flush().await?;

    Ok(())
}

#[cfg(test)]
mod write_to_file_tests {
    use super::*;

    /// Creates a named temp file, then returns (PathBuf, NamedTempFile).
    /// We keep the NamedTempFile in scope so it persists until the test ends,
    /// preventing collisions or early cleanup.
    fn named_temp_file_with_path(prefix: &str) -> (PathBuf, NamedTempFile) {
        let file = NamedTempFile::new().expect("Failed to create NamedTempFile");
        let path = file.path().to_path_buf();
        // Optionally rename it so we can see the prefix in the path, 
        // but leaving as-is is usually fine. We'll just rely on the unique name.
        // We only do it if you want a more descriptive name in the filesystem:
        /*
        let renamed = file.into_temp_path();
        renamed.persist(format!("{}_{}", prefix, Uuid::new_v4())).unwrap();
        // but that changes usage. We'll skip it for now.
        */
        (path, file)
    }

    #[traced_test]
    async fn test_write_to_file_success() {
        info!("Starting test_write_to_file_success");
        let (temp_path, _tempfile) = named_temp_file_with_path("success");

        let json_content = r#"{"key": "value"}"#;
        let result = write_to_file(&temp_path, json_content).await;
        assert!(result.is_ok());

        // Read back to verify
        let written = fs::read_to_string(&temp_path).await.unwrap();
        pretty_assert_eq!(written, json_content);

        // Cleanup is automatic because NamedTempFile is in scope.
        info!("test_write_to_file_success passed.");
    }

    #[traced_test]
    async fn test_write_to_file_invalid_path() {
        info!("Starting test_write_to_file_invalid_path");
        let invalid_path = PathBuf::from("/invalid_path/test_output.json");
        let json_content = r#"{"key": "value"}"#;

        let result = write_to_file(&invalid_path, json_content).await;
        assert!(result.is_err(), "Should fail writing to an invalid path");
        info!("test_write_to_file_invalid_path passed.");
    }

    #[traced_test]
    async fn returns_error_on_invalid_path() {
        info!("Starting returns_error_on_invalid_path");
        let invalid_path = PathBuf::from("/this/path/does/not/exist.json");
        let json_content = r#"{"key": "value"}"#;

        let result = write_to_file(&invalid_path, json_content).await;
        debug!("Result from write_to_file: {:?}", result);
        assert!(result.is_err(), "Expected an I/O error for invalid path");
        info!("returns_error_on_invalid_path passed.");
    }

    #[traced_test]
    async fn overwrites_existing_file() {
        info!("Starting overwrites_existing_file");
        let (temp_path, _tempfile) = named_temp_file_with_path("overwrite");

        let initial = r#"{"initial": "data"}"#;
        let updated = r#"{"updated": "data"}"#;

        // Write initial
        write_to_file(&temp_path, initial).await.unwrap();
        // Overwrite
        write_to_file(&temp_path, updated).await.unwrap();

        // Verify final
        let final_contents = fs::read_to_string(&temp_path).await.unwrap();
        pretty_assert_eq!(final_contents, updated);

        info!("overwrites_existing_file passed.");
    }

    #[traced_test]
    async fn handles_empty_content() {
        info!("Starting handles_empty_content");
        let (temp_path, _tempfile) = named_temp_file_with_path("empty");
        let empty_content = "";

        write_to_file(&temp_path, empty_content).await.unwrap();

        let read_back = fs::read_to_string(&temp_path).await.unwrap();
        assert!(read_back.is_empty(), "File should be empty after writing empty string");

        info!("handles_empty_content passed.");
    }

    #[traced_test]
    async fn handles_concurrent_writes() {
        info!("Starting handles_concurrent_writes");
        // We'll do 3 concurrency writes to 3 separate files:
        let sets = vec![
            ("concurrent_test_1", r#"{"data": 1}"#),
            ("concurrent_test_2", r#"{"data": 2}"#),
            ("concurrent_test_3", r#"{"data": 3}"#),
        ];

        let mut tasks = Vec::new();
        let mut file_paths = Vec::new();

        for (prefix, content) in sets {
            let (path, tempfile) = named_temp_file_with_path(prefix);
            let content = content.to_string();
            file_paths.push((path.clone(), tempfile)); // keep the NamedTempFile in scope
            tasks.push(tokio::spawn(async move {
                write_to_file(&path, &content).await
            }));
        }

        for task in tasks {
            let res = task.await.expect("Task panicked");
            assert!(res.is_ok(), "Concurrent write task failed");
        }

        // Now verify each
        for (path, _tempfile) in file_paths {
            let data = fs::read_to_string(&path).await.unwrap();
            debug!("Read from {:?}: {}", path, data);
            assert!(data.contains("data"), "Content mismatch in concurrency test");
        }

        info!("handles_concurrent_writes passed.");
    }

    #[traced_test]
    async fn writes_json_content_correctly() {
        trace!("===== BEGIN_TEST: writes_json_content_correctly =====");

        // Use a unique named temp file for this test
        let (temp_path, _tempfile) = named_temp_file_with_path("writes_json_content_correctly");

        let json_content = r#"{"key": "value"}"#;
        let result = write_to_file(&temp_path, json_content).await;
        debug!("write_to_file result: {:?}", result);
        assert!(result.is_ok(), "Expected Ok from write_to_file");

        // Read back content
        let read_content = fs::read_to_string(&temp_path)
            .await
            .expect("Failed to read test file");
        pretty_assert_eq!(read_content, json_content);

        trace!("===== END_TEST: writes_json_content_correctly =====");
    }
}
