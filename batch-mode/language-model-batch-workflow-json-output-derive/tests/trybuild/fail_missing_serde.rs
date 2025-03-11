// ---------------- [ File: tests/trybuild/fail_missing_serde.rs ]
// tests/trybuild-tests/fail_missing_serde.rs
use language_model_batch_workflow_json_output_derive::*;
use batch_mode_batch_workflow::*;

// Missing Serialize/Deserialize => The macro should fail with an error.
#[derive(AiJsonTemplate)]
struct NoSerde {
    text: String,
}

fn main() {}
