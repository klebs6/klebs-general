// ---------------- [ File: ai-json-template-derive/tests/trybuild/fail_missing_serde.rs ]
// tests/trybuild-tests/fail_missing_serde.rs
use ai_json_template_derive::*;
use ai_json_template::*;
use save_load_traits::*;
use save_load_derive::*;

// Missing Serialize/Deserialize => The macro should fail with an error.
#[derive(SaveLoad,Debug,Clone,AiJsonTemplate)]
struct NoSerde {
    text: String,
}

fn main() {}
