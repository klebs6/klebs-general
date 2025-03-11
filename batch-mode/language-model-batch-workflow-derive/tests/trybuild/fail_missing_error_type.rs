// ---------------- [ File: tests/trybuild/fail_missing_error_type.rs ]
//
// If the user does not provide `#[batch_error_type(...)]`, parse logic also fails.

use language_model_batch_workflow_derive::LanguageModelBatchWorkflow;

// No `#[batch_error_type(MyErr)]` => fails parse.
#[derive(LanguageModelBatchWorkflow)]
struct MissingErrorType {
    #[batch_client]
    client: (),

    #[batch_workspace]
    workspace: (),

    #[model_type]
    mt: (),
}

// We intentionally do NOT define MyErr or annotate the struct with `#[batch_error_type(MyErr)]`.
// The macro should fail with “Missing required `#[batch_error_type(...)]`.”
fn main() {}
