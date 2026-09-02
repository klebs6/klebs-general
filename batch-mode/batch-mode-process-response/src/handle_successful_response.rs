// ---------------- [ File: batch-mode-process-response/src/handle_successful_response.rs ]
crate::ix!();

/// Produce a *flattened* JSON payload suitable for saving to `target/`
///
/// * If the original assistant JSON already matches the target structure,
///   we return it unchanged.
/// * If it was wrapped in a `{ "fields": { … } }` envelope, we drop the
///   envelope and return only the inner object so that subsequent
///   `T::load_from_file()` calls (which expect a *flat* object) succeed.
///
/// All paths are traced for debuggability.
#[instrument(level = "trace", skip_all)]
fn flatten_json_for_persistence(original: &serde_json::Value) -> serde_json::Value {
    if let Some(inner) = original.get("fields") {
        trace!("Detected `fields` wrapper — flattening for persistence.");
        inner.clone()
    } else {
        trace!("No `fields` wrapper — JSON already flat.");
        original.clone()
    }
}

/// ---------------------------------------------------------------------------
/// Guarantee *header echo*:  
/// If the JSON produced by the LLM contains a `"header"` key, we make sure its
/// value matches `T::name()`.  If not, we patch it **before the file is written**.
/// This keeps the rule generic – no per‑type hard‑coding is required.
/// ---------------------------------------------------------------------------
#[inline(always)]
fn enforce_header_echo(seed: &(dyn Named), json: &mut serde_json::Value) {
    let expected = seed.name();
    match json.get_mut("header") {
        Some(h) if h == expected.as_ref() => {} // already fine
        Some(h) => *h = serde_json::Value::String(expected.into_owned()),
        None => {
            json.as_object_mut()
                .expect("output must be object")
                .insert("header".into(), serde_json::Value::String(expected.into_owned()));
        }
    }
}

#[instrument(level = "trace", skip_all)]
pub async fn handle_successful_response<T>(
    success_body:          &BatchSuccessResponseBody,
    workspace:             &dyn BatchWorkspaceInterface,
    expected_content_type: &ExpectedContentType,
    original_seed:         &(dyn Named + Send + Sync),
) -> Result<(), BatchSuccessResponseHandlingError>
where
    T: 'static + Send + Sync + Named + DeserializeOwned + GetTargetPathForAIExpansion,
{
    trace!(
        "Entering handle_successful_response with success_body ID: {}",
        success_body.id()
    );
    trace!(
        "success_body => finish_reason={:?}, total_choices={}",
        success_body.choices().get(0).map(|c| c.finish_reason()),
        success_body.choices().len()
    );

    let choice = &success_body.choices()[0];
    let message_content = choice.message().content();
    trace!(
        "Pulled first choice => finish_reason={:?}",
        choice.finish_reason()
    );

    if *choice.finish_reason() == FinishReason::Length {
        trace!("Detected finish_reason=Length => calling handle_finish_reason_length");
        handle_finish_reason_length(success_body.id(), message_content).await?;
        trace!(
            "Returned from handle_finish_reason_length with success_body ID: {}",
            success_body.id()
        );
    }

    match expected_content_type {
        ExpectedContentType::Json => {
            trace!(
                "ExpectedContentType::Json => about to extract/repair JSON for success_body ID: {}",
                success_body.id()
            );

            match message_content.extract_clean_parse_json_with_repair() {
                Ok(mut json_content) => {
                    debug!(
                        "JSON parse/repair succeeded for success_body ID: {}",
                        success_body.id()
                    );

                    // ---------- DESERIALISE (with optional wrapper) ----------
                    let typed_item: T = match deserialize_json_with_optional_fields_wrapper::<T>(&json_content) {
                        Ok(item) => item,
                        Err(e) => {
                            error!("Deserialization into T failed: {:?}", e);
                            handle_failed_json_repair(success_body.id(), message_content, workspace).await?;
                            return Err(e.into());
                        }
                    };

                    /* Enforce the header‑echo rule *generically*. */
                    enforce_header_echo(original_seed, &mut json_content);

                    //------------------------------------------------------------------

                    trace!("Wrapping typed_item in Arc => T::name()={}", typed_item.name());
                    let typed_item_arc: Arc<
                        dyn GetTargetPathForAIExpansion + Send + Sync + 'static,
                    > = Arc::new(typed_item);

                    // Choose where we will write the output **before** we flatten it.
                    let target_path = workspace.target_path(&typed_item_arc, expected_content_type);
                    trace!("Target path computed => {:?}", target_path);

                    // ---------- NEW: ALWAYS WRITE A *FLAT* OBJECT ----------
                    let flattened_json = flatten_json_for_persistence(&json_content);
                    let serialized_json = match serde_json::to_string_pretty(&flattened_json) {
                        Ok(s) => {
                            trace!(
                                "Successfully created pretty JSON string for success_body ID: {}",
                                success_body.id()
                            );
                            s
                        }
                        Err(e) => {
                            error!("Re‑serialization to pretty JSON failed: {:?}", e);
                            return Err(JsonParseError::SerdeError(e).into());
                        }
                    };
                    //--------------------------------------------------------

                    info!("writing JSON output to {:?}", target_path);
                    write_to_file(&target_path, &serialized_json).await?;
                    trace!("Successfully wrote JSON file => {:?}", target_path);
                    trace!(
                        "Exiting handle_successful_response with success_body ID: {}",
                        success_body.id()
                    );
                    Ok(())
                }
                Err(e_extract) => {
                    warn!(
                        "JSON extraction/repair failed for success_body ID: {} with error: {:?}",
                        success_body.id(),
                        e_extract
                    );
                    let failed_id = success_body.id();
                    trace!("Calling handle_failed_json_repair for ID={}", failed_id);
                    handle_failed_json_repair(failed_id, message_content, workspace).await?;
                    trace!(
                        "Returned from handle_failed_json_repair => now returning error for ID={}",
                        failed_id
                    );
                    Err(e_extract.into())
                }
            }
        }
        ExpectedContentType::PlainText => {
            trace!(
                "Received plain text content for request {} => length={}",
                success_body.id(),
                message_content.len()
            );
            let index = BatchIndex::from_uuid_str(success_body.id())?;
            trace!("Parsed BatchIndex => {:?}", index);

            let text_path = workspace.text_storage_path(&index);
            info!("writing plain text output to {:?}", text_path);
            write_to_file(&text_path, message_content.as_str()).await?;
            trace!("Successfully wrote plain text file => {:?}", text_path);
            trace!(
                "Exiting handle_successful_response with success_body ID: {}",
                success_body.id()
            );
            Ok(())
        }
        _ => {
            todo!("Unsupported ExpectedContentType variant encountered.")
        }
    }
}

#[cfg(test)]
mod handle_successful_response_tests {
    use super::*;
    use std::fs;

    #[derive(Debug, Deserialize, Serialize, NamedItem)]
    pub struct MockItemForSuccess {
        pub name: String,
    }

    #[traced_test]
    async fn test_handle_successful_response_json_failure() {
        // This test tries to parse invalid JSON into our `MockItemForSuccess`,
        // expecting it to fail and log a file in `failed_json_repairs_dir`.
        trace!("===== BEGIN TEST: test_handle_successful_response_json_failure =====");

        // 1) Use ephemeral default workspace (no overrides).
        let workspace = BatchWorkspace::new_temp().await.unwrap();
        info!("Created ephemeral workspace: {:?}", workspace);

        // 2) Ensure repairs dir is empty
        let repairs_dir = workspace.failed_json_repairs_dir();

        // 3) Create a response that is *not* valid JSON
        let invalid_msg = ChatCompletionResponseMessage {
            role: Role::Assistant,
            content: Some("this is not valid json at all".into()),
            audio: None,
            function_call: None,
            refusal: None,
            tool_calls: None,
        };

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

        // 4) Call handle_successful_response with ExpectedContentType=Json
        let rc = handle_successful_response::<MockItemForSuccess>(
            &success_body,
            workspace.as_ref(),
            &ExpectedContentType::Json
        ).await;

        // 5) Confirm it fails
        assert!(rc.is_err(), "We expect an error due to invalid JSON content");

        // 6) Confirm the "some-other-uuid" file is in the ephemeral repairs dir
        let repair_path = repairs_dir.join("some-other-uuid");
        trace!("Asserting that repair file path exists: {:?}", repair_path);
        assert!(repair_path.exists(), "A repair file must be created for invalid JSON");

        trace!("===== END TEST: test_handle_successful_response_json_failure =====");
    }
}
