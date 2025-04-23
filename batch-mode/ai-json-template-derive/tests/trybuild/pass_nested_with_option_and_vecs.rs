// ---------------- [ File: ai-json-template-derive/tests/trybuild/pass_nested_with_option_and_vecs.rs ]
// ======================= File: tests/trybuild/pass_nested_with_option_and_vecs.rs =======================
// Another nested struct scenario, this time with multiple Option and Vec fields at different levels.

#![allow(dead_code)]
#![allow(unused_imports)]

use ai_json_template_derive::*;
use ai_json_template::*;
use serde::{Serialize, Deserialize};
use getset::{Getters, Setters};
use derive_builder::Builder;
use save_load_derive::*;
use save_load_traits::*;

/// Deeply nested struct with various optional fields
#[derive(SaveLoad,Clone, Debug, Serialize, Deserialize, Getters, Setters, Builder)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct NestedOptionals {
    /// Could be present or absent
    notes: Option<String>,

    /// Could also be present or absent
    tags: Option<Vec<String>>,
}

/// Top-level struct that includes multiple optional fields and some required ones
#[derive(SaveLoad,Clone, Debug, Serialize, Deserialize, Getters, Setters, Builder)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct MultiOptionalOuter {
    /// Always required
    main_title: String,

    /// Potentially omitted
    meta_info: Option<String>,

    /// Another vector, always required
    data_points: Vec<String>,

    /// Nested structure with optionals
    nested_details: NestedOptionals,
}
fn main() {}
