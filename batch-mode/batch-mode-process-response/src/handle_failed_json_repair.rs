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
    async fn test_handle_failed_json_repair_writes_file() {
        // We rely on MockWorkspace's ephemeral sandbox to avoid leaving test files around.
        // We no longer override the `failed_json_repairs_dir`; we want the default ephemeral dir.
        trace!("===== BEGIN TEST: test_handle_failed_json_repair_writes_file =====");
        // 1) Create the ephemeral workspace
        let workspace = BatchWorkspace::new_temp().await.unwrap();
        info!("Created ephemeral workspace: {:?}", workspace);

        // 2) Clear out the ephemeral `failed_json_repairs_dir()` if it exists
        let repairs_dir = workspace.failed_json_repairs_dir();

        // 3) Prepare data
        let failed_id = "failed_request_123";
        let message_content = BatchMessageContentBuilder::default()
            .content::<String>("some broken json stuff".into())
            .build()
            .unwrap();

        // 4) Invoke the function
        let result = handle_failed_json_repair(failed_id, &message_content, workspace.as_ref()).await;
        assert!(result.is_ok(), "handle_failed_json_repair should succeed");

        // 5) Verify the file was actually written into the ephemeral repairs dir
        let path = repairs_dir.join(failed_id);
        trace!("Asserting that repair file path exists: {:?}", path);
        assert!(path.exists(), "Repaired JSON file must exist in ephemeral dir");

        // 6) Verify contents match
        let written = fs::read_to_string(&path).unwrap();
        assert_eq!(written, message_content.get_sanitized_json_str());

        trace!("===== END TEST: test_handle_failed_json_repair_writes_file =====");
    }
}
