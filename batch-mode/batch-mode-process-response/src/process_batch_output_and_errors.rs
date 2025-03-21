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

    #[derive(Debug, Clone, Deserialize, Serialize, NamedItem)]
    pub struct BatchOutputErrorMockItem {
        pub name: String,
    }

    #[traced_test]
    fn test_process_batch_output_and_errors() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let workspace = Arc::new(MockWorkspace::default());
            let _ = fs::remove_dir_all(workspace.workdir());

            let success_msg = BatchMessageBuilder::default()
                .role(MessageRole::Assistant)
                .content(
                    BatchMessageContentBuilder::default()
                        .content("{\"name\":\"batch_out_item\"}".to_string())
                        .build()
                        .unwrap()
                )
                .build()
                .unwrap();
            let success_choice = BatchChoiceBuilder::default()
                .index(0_u32)
                .finish_reason(FinishReason::Stop)
                .message(success_msg)
                .build()
                .unwrap();
            let success_body = BatchSuccessResponseBodyBuilder::default()
                .id("just-an-id".to_string())
                .object("chat.completion".to_string())
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
                .custom_id(CustomRequestId::new("ok_1")) // <-- WORKS
                .response(success_content)
                .build()
                .unwrap();

            let error_details = BatchErrorDetailsBuilder::default()
                .message("some test error".to_string())
                .error_type(ErrorType::Unknown("999".to_string()))
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
                .custom_id(CustomRequestId::new("err_1")) // <-- WORKS
                .response(error_content)
                .build()
                .unwrap();

            let out_data = BatchOutputData::new(vec![success_record]);
            let err_data = BatchErrorData::new(vec![error_record]);
            let batch_result = BatchExecutionResultBuilder::default() // <-- WORKS
                .outputs(Some(out_data))
                .errors(Some(err_data))
                .build()
                .unwrap();

            let result = process_batch_output_and_errors::<BatchOutputErrorMockItem>(
                workspace.as_ref(),
                &batch_result,
                &ExpectedContentType::Json,
            ).await;
            assert!(result.is_ok());
            let out_path = workspace.workdir().join("batch_out_item.json");
            assert!(out_path.exists());
        });
    }
}
