// ---------------- [ File: src/save_failed_entries.rs ]
crate::ix!();

pub async fn save_failed_entries(
    workspace:      &dyn BatchWorkspaceInterface,
    failed_entries: &[&BatchResponseRecord],
) -> Result<(), ErrorSavingFailedBatchEntries> {

    info!("saving failed entries: {:#?}", failed_entries);

    // Serialize the failed entries to JSON Lines format
    let mut serialized_entries = String::new();
    for entry in failed_entries {
        let json_line = serde_json::to_string(entry)?;
        serialized_entries.push_str(&json_line);
        serialized_entries.push('\n');
    }

    // Append to the failed entries file
    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(workspace.failed_items_dir())
        .await?;
    file.write_all(serialized_entries.as_bytes()).await?;

    Ok(())
}
