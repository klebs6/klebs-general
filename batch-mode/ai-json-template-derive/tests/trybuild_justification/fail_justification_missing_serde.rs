// ---------------- [ File: ai-json-template-derive/tests/trybuild_justification/fail_justification_missing_serde.rs ]
// tests/trybuild_justification/fail_justification_missing_serde.rs
//
// We omit Serialize/Deserialize => must fail to compile.

#![allow(dead_code)]
#![allow(unused_imports)]

use ai_json_template::*;
use ai_json_template_derive::*;
use save_load_traits::*;
use save_load_derive::*;
use getset::*;
use serde::*;

// Missing #[derive(Serialize, Deserialize)] => This should fail.
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
#[derive(Debug, Clone)]
struct MissingSerdeField {
    text: String,
}

fn main() {}
