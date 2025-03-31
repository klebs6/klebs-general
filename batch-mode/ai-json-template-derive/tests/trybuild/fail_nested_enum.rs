// ---------------- [ File: ai-json-template-derive/tests/trybuild/fail_nested_enum.rs ]
// ======================= File: tests/trybuild/fail_nested_enum.rs =======================
// Tests a struct that nests an enum. The macro's logic (in this codebase) doesn't handle enums,
// so it should fail.

#![allow(dead_code)]

use ai_json_template_derive::*;
use ai_json_template::*;
use serde::{Serialize, Deserialize};
use getset::{Getters, Setters};
use derive_builder::Builder;
use save_load_derive::*;
use save_load_traits::*;

#[derive(SaveLoad,Clone, Debug, Serialize, Deserialize)]
enum InnerEnum {
    VariantA,
    VariantB,
}

#[derive(SaveLoad,Clone, Debug, Serialize, Deserialize, Getters, Setters, Builder)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct FailNestedEnum {
    /// The presence of an enum here should cause the derive to fail
    something: InnerEnum,

    /// Some other field
    text: String,
}

fn main() {}
