#![allow(unused_imports)]
// ---------------- [ File: tests/trybuild/pass_complex_struct.rs ]
// ======================= File: tests/trybuild/pass_complex_struct.rs =======================
// Demonstrates a named struct with multiple valid field types: String, Vec<String>, Option<String>,
// and a nested struct. This should compile successfully under the AiJsonTemplate derive.

#![allow(dead_code)]

use ai_json_template_derive::*;
use ai_json_template::*;
use serde::{Serialize, Deserialize};
use getset::{Getters, Setters};
use derive_builder::Builder;
use save_load_derive::*;
use save_load_traits::*;

/// A nested struct that also implements the AiJsonTemplate trait.
#[derive(SaveLoad,Clone, Debug, Serialize, Deserialize, Getters, Setters, Builder)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct InnerPart {
    /// Some detail line
    detail: String,
}

/// A more complex struct showcasing multiple field types and a nested struct.
#[derive(SaveLoad,Clone, Debug, Serialize, Deserialize, Getters, Setters, Builder)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct ComplexStruct {
    /// Required text field
    summary: String,

    /// A list of items
    items: Vec<String>,

    /// A potentially missing field
    optional_note: Option<String>,

    /// A nested structure
    inner: InnerPart,
}

fn main() {}
