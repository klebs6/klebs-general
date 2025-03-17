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
