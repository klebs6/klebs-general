// ---------------- [ File: src/process_output_data.rs ]
crate::ix!();

#[instrument(level="trace", skip_all)]
pub async fn process_output_data<T>(
    output_data:           &BatchOutputData,
    workspace:             &dyn BatchWorkspaceInterface,
    expected_content_type: &ExpectedContentType,
) -> Result<(), BatchOutputProcessingError> 
where
    // `'static + Send + Sync` ensures T can be held in `Arc<dyn ... + Send + Sync + 'static>`,
    // and that the future is `Send`.
    T: 'static + Send + Sync + DeserializeOwned + Named + GetTargetPathForAIExpansion,
{
    trace!("entering process_output_data, output_data len = {}", output_data.responses().len());

    let mut failed_entries = Vec::new();

    for response_record in output_data.responses() {
        info!("processing output data record with custom_id={}", response_record.custom_id());

        if let Some(success_body) = response_record.response().body().as_success() {
            if let Err(e) = handle_successful_response::<T>(success_body, workspace, expected_content_type).await {
                eprintln!(
                    "Failed to process response for request ID '{}', error: {:?}, response: {:?}",
                    response_record.custom_id(),
                    e,
                    success_body
                );
                failed_entries.push(response_record);
            }
        }
    }

    if !failed_entries.is_empty() {
        warn!("some entries failed, saving them.");
        save_failed_entries(workspace, &failed_entries).await?;
    }

    info!("process_output_data completed without fatal errors.");
    Ok(())
}

// =======================
// src/process_output_data.rs (RELEVANT TEST ONLY)
// =======================

#[cfg(test)]
mod process_output_data_tests {
    use super::*;
    use std::fs;
    use tokio::runtime::Runtime;

    #[derive(Debug, Clone, Deserialize, Serialize, NamedItem)]
    pub struct MockItem {
        pub name: String,
    }

    #[traced_test]
    fn test_process_output_data_with_deserialization_failure() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // 1) Build a workspace with *all* required fields
            let workspace = Arc::new(
                BatchWorkspaceBuilder::default()
                    .workdir::<PathBuf>("mock_workspace_dir".into())
                    .logdir::<PathBuf>("mock_log_dir".into())
                    .failed_json_repairs_dir::<PathBuf>("mock_failed_json_repairs_dir".into())
                    .failed_items_dir::<PathBuf>("mock_failed_items_dir".into())
                    .done_dir::<PathBuf>("mock_done_dir".into())
                    .target_dir::<PathBuf>("mock_target_dir".into())
                    .temporary(false)
                    .build()
                    .unwrap()
            );

            // 2) Create or clean these directories so no I/O error occurs
            let _ = fs::remove_dir_all(workspace.workdir());
            let _ = fs::remove_dir_all(workspace.logdir());
            let _ = fs::remove_dir_all(workspace.failed_json_repairs_dir());
            let _ = fs::remove_dir_all(workspace.failed_items_dir());
            let _ = fs::remove_dir_all(workspace.done_dir());
            tokio::fs::create_dir_all(workspace.workdir()).await.unwrap();
            tokio::fs::create_dir_all(workspace.logdir()).await.unwrap();
            tokio::fs::create_dir_all(workspace.failed_json_repairs_dir()).await.unwrap();
            tokio::fs::create_dir_all(workspace.failed_items_dir()).await.unwrap();
            tokio::fs::create_dir_all(workspace.done_dir()).await.unwrap();

            // 3) Construct an output_data with a record that fails deserialization into `MockItem`.
            let invalid_msg = BatchMessageBuilder::default()
                .role(MessageRole::Assistant)
                .content(
                    BatchMessageContentBuilder::default()
                        .content("{\"invalid_field\":12}".to_string())
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap();

            let choice_fail = BatchChoiceBuilder::default()
                .index(0_u32)
                .finish_reason(FinishReason::Stop)
                .logprobs(None)
                .message(invalid_msg)
                .build()
                .unwrap();

            let success_body_fail = BatchSuccessResponseBodyBuilder::default()
                .id("550e8400-e29b-41d4-a716-446655440000".to_string())
                .object("response".to_string())
                .created(0_u64)
                .model("test-model".to_string())
                .choices(vec![choice_fail])
                .usage(BatchUsage::mock())
                .build()
                .unwrap();

            let response_content_fail = BatchResponseContentBuilder::default()
                .status_code(200_u16)
                .request_id(ResponseRequestId::new("resp_req_mock_item_2"))
                .body(BatchResponseBody::Success(success_body_fail))
                .build()
                .unwrap();

            let record_fail = BatchResponseRecordBuilder::default()
                .id(BatchRequestId::new("batch_req_mock_item_2"))
                .custom_id(CustomRequestId::new("mock_item_2"))
                .response(response_content_fail)
                .build()
                .unwrap();

            let output_data = BatchOutputData::new(vec![record_fail]);

            // 4) Attempt processing
            let result = process_output_data::<MockItem>(
                &output_data,
                workspace.as_ref(),
                &ExpectedContentType::Json,
            ).await;

            // Because the record is missing the "name" field, it fails to deserialize,
            // but we expect `process_output_data` to handle it by saving to
            // failed_entries.jsonl (not panic).
            assert!(
                result.is_ok(),
                "Should handle the failing record gracefully by saving a failed entry."
            );
        });
    }
}
