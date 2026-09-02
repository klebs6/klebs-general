// ---------------- [ File: batch-mode-process-response/src/process_output_data.rs ]
crate::ix!();

#[instrument(level = "trace", skip_all)]
pub async fn process_output_data<T>(
    output_data: &BatchOutputData,
    workspace: &dyn BatchWorkspaceInterface,
    expected_content_type: &ExpectedContentType,
) -> Result<(), BatchOutputProcessingError>
where
    T: 'static + Send + Sync + DeserializeOwned + Named + GetTargetPathForAIExpansion,
{
    trace!(
        "Starting process_output_data with {} response record(s)",
        output_data.responses().len()
    );

    let mut failed_entries = Vec::new();

    for response_record in output_data.responses() {
        let custom_id = response_record.custom_id();
        trace!("Processing response record custom_id={}", custom_id);

        match response_record.response().body().as_success() {
            Some(success_body) => {
                trace!("Successfully extracted body for custom_id={}", custom_id);

                match workspace.load_seed_by_custom_id(custom_id).await {
                    Ok(seed_box) => {
                        trace!(
                            "Successfully loaded seed named '{}' for custom_id={}",
                            seed_box.name(),
                            custom_id
                        );

                        match handle_successful_response::<T>(
                            success_body,
                            workspace,
                            expected_content_type,
                            seed_box.as_ref(),
                        )
                        .await
                        {
                            Ok(()) => {
                                debug!("Successfully handled response for custom_id={}", custom_id);
                            }
                            Err(e) => {
                                error!(
                                    "Error processing successful response for custom_id={}: {:?}",
                                    custom_id, e
                                );
                                debug!("Problematic success_body for custom_id={}: {:?}", custom_id, success_body);
                                failed_entries.push(response_record);
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to load seed for custom_id={}, error: {:?}",
                            custom_id, e
                        );
                        failed_entries.push(response_record);
                    }
                }
            }
            None => {
                warn!("No success body for custom_id={}", custom_id);
                failed_entries.push(response_record);
            }
        }
    }

    if !failed_entries.is_empty() {
        warn!(
            "{} response record(s) failed, invoking save_failed_entries",
            failed_entries.len()
        );

        if let Err(e) = save_failed_entries(workspace, &failed_entries).await {
            error!(
                "Failed to save {} failed entries: {:?}",
                failed_entries.len(),
                e
            );
            return Err(e.into());
        } else {
            info!("Successfully saved {} failed entries.", failed_entries.len());
        }
    }

    info!(
        "Finished process_output_data ({} succeeded, {} failed)",
        output_data.responses().len() - failed_entries.len(),
        failed_entries.len()
    );

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
    async fn test_process_output_data_with_deserialization_failure() {

        let workspace: Arc<dyn BatchWorkspaceInterface> = BatchWorkspace::new_temp().await.unwrap();

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
    }
}
