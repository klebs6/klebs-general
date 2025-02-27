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
