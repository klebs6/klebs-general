// ---------------- [ File: src/process_batch_output_and_errors.rs ]
crate::ix!();

pub async fn process_batch_output_and_errors(
    workspace:              &dyn BatchWorkspaceInterface, 
    batch_execution_result: &BatchExecutionResult,
    expected_content_type:  &ExpectedContentType,

) -> Result<(),BatchProcessingError> {

    // Process the outputs
    if let Some(output_data) = &batch_execution_result.outputs() {
        info!("processing batch output data of len {}", output_data.len());
        process_output_data(&output_data,workspace,expected_content_type).await?;
    }

    // Process the errors
    if let Some(error_data) = &batch_execution_result.errors() {
        info!("processing batch error data of len {}", error_data.len());
        process_error_data(&error_data).await?;
    }

    Ok(())
}
