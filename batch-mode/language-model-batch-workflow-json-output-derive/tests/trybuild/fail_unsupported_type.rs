// ---------------- [ File: tests/trybuild/fail_unsupported_type.rs ]
use language_model_batch_workflow_json_output_derive::*;
use batch_mode_batch_workflow::*;
use serde::{Serialize, Deserialize};

#[derive(AiJsonTemplate, Serialize, Deserialize)]
struct BadConfig {
    /// i32 is not allowed if your macro only supports String, Option<String>, Vec<String>, or nested.
    number: i32,
}

fn main() {
    // We expect a compile-time error about "Unsupported field type for AiJsonTemplate: i32"
    // The test harness calls `t.compile_fail("...")` on this file.
}
