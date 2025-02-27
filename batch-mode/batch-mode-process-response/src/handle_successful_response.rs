// ---------------- [ File: src/handle_successful_response.rs ]
crate::ix!();

pub async fn handle_successful_response(
    success_body:          &BatchSuccessResponseBody,
    workspace:             &dyn BatchWorkspaceInterface,
    expected_content_type: &ExpectedContentType,

) -> Result<(), BatchSuccessResponseHandlingError> {

    let choice = &success_body.choices()[0];
    let message_content = choice.message().content();

    if *choice.finish_reason() == FinishReason::Length {
        handle_finish_reason_length(success_body.id(), message_content).await?;
    }

    match expected_content_type {
        ExpectedContentType::Json => {
            // Always attempt parse-with-repair, which does a normal parse first
            // and repairs only if it looks broken. It’ll succeed immediately if all is well.
            match message_content.extract_clean_parse_json_with_repair() {
                Ok(json_content) => {
                    let token_name_field = extract_token_name_field(&json_content)?;
                    let token_name       = CamelCaseTokenWithComment::from_str(&token_name_field)?;
                    let target_path      = workspace.token_expansion_path(&token_name);

                    // Pretty-print the JSON
                    let serialized_json = serde_json::to_string_pretty(&json_content)
                        .map_err(JsonParseError::SerdeError)?;

                    write_to_file(&target_path, &serialized_json).await?;
                    Ok(())
                }
                Err(e) => {
                    let failed_id = success_body.id();
                    handle_failed_json_repair(failed_id, message_content, workspace).await?;
                    Err(e.into())
                }
            }
        }
        ExpectedContentType::PlainText => {
            // If we just expect raw text, do whatever you need with it.
            trace!(
                "Received plain text content for request {}:\n{}",
                success_body.id(),
                message_content.as_str()
            );

            let index     = BatchIndex::from_uuid_str(success_body.id())?;
            let text_path = workspace.text_storage_path(&index);
            write_to_file(&text_path, message_content.as_str()).await?;
            Ok(())
        }
    }
}
