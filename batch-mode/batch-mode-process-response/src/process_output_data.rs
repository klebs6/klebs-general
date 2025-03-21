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

#[cfg(test)]
mod process_output_data_tests {
    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize, NamedItem)]
    pub struct MockItem {
        pub name: String,
    }

    #[traced_test]
    fn test_process_output_data_with_deserialization_failure() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let workspace = Arc::new(MockWorkspace::default());

            let invalid_msg = BatchMessageBuilder::default()
                .role(MessageRole::Assistant)
                .content(
                    BatchMessageContentBuilder::default()
                        .content("{\"invalid_field\":12}".to_string())
                        .build()
                        .unwrap()
                )
                .build()
                .unwrap();
            let choice_fail = BatchChoiceBuilder::default()
                .index(0_u32)
                .finish_reason(FinishReason::Stop)
                .message(invalid_msg)
                .build()
                .unwrap();
            let success_body_fail = BatchSuccessResponseBodyBuilder::default()
                .id("550e8400-e29b-41d4-a716-446655440000".to_string())
                .choices(vec![choice_fail])
                .build()
                .unwrap();
            let response_content_fail = BatchResponseContentBuilder::default()
                .status_code(200_u16)
                .request_id(ResponseRequestId::new("resp_req_mock_item_2"))
                .body(BatchResponseBody::Success(success_body_fail))
                .build()
                .unwrap();
            let record_fail = BatchResponseRecordBuilder::default()
                .custom_id(CustomRequestId::new("mock_item_2")) // <-- WORKS
                .response(response_content_fail)
                .build()
                .unwrap();

            let output_data = BatchOutputData::new(vec![record_fail]);
            let result = process_output_data::<MockItem>(
                &output_data,
                workspace.as_ref(),
                &ExpectedContentType::Json
            ).await;
            assert!(result.is_ok());
        });
    }
}
