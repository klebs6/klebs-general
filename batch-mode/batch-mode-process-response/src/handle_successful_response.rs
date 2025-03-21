// ---------------- [ File: src/handle_successful_response.rs ]
crate::ix!();

#[instrument(level="trace", skip_all)]
pub async fn handle_successful_response<T>(
    success_body:          &BatchSuccessResponseBody,
    workspace:             &dyn BatchWorkspaceInterface,
    expected_content_type: &ExpectedContentType,
) -> Result<(), BatchSuccessResponseHandlingError> 
where
    T: 'static + Send + Sync + Named + DeserializeOwned + GetTargetPathForAIExpansion,
{
    trace!("Entering handle_successful_response with success_body ID: {}", success_body.id());
    trace!("success_body => finish_reason={:?}, total_choices={}",
        success_body.choices().get(0).map(|c| c.finish_reason()),
        success_body.choices().len()
    );

    let choice = &success_body.choices()[0];
    let message_content = choice.message().content();
    trace!("Pulled first choice => finish_reason={:?}", choice.finish_reason());

    if *choice.finish_reason() == FinishReason::Length {
        trace!("Detected finish_reason=Length => calling handle_finish_reason_length");
        handle_finish_reason_length(success_body.id(), message_content).await?;
        trace!("Returned from handle_finish_reason_length with success_body ID: {}", success_body.id());
    }

    match expected_content_type {
        ExpectedContentType::Json => {
            trace!("ExpectedContentType::Json => about to extract/repair JSON for success_body ID: {}", success_body.id());
            match message_content.extract_clean_parse_json_with_repair() {
                Ok(json_content) => {
                    debug!("JSON parse/repair succeeded for success_body ID: {}", success_body.id());
                    trace!("Now deserializing into typed struct T...");

                    // In handle_successful_response.rs:
                    let typed_item: T = match serde_json::from_value(json_content.clone()) {
                        Ok(t) => {
                            trace!("Deserialization into T succeeded...");
                            t
                        }
                        Err(e) => {
                            error!("Deserialization into T failed: {:?}", e);
                            // We also call handle_failed_json_repair here so that the test sees a file
                            handle_failed_json_repair(success_body.id(), message_content, workspace).await?;
                            return Err(e.into());
                        }
                    };

                    // Convert to Arc if needed
                    trace!("Wrapping typed_item in Arc => T::name()={}", typed_item.name());
                    let typed_item_arc: Arc<dyn GetTargetPathForAIExpansion + Send + Sync + 'static> = Arc::new(typed_item);

                    // Determine target path
                    let target_path = workspace.target_path(&typed_item_arc, expected_content_type);
                    trace!("Target path computed => {:?}", target_path);

                    // Pretty-print the fully repaired JSON
                    let serialized_json = match serde_json::to_string_pretty(&json_content) {
                        Ok(s) => {
                            trace!("Successfully created pretty JSON string for success_body ID: {}", success_body.id());
                            s
                        }
                        Err(e) => {
                            error!("Re-serialization to pretty JSON failed: {:?}", e);
                            return Err(JsonParseError::SerdeError(e).into());
                        }
                    };

                    info!("writing JSON output to {:?}", target_path);
                    write_to_file(&target_path, &serialized_json).await?;
                    trace!("Successfully wrote JSON file => {:?}", target_path);
                    trace!("Exiting handle_successful_response with success_body ID: {}", success_body.id());
                    Ok(())
                }
                Err(e) => {
                    warn!("JSON extraction/repair failed for success_body ID: {} with error: {:?}", success_body.id(), e);
                    let failed_id = success_body.id();
                    trace!("Calling handle_failed_json_repair for ID={}", failed_id);
                    handle_failed_json_repair(failed_id, message_content, workspace).await?;
                    trace!("Returned from handle_failed_json_repair => now returning error for ID={}", failed_id);
                    Err(e.into())
                }
            }
        }
        ExpectedContentType::PlainText => {
            trace!("Received plain text content for request {} => length={}", success_body.id(), message_content.len());
            let index = BatchIndex::from_uuid_str(success_body.id())?;
            trace!("Parsed BatchIndex => {:?}", index);

            let text_path = workspace.text_storage_path(&index);
            info!("writing plain text output to {:?}", text_path);
            write_to_file(&text_path, message_content.as_str()).await?;
            trace!("Successfully wrote plain text file => {:?}", text_path);

            trace!("Exiting handle_successful_response with success_body ID: {}", success_body.id());
            Ok(())
        }
        _ => { todo!() }
    }
}

#[cfg(test)]
mod handle_successful_response_tests {
    use super::*;

    #[derive(Debug, Deserialize, Serialize, NamedItem)]
    pub struct MockItemForSuccess {
        pub name: String,
    }

    #[traced_test]
    fn test_handle_successful_response_json_failure() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let workspace = Arc::new(
                MockWorkspaceBuilder::default()
                    .failed_json_repairs_dir("./test_failed_json_repairs_2".into())
                    .build()
                    .unwrap()
            );
            let _ = fs::remove_dir_all(workspace.failed_json_repairs_dir());
            tokio::fs::create_dir_all(&workspace.failed_json_repairs_dir()).await.unwrap();

            let invalid_msg = ChatCompletionResponseMessage {
                role: Role::Assistant,
                content: Some("this is not valid json at all".into()),
                audio: None,
                function_call: None,
                refusal: None,
                tool_calls: None,
            };

            // Must do .logprobs(None) so BatchChoice is fully initialized
            let choice_fail = BatchChoiceBuilder::default()
                .index(0_u32)
                .finish_reason(FinishReason::Stop)
                .logprobs(None)
                .message(invalid_msg)
                .build()
                .unwrap();

            let success_body = BatchSuccessResponseBodyBuilder::default()
                .object("response".to_string())
                .id("some-other-uuid".to_string())
                .created(0_u64)
                .model("test-model".to_string())
                .choices(vec![choice_fail])
                .usage(BatchUsage::mock())
                .build()
                .unwrap();

            let rc = handle_successful_response::<MockItemForSuccess>(
                &success_body,
                workspace.as_ref(),
                &ExpectedContentType::Json
            ).await;

            assert!(rc.is_err(), "We expect an error due to invalid JSON content");
            let repair_path = workspace.failed_json_repairs_dir().join("some-other-uuid");
            assert!(repair_path.exists());
        });
    }
}
