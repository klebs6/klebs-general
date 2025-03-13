// ---------------- [ File: src/process_output_file.rs ]
crate::ix!();

/**
 * The core async function to process the output file 
 * for a given triple. Now requiring T: 'static + Send + Sync.
 */
pub async fn process_output_file<T>(
    triple:                &BatchFileTriple,
    workspace:             &dyn BatchWorkspaceInterface,
    expected_content_type: &ExpectedContentType,
) -> Result<(), BatchOutputProcessingError> 
where
    T: 'static + Send + Sync + DeserializeOwned + Named + GetTargetPathForAIExpansion,
{
    trace!("process_output_file => index = {:?}", triple.index());
    let output_data = load_output_file(triple.output().as_ref().unwrap()).await?;
    process_output_data::<T>(&output_data, workspace, expected_content_type).await
}

/**
 * The bridging function EXACTLY matches the `BatchWorkflowProcessOutputFileFn` type:
 * 
 *   for<'a> fn(
 *       &'a BatchFileTriple,
 *       &'a (dyn BatchWorkspaceInterface + 'a),
 *       &'a ExpectedContentType,
 *   ) -> Pin<Box<dyn Future<Output=Result<(),BatchOutputProcessingError>> + Send + 'a>>
 */
pub fn process_output_file_bridge_fn<'a, T>(
    triple:    &'a BatchFileTriple,
    workspace: &'a (dyn BatchWorkspaceInterface + 'a),
    ect:       &'a ExpectedContentType,
) -> Pin<Box<dyn Future<Output = Result<(), BatchOutputProcessingError>> + Send + 'a>> 
where
    T: 'static + Send + Sync + DeserializeOwned + Named + GetTargetPathForAIExpansion,
{
    Box::pin(async move {
        process_output_file::<T>(triple, workspace, ect).await
    })
}

/// A non-generic fallback bridging function:
pub fn default_output_file_bridge_fn<'a>(
    triple:    &'a BatchFileTriple,
    workspace: &'a (dyn BatchWorkspaceInterface + 'a),
    ect:       &'a ExpectedContentType,
) -> Pin<Box<dyn Future<Output = Result<(), BatchOutputProcessingError>> + Send + 'a>>
{
    Box::pin(async move {
        // Use CamelCaseTokenWithComment (or another default type) here:
        process_output_file::<CamelCaseTokenWithComment>(triple, workspace, ect).await
    })
}

/// The const pointer the macro references.
pub const DEFAULT_OUTPUT_FILE_BRIDGE: BatchWorkflowProcessOutputFileFn 
    = default_output_file_bridge_fn;
