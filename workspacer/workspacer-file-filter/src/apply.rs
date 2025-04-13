crate::ix!();

/// In real usage, you could have a convenience function that:
///  1) Reads a list of file paths from a text file (one per line)
///  2) Builds requests
///  3) Calls `execute_ai_file_filter_requests(...)`
/// This is just an example to illustrate the concept.
#[tracing::instrument(level="trace")]
pub async fn apply_text_filter_to_files(
    list_path: impl AsRef<Path>,
    user_instructions: &str,
    plant: bool,
    config: &FileFilterConfig
) -> Result<(), AiFileFilterError> {
    // 1) Read the lines
    let lines = tokio::fs::read_to_string(&list_path).await.map_err(|io_err| {
        AiFileFilterError::IoError {
            io_error: std::sync::Arc::new(io_err),
            context: format!("Failed to read file list at {}", list_path.as_ref().display()),
        }
    })?;

    // 2) For each line, trim and skip empty lines
    let mut requests = Vec::new();
    for (idx, raw_line) in lines.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            warn!("Skipping empty line #{} in file list", idx);
            continue;
        }
        // Build the request
        let path_obj = PathBuf::from(trimmed);
        let req = AiFileFilterRequest::async_try_from_path(path_obj, user_instructions, config).await?;
        requests.push(req);
    }

    // 3) If no requests, we do nothing
    if requests.is_empty() {
        info!("No valid files were provided, so nothing to do.");
        return Ok(());
    }

    // 4) Acquire the AiFileFilter object
    let mut writer = AiFileFilter::default().await?;

    // 5) Execute the requests
    execute_ai_file_filter_requests(&mut writer, &requests, plant).await?;

    info!("apply_text_filter_to_files completed successfully.");
    Ok(())
}
