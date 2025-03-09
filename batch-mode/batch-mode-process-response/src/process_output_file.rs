// ---------------- [ File: src/process_output_file.rs ]
crate::ix!();

/**
 * This is the actual async function that processes the output file 
 * for a given triple, using the provided workspace and expected content type.
 */
pub async fn process_output_file(
    triple:                &BatchFileTriple,
    workspace:             &dyn BatchWorkspaceInterface,
    expected_content_type: &ExpectedContentType,
) -> Result<(), BatchOutputProcessingError> {
    let output_data = load_output_file(triple.output().as_ref().unwrap()).await?;
    process_output_data(&output_data, workspace, expected_content_type).await
}

/**
 * This bridging function EXACTLY matches the `BatchWorkflowProcessOutputFileFn` type:
 * 
 *  for<'a> fn(
 *      &'a BatchFileTriple,
 *      &'a (dyn BatchWorkspaceInterface + 'a),
 *      &'a ExpectedContentType,
 *  ) -> Pin<Box<dyn Future<Output=Result<(),BatchOutputProcessingError>> + Send + 'a>>
 *  
 *   Notice `workspace: &'a (dyn BatchWorkspaceInterface + 'a)`, 
 *   and the return type includes `+ 'a`.
 */
fn process_output_file_bridge_fn<'a>(
    triple: &'a BatchFileTriple,
    workspace: &'a (dyn BatchWorkspaceInterface + 'a),
    ect: &'a ExpectedContentType,
) -> Pin<Box<dyn Future<Output = Result<(), BatchOutputProcessingError>> + Send + 'a>>
{
    Box::pin(async move {
        process_output_file(triple, workspace, ect).await
    })
}

/**
 * We expose a CONST of type `BatchWorkflowProcessOutputFileFn`, so that 
 * passing `&PROCESS_OUTPUT_FILE_BRIDGE` has the correct signature 
 * recognized by the compiler.
 */
pub const PROCESS_OUTPUT_FILE_BRIDGE: BatchWorkflowProcessOutputFileFn = process_output_file_bridge_fn;
