// ---------------- [ File: language-model-batch-workflow-derive/tests/trybuild/fail_missing_batch_workspace.rs ]
//
// Similarly, no `#[batch_workspace]` => compile-fail.

use language_model_batch_workflow_derive::LanguageModelBatchWorkflow;
use batch_mode_batch_workflow::*;

// "fail_missing_batch_workspace.rs"
#[derive(LanguageModelBatchWorkflow)]
struct MissingWorkspace {
    #[batch_client]
    client: std::sync::Arc<OpenAIClientHandle<OpenAIClientError>>,

    #[model_type]
    mt: LanguageModelType,
}

struct MyErr;

// Expect an error: “Missing required `#[batch_workspace]`.”
fn main() {}
