// ---------------- [ File: ai-json-template-derive/tests/trybuild_justification/fail_justification_enum_field.rs ]
// tests/trybuild_justification/fail_justification_enum_field.rs
//
// We nest an enum inside the struct => your macro doesn't handle enums in the fields
// => compilation fails.

#![allow(dead_code)]
#![allow(unused_imports)]

use ai_json_template_derive::*;
use ai_json_template::*;
use serde::*;
use getset::*;
use derive_builder::Builder;
use save_load_derive::*;
use save_load_traits::*;
use tracing::*;

#[derive(PartialEq,Eq,SaveLoad, Debug, Clone, Serialize, Deserialize)]
enum BadEnum {
    A,
    B,
}

#[derive(PartialEq,Eq,SaveLoad, Debug, Clone, Serialize, Deserialize, Getters, Setters, Builder)]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct ContainsEnum {
    // This field is an enum => should fail.
    data: BadEnum,
}

fn main() {}
