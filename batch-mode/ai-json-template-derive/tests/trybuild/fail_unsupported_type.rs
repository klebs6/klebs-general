// ---------------- [ File: tests/trybuild/fail_unsupported_type.rs ]
use ai_json_template_derive::*;
use ai_json_template::*;
use save_load_traits::*;
use batch_mode_3p::*;
use serde::{Serialize, Deserialize};

#[derive(Debug,Clone,AiJsonTemplate, Serialize, Deserialize)]
struct BadConfig {
    /// i32 is not allowed if your macro only supports String, Option<String>, Vec<String>, or nested.
    number: i32,
}

impl_default_save_to_file_traits!{BadConfig}

fn main() {
    // We expect a compile-time error about "Unsupported field type for AiJsonTemplate: i32"
    // The test harness calls `t.compile_fail("...")` on this file.
}
