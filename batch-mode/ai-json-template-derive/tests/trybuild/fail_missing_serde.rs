// ---------------- [ File: tests/trybuild/fail_missing_serde.rs ]
// tests/trybuild-tests/fail_missing_serde.rs
use ai_json_template_derive::*;
use ai_json_template::*;
use save_load_traits::*;
use batch_mode_3p::*;

// Missing Serialize/Deserialize => The macro should fail with an error.
#[derive(Debug,Clone,AiJsonTemplate)]
struct NoSerde {
    text: String,
}

impl_default_save_to_file_traits!{NoSerde}

fn main() {}
