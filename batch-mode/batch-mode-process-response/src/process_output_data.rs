// ---------------- [ File: src/process_output_data.rs ]
crate::ix!();

pub async fn process_output_data(
    output_data:           &BatchOutputData,
    workspace:             &dyn BatchWorkspaceInterface,
    expected_content_type: &ExpectedContentType,

) -> Result<(), BatchOutputProcessingError> {

    let mut failed_entries = Vec::new();

    for response_record in output_data.responses() {

        info!("-------[processing output data record]");

        if let Some(success_body) = response_record.response().body().as_success() {

            if let Err(e) = handle_successful_response(success_body,workspace,expected_content_type).await {

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
        save_failed_entries(workspace,&failed_entries).await?;
    }

    Ok(())
}
