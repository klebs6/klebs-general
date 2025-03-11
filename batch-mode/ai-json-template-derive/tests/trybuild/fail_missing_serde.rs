// ---------------- [ File: tests/trybuild/fail_missing_serde.rs ]
// tests/trybuild-tests/fail_missing_serde.rs
use ai_json_template_derive::*;
use ai_json_template::*;

// Missing Serialize/Deserialize => The macro should fail with an error.
#[derive(AiJsonTemplate)]
struct NoSerde {
    text: String,
}

fn main() {}
