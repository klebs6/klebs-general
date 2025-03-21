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

    // 2) Append them to a file named, e.g. "failed_entries.jsonl"
    let file_path = workspace.failed_items_dir().join("failed_entries.jsonl");

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

#[cfg(test)]
mod save_failed_entries_tests {
    use super::*;

    #[traced_test]
    fn test_save_failed_entries() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let workspace = Arc::new(
                MockWorkspaceBuilder::default()
                    .failed_items_dir("./test_failed_items_dir".into())
                    .build()
                    .unwrap()
            );
            let _ = std::fs::remove_dir_all(workspace.failed_items_dir());
            tokio::fs::create_dir_all(&workspace.failed_items_dir()).await.unwrap();

            // Instead of `.code("xxx".into())`, use `.code(Some("xxx".to_string()))`
            // Instead of `.message("some error".into())`, use `.message("some error".to_string())
            // so the builder sees the correct types:

            let fail_details = BatchErrorDetailsBuilder::default()
                .code(Some("xxx".to_string()))
                .message("some error".to_string())
                .build()
                .unwrap();

            let fail_errbody = BatchErrorResponseBodyBuilder::default()
                .error(fail_details)
                .build()
                .unwrap();

            let fail_respcontent = BatchResponseContentBuilder::default()
                .status_code(400_u16)
                .request_id(ResponseRequestId::new("resp_fail_1"))
                .body(BatchResponseBody::Error(fail_errbody))
                .build()
                .unwrap();

            let fail_rec = BatchResponseRecordBuilder::default()
                .custom_id(CustomRequestId::new("fail_1"))
                .response(fail_respcontent)
                .build()
                .unwrap();

            let failed_records = vec![ &fail_rec ];
            let result = save_failed_entries(workspace.as_ref(), &failed_records).await;
            assert!(result.is_ok());

            let file_path = workspace.failed_items_dir().join("failed_entries.jsonl");
            assert!(file_path.exists());
            let contents = std::fs::read_to_string(file_path).unwrap();
            assert!(contents.contains("\"fail_1\""));
        });
    }
}
