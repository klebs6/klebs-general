// ---------------- [ File: src/handle_failed_json_repair.rs ]
crate::ix!();

pub async fn handle_failed_json_repair(
    failed_id:       &str,
    message_content: &BatchMessageContent,
    workspace:       &dyn BatchWorkspaceInterface,

) -> Result<(), BatchSuccessResponseHandlingError> {

    warn!("handing failed json repair");

    let mut failed_repair_log = workspace.failed_json_repairs_dir().to_path_buf();

    failed_repair_log.push(failed_id);

    let failed_str = message_content.get_sanitized_json_str();

    write_to_file(&failed_repair_log,failed_str).await?;

    Ok(())
}

#[cfg(test)]
mod handle_failed_json_repair_tests {
    use super::*;
    use futures::executor::block_on;
    use std::fs;

    #[traced_test]
    fn test_handle_failed_json_repair_writes_file() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let workspace = Arc::new(
                MockWorkspaceBuilder::default()
                    // Suppose your real method is `json_repairs_dir(...)`
                    .json_repairs_dir("./test_failed_json_repairs".into())
                    .build()
                    .unwrap()
            );
            let _ = fs::remove_dir_all(workspace.json_repairs_dir());
            tokio::fs::create_dir_all(&workspace.json_repairs_dir()).await.unwrap();

            let failed_id = "failed_request_123";
            // Instead of `BatchMessageContent::from("...")`, we do builder:
            let message_content = BatchMessageContentBuilder::default()
                .content::<String>("some broken json stuff".into())
                .build()
                .unwrap();

            let result = handle_failed_json_repair(
                failed_id,
                &message_content,
                workspace.as_ref()
            ).await;
            assert!(result.is_ok());

            let path = workspace.json_repairs_dir().join(failed_id);
            assert!(path.exists());
            let written = fs::read_to_string(&path).unwrap();
            assert_eq!(written, message_content.get_sanitized_json_str());
        });
    }
}
