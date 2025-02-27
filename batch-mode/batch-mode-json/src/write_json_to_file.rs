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
mod tests {
    use super::*;
    use tokio::fs;
    use std::path::PathBuf;

    #[tokio::test]
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

    #[tokio::test]
    async fn test_write_to_file_invalid_path() {
        // Use an invalid path
        let invalid_path = PathBuf::from("/invalid_path/test_output.json");
        let json_content = r#"{"key": "value"}"#;

        // Attempt to write to the file
        let result = write_to_file(&invalid_path, json_content).await;

        // Verify the result is an error
        assert!(result.is_err());
    }
}
