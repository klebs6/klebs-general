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
