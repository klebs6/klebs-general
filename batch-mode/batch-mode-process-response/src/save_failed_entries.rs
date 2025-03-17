// ---------------- [ File: src/save_failed_entries.rs ]
crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub async fn save_failed_entries(
    workspace:      &dyn BatchWorkspaceInterface,
    failed_entries: &[&BatchResponseRecord],
) -> Result<(), ErrorSavingFailedBatchEntries> {
    trace!("Entering save_failed_entries.");

    info!("saving failed entries: {:#?}", failed_entries);

    // 1) Serialize failed entries to JSON Lines format
    let mut serialized_entries = String::new();
    for entry in failed_entries {
        let json_line = serde_json::to_string(entry)
            .map_err(ErrorSavingFailedBatchEntries::from)?;
        serialized_entries.push_str(&json_line);
        serialized_entries.push('\n');
    }

    // 2) Append them to a file named, for example, "failed_entries.jsonl" inside
    //    the `failed_items_dir()`. We do NOT open the directory itself.
    let file_path = workspace
        .failed_items_dir()
        .join("failed_entries.jsonl");

    debug!("Appending failed entries to file at path: {:?}", file_path);

    // 3) Write to the file
    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .await
        .map_err(ErrorSavingFailedBatchEntries::from)?;

    file.write_all(serialized_entries.as_bytes())
        .await
        .map_err(ErrorSavingFailedBatchEntries::from)?;

    info!("save_failed_entries completed successfully.");
    Ok(())
}
