use language_model_batch_workflow_derive::LanguageModelBatchWorkflow;
use std::sync::Arc;
use batch_mode_batch_workflow::*;

#[derive(LanguageModelBatchWorkflow)]
#[batch_error_type(MyErr)]
pub struct MyValidStruct {
    // Must be Arc<dyn LanguageModelClientInterface<OpenAIClientError>> or Arc<OpenAIClientHandle>
    #[batch_client]
    client: Arc<dyn LanguageModelClientInterface<OpenAIClientError>>,

    // Use Arc<BatchWorkspace> to avoid trait upcasting issues
    #[batch_workspace]
    workspace: Arc<BatchWorkspace>,

    #[expected_content_type]
    ect: ExpectedContentType,

    #[model_type]
    mt: LanguageModelType,

    // optional process batch functions => must be BatchWorkflowProcessOutputFileFn / BatchWorkflowProcessErrorFileFn
    #[custom_process_batch_output_fn]
    pbo: BatchWorkflowProcessOutputFileFn,

    #[custom_process_batch_error_fn]
    pbe: BatchWorkflowProcessErrorFileFn,
}

impl ComputeLanguageModelRequests for MyValidStruct {
    type Seed = ();  // or some real “Seed” type

    fn compute_language_model_requests(
        &self,
        _model: &LanguageModelType,
        _input_tokens: &[Self::Seed]
    ) -> Vec<LanguageModelBatchAPIRequest> {
        // Just return nothing for the test's sake:
        vec![]
    }
}

// user-defined error
struct MyErr;

impl From<BatchWorkspaceError> for MyErr {
    fn from(_: BatchWorkspaceError) -> Self { MyErr }
}
impl From<BatchReconciliationError> for MyErr {
    fn from(_: BatchReconciliationError) -> Self { MyErr }
}
impl From<BatchInputCreationError> for MyErr {
    fn from(_: BatchInputCreationError) -> Self { MyErr }
}
impl From<BatchProcessingError> for MyErr {
    fn from(_: BatchProcessingError) -> Self { MyErr }
}
impl From<FileMoveError> for MyErr {
    fn from(_: FileMoveError) -> Self { MyErr }
}

// This should compile with no errors if we have actual real types.
fn main() {}
