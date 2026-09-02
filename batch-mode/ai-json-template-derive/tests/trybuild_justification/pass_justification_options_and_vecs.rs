// ---------------- [ File: ai-json-template-derive/tests/trybuild_justification/pass_justification_options_and_vecs.rs ]
// tests/trybuild_justification/pass_justification_options_and_vecs.rs

#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use] extern crate getset;

use ai_json_template_derive::*;
use ai_json_template::*;
use serde::*;
use getset::*;
use derive_builder::Builder;
use save_load_derive::*;
use save_load_traits::*;
use tracing::*;

#[derive(Default,Getters,Setters,Debug, Clone, PartialEq, Serialize, Deserialize, Builder, SaveLoad)]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct OptionalFieldsStruct {
    /// Some optional text
    note: Option<String>,
    /// A list of integers
    values: Vec<u32>,
    /// A plain required field
    label: String,
}

fn main() {
    let _item = OptionalFieldsStruct {
        note: None,
        values: vec![1, 2, 3],
        label: "Hello".into(),
    };
    println!("Compiled pass_justification_options_and_vecs.rs successfully!");
}
