// ---------------- [ File: src/write_json_to_file.rs ]
crate::ix!();

/// Writes serialized JSON content to a file asynchronously.
///
/// # Arguments
/// * `target_path` - A reference to the path where the file will be written.
/// * `serialized_json` - The JSON content as a string to write to the file.
///
/// # Returns
/// * `Result<(), io::Error>` - `Ok(())` if the write succeeds, or an `io::Error` otherwise.
///
/// # Errors
/// * Returns an `io::Error` if file creation, writing, or flushing fails.
pub async fn write_to_file(target_path: impl AsRef<Path>, serialized_json: &str) 
    -> Result<(), io::Error> 
{
    info!("writing some json content to the file {:?}", target_path.as_ref());

    // Create or overwrite the target file
    let mut target_file = File::create(target_path).await?;

    // Write the serialized JSON content to the file
    target_file.write_all(serialized_json.as_bytes()).await?;

    // Ensure all data is written and flushed to the disk
    target_file.flush().await?;

    Ok(())
}

#[cfg(test)]
mod write_to_file_tests {
    use super::*;
    use tokio::fs;
    use std::path::PathBuf;

    #[traced_test]
    async fn test_write_to_file_success() {
        // Create a temporary file path
        let temp_path = PathBuf::from("test_output.json");

        // JSON content to write
        let json_content = r#"{"key": "value"}"#;

        // Write to file
        let result = write_to_file(&temp_path, json_content).await;
        assert!(result.is_ok());

        // Read back the file content to verify
        let written_content = fs::read_to_string(&temp_path).await.unwrap();
        assert_eq!(written_content, json_content);

        // Cleanup
        fs::remove_file(temp_path).await.unwrap();
    }

    #[traced_test]
    async fn test_write_to_file_invalid_path() {
        // Use an invalid path
        let invalid_path = PathBuf::from("/invalid_path/test_output.json");
        let json_content = r#"{"key": "value"}"#;

        // Attempt to write to the file
        let result = write_to_file(&invalid_path, json_content).await;

        // Verify the result is an error
        assert!(result.is_err());
    }

    #[traced_test]
    async fn returns_error_on_invalid_path() {
        info!("Starting test: returns_error_on_invalid_path");
        let invalid_path = PathBuf::from("/invalid_path/test_output.json");
        let json_content = r#"{"key": "value"}"#;

        trace!(
            "Invoking write_to_file with an invalid path {:?} and content: {}",
            invalid_path,
            json_content
        );
        let result = write_to_file(&invalid_path, json_content).await;
        debug!("write_to_file result: {:?}", result);

        assert!(
            result.is_err(),
            "Expected an error from write_to_file for an invalid path"
        );
        info!("Completed test: returns_error_on_invalid_path");
    }

    #[traced_test]
    async fn overwrites_existing_file() {
        info!("Starting test: overwrites_existing_file");
        let temp_path = PathBuf::from("test_output_overwrite.json");
        let initial_content = r#"{"initial": "data"}"#;
        let new_content = r#"{"updated": "data"}"#;

        trace!("Creating file with initial content");
        let create_result = write_to_file(&temp_path, initial_content).await;
        assert!(
            create_result.is_ok(),
            "Failed to create or write initial content to file"
        );

        trace!("Overwriting file with new content");
        let overwrite_result = write_to_file(&temp_path, new_content).await;
        assert!(
            overwrite_result.is_ok(),
            "Failed to overwrite file with new content"
        );

        trace!("Verifying overwritten content");
        let final_read = fs::read_to_string(&temp_path).await.unwrap();
        assert_eq!(
            final_read, new_content,
            "File content was not correctly overwritten"
        );

        trace!("Cleaning up temporary file {:?}", temp_path);
        let cleanup_result = fs::remove_file(&temp_path).await;
        assert!(
            cleanup_result.is_ok(),
            "Failed to remove temporary file after overwrite test"
        );
        info!("Completed test: overwrites_existing_file");
    }

    #[traced_test]
    async fn handles_empty_content() {
        info!("Starting test: handles_empty_content");
        let temp_path = PathBuf::from("test_output_empty.json");
        let empty_content = "";

        trace!("Attempting to write empty content to file {:?}", temp_path);
        let result = write_to_file(&temp_path, empty_content).await;
        assert!(result.is_ok(), "Expected Ok when writing empty content to file");

        trace!("Reading back content for verification");
        let written_content = fs::read_to_string(&temp_path).await.unwrap();
        debug!("Read content: {}", written_content);

        assert!(
            written_content.is_empty(),
            "File should be empty after writing empty content"
        );

        trace!("Cleaning up temporary file {:?}", temp_path);
        let cleanup_result = fs::remove_file(&temp_path).await;
        assert!(
            cleanup_result.is_ok(),
            "Failed to remove temporary file after writing empty content"
        );
        info!("Completed test: handles_empty_content");
    }

    #[traced_test]
    async fn handles_concurrent_writes() {
        info!("Starting test: handles_concurrent_writes");
        let paths_and_contents = vec![
            (PathBuf::from("concurrent_test_1.json"), r#"{"data": 1}"#),
            (PathBuf::from("concurrent_test_2.json"), r#"{"data": 2}"#),
            (PathBuf::from("concurrent_test_3.json"), r#"{"data": 3}"#),
        ];

        trace!("Spawning concurrent write tasks");
        let mut tasks = vec![];
        for (path, content) in paths_and_contents.iter() {
            let path_clone = path.clone();
            let content_clone = content.to_string();
            tasks.push(tokio::spawn(async move {
                write_to_file(path_clone, &content_clone).await
            }));
        }

        trace!("Awaiting all write tasks");
        for task in tasks {
            let result = task.await.expect("Task panicked unexpectedly");
            debug!("Concurrent write result: {:?}", result);
            assert!(result.is_ok(), "Concurrent write operation failed");
        }

        trace!("Verifying each file's content");
        for (path, content) in paths_and_contents {
            let read_back = fs::read_to_string(&path).await.unwrap();
            debug!("Read from {:?}: {}", path, read_back);
            assert_eq!(
                read_back, content,
                "Mismatch between written and read content in concurrency test"
            );
            let cleanup_result = fs::remove_file(&path).await;
            assert!(
                cleanup_result.is_ok(),
                "Failed to clean up temporary file in concurrency test"
            );
        }

        // Demonstrating a small delay to ensure all flushes complete before test ends
        sleep(Duration::from_millis(100)).await;
        info!("Completed test: handles_concurrent_writes");
    }

    #[traced_test]
    async fn writes_json_content_correctly() {
        trace!("===== BEGIN_TEST: writes_json_content_correctly =====");

        let file_path = "test_output.json";
        let json_content = r#"{"key": "value"}"#;

        trace!("Invoking write_to_file with path \"{}\" and content: {}", file_path, json_content);

        trace!("writing some json content to the file \"{}\"", file_path);
        let result = write_to_file(file_path, json_content).await;

        debug!("write_to_file result: {:?}", result);
        assert!(result.is_ok());

        // Verify file content
        trace!("Reading back content for verification");
        let read_content = fs::read_to_string(file_path).await.expect("Failed to read test file");
        debug!("Read content: {}", read_content);
        assert_eq!(read_content, json_content);

        // Clean up
        trace!("Cleaning up temporary file \"{}\"", file_path);
        if let Err(err) = fs::remove_file(file_path).await {
            warn!("Failed to remove temporary file during cleanup: {:?}", err);
        }

        trace!("===== END_TEST: writes_json_content_correctly =====");
    }
}
