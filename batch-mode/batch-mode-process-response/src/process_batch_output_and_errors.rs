// ---------------- [ File: src/process_batch_output_and_errors.rs ]
crate::ix!();

/// We add `'static + Send + Sync` to `T`, so that storing it in an
/// `Arc<dyn ... + Send + Sync + 'static>` is valid and the resulting
/// `Future` can be `Send`.
pub async fn process_batch_output_and_errors<T>(
    workspace:              &dyn BatchWorkspaceInterface, 
    batch_execution_result: &BatchExecutionResult,
    expected_content_type:  &ExpectedContentType,
) -> Result<(), BatchProcessingError> 
where
    // The key is here: `'static + Send + Sync`.
    // This ensures that any Arc<T> or Arc<dyn ...> is also Send + Sync,
    // letting the async function be `Pin<Box<dyn Future<...> + Send>>`.
    T: 'static + Send + Sync + DeserializeOwned + Named + GetTargetPathForAIExpansion,
{
    trace!("process_batch_output_and_errors => start.");

    // Process the outputs
    if let Some(output_data) = &batch_execution_result.outputs() {
        info!("processing batch output data of len {}", output_data.len());
        // Also requires `'static + Send + Sync` in process_output_data signature
        process_output_data::<T>(output_data, workspace, expected_content_type).await?;
    }

    // Process the errors
    if let Some(error_data) = &batch_execution_result.errors() {
        info!("processing batch error data of len {}", error_data.len());
        process_error_data(error_data).await?;
    }

    Ok(())
}

#[cfg(test)]
mod process_batch_output_and_errors_tests {
    use super::*;
    use std::fs;
    use tokio::runtime::Runtime;

    #[derive(Debug, Clone, Deserialize, Serialize, NamedItem)]
    pub struct BatchOutputErrorMockItem {
        pub name: String,
    }

    #[traced_test]
    fn test_process_batch_output_and_errors() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // 1) Build a workspace with *all* required fields, including 'target_dir' and 'temporary'
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

            // 2) Clear or create directories
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

            // 3) Build a successful record
            let success_msg = BatchMessageBuilder::default()
                .role(MessageRole::Assistant)
                .content(
                    BatchMessageContentBuilder::default()
                        .content("{\"name\":\"batch_out_item\"}".to_string())
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap();

            let success_choice = BatchChoiceBuilder::default()
                .index(0_u32)
                .finish_reason(FinishReason::Stop)
                .logprobs(None)
                .message(success_msg)
                .build()
                .unwrap();

            let success_body = BatchSuccessResponseBodyBuilder::default()
                .id("just-an-id".to_string())
                .object("chat.completion".to_string())
                .created(0_u64)
                .model("test-model".to_string())
                .choices(vec![success_choice])
                .usage(BatchUsage::mock())
                .build()
                .unwrap();

            let success_content = BatchResponseContentBuilder::default()
                .status_code(200_u16)
                .request_id(ResponseRequestId::new("resp_req_ok_1"))
                .body(BatchResponseBody::Success(success_body))
                .build()
                .unwrap();

            let success_record = BatchResponseRecordBuilder::default()
                .id(BatchRequestId::new("batch_req_ok_1"))
                .custom_id(CustomRequestId::new("ok_1"))
                .response(success_content)
                .build()
                .unwrap();

            // 4) Build an error record
            let error_details = BatchErrorDetailsBuilder::default()
                .error_type(ErrorType::Unknown("999".to_string()))
                .message("some test error".to_string())
                .build()
                .unwrap();

            let error_body = BatchErrorResponseBodyBuilder::default()
                .error(error_details)
                .build()
                .unwrap();

            let error_content = BatchResponseContentBuilder::default()
                .status_code(400_u16)
                .request_id(ResponseRequestId::new("resp_req_err_1"))
                .body(BatchResponseBody::Error(error_body))
                .build()
                .unwrap();

            let error_record = BatchResponseRecordBuilder::default()
                .id(BatchRequestId::new("batch_req_err_1"))
                .custom_id(CustomRequestId::new("err_1"))
                .response(error_content)
                .build()
                .unwrap();

            // 5) Construct a batch result
            let out_data = BatchOutputData::new(vec![success_record]);
            let err_data = BatchErrorData::new(vec![error_record]);
            let batch_result = BatchExecutionResultBuilder::default()
                .outputs(Some(out_data))
                .errors(Some(err_data))
                .build()
                .unwrap();

            // 6) Attempt to process
            let result = process_batch_output_and_errors::<BatchOutputErrorMockItem>(
                workspace.as_ref(),
                &batch_result,
                &ExpectedContentType::Json,
            ).await;

            // 7) We expect it to succeed
            assert!(
                result.is_ok(),
                "Should handle success & error records gracefully, returning Ok."
            );
        });
    }
}
