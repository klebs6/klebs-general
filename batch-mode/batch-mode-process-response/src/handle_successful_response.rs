// ---------------- [ File: src/handle_successful_response.rs ]
crate::ix!();

#[instrument(level="trace", skip_all)]
pub async fn handle_successful_response<T>(
    success_body:          &BatchSuccessResponseBody,
    workspace:             &dyn BatchWorkspaceInterface,
    expected_content_type: &ExpectedContentType,
) -> Result<(), BatchSuccessResponseHandlingError> 
where
    // We require T be something we can fully deserialize from JSON
    // (DeserializeOwned), plus Named, plus 'static + Send + Sync.
    T: 'static + Send + Sync + Named + DeserializeOwned + GetTargetPathForAIExpansion,
{
    trace!("entering handle_successful_response with success_body ID: {}", success_body.id());

    let choice = &success_body.choices()[0];
    let message_content = choice.message().content();

    if *choice.finish_reason() == FinishReason::Length {
        handle_finish_reason_length(success_body.id(), message_content).await?;
    }

    match expected_content_type {
        ExpectedContentType::Json => {
            // Attempt to parse & repair JSON
            match message_content.extract_clean_parse_json_with_repair() {
                Ok(json_content) => {
                    debug!("JSON parse/repair succeeded for success_body ID: {}", success_body.id());

                    // 1) Deserialize into T so we can access T::name().
                    //    If T is not a perfect match for the entire JSON, that's OK
                    //    as long as it can parse the relevant fields. We'll keep
                    //    the original (potentially bigger) JSON for writing out.
                    let typed_item: T = serde_json::from_value(json_content.clone())?;

                    // 2) Convert T into an Arc<dyn ...> if needed:
                    let typed_item_arc: Arc<dyn GetTargetPathForAIExpansion + Send + Sync + 'static> =
                        Arc::new(typed_item);

                    // 3) Determine the output file path.
                    //    (We still rely on T for `Named` and possibly other logic,
                    //     such as `target_path_for_ai_json_expansion`.)
                    let target_path = workspace.target_path(&typed_item_arc, expected_content_type);

                    // 4) Pretty-print the *fully-repaired* JSON we got from
                    //    `extract_clean_parse_json_with_repair()`. If you prefer
                    //    to re-serialize `typed_item`, you can do so, but that
                    //    might drop fields unknown to T.
                    let serialized_json = serde_json::to_string_pretty(&json_content)
                        .map_err(JsonParseError::SerdeError)?;

                    info!("writing JSON output to {:?}", target_path);
                    write_to_file(&target_path, &serialized_json).await?;
                    Ok(())
                }
                Err(e) => {
                    warn!(
                        "JSON extraction/repair failed for success_body ID: {} with error: {:?}",
                        success_body.id(),
                        e
                    );
                    let failed_id = success_body.id();
                    handle_failed_json_repair(failed_id, message_content, workspace).await?;
                    Err(e.into())
                }
            }
        }
        ExpectedContentType::PlainText => {
            trace!(
                "Received plain text content for request {}:\n{}",
                success_body.id(),
                message_content.as_str()
            );

            let index = BatchIndex::from_uuid_str(success_body.id())?;
            let text_path = workspace.text_storage_path(&index);
            info!("writing plain text output to {:?}", text_path);
            write_to_file(&text_path, message_content.as_str()).await?;
            Ok(())
        }
    }
}
