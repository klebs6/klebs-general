// ---------------- [ File: tests/trybuild/fail_multiple_unsupported_types.rs ]
// ======================= File: tests/trybuild/fail_multiple_unsupported_types.rs =======================
// A struct that uses multiple unsupported numeric types to ensure the macro fails.

#![allow(dead_code)]

use ai_json_template_derive::AiJsonTemplate;
use ai_json_template::*;
use serde::{Serialize, Deserialize};
use getset::{Getters, Setters};
use derive_builder::Builder;
use save_load_derive::*;
use save_load_traits::*;

#[derive(SaveLoad,Clone, Debug, Serialize, Deserialize, Getters, Setters, Builder)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct FailMultipleUnsupportedTypes {
    /// An integer field not supported by AiJsonTemplate in this codebase
    quantity: i64,

    /// A floating-point field also not supported
    ratio: f32,

    /// Another integer
    count: usize,
}

fn main() {}
