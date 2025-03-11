// ---------------- [ File: tests/trybuild/fail_missing_batch_client.rs ]
//
// Because the macro requires `#[batch_client]` on exactly one field, failing to provide it
// should yield a compile error. The harness calls `t.compile_fail(...)`, expecting an error.

use language_model_batch_workflow_derive::LanguageModelBatchWorkflow;
use batch_mode_batch_workflow::*;

// Missing `#[batch_client]` => should fail at compile time with a clear error about
// “Missing required `#[batch_client]`.”

// "fail_missing_batch_client.rs"
#[derive(LanguageModelBatchWorkflow)]
struct MissingClient {
    #[batch_workspace]
    ws: std::sync::Arc<BatchWorkspace>, // <-- now meets the new type requirement
    #[model_type]
    mt: LanguageModelType,
}

// We define the custom error type, so that part is satisfied:
struct MyErr;

// Because there's no #[batch_client], the derive macro logic should fail parse.
fn main() {}
