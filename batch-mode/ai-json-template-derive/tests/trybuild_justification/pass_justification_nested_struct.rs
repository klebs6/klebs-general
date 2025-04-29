// ---------------- [ File: ai-json-template-derive/tests/trybuild_justification/pass_justification_nested_struct.rs ]
// tests/trybuild_justification/pass_justification_nested_struct.rs

#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use] extern crate getset;

use ai_json_template::*;
use ai_json_template_derive::*;
use serde::*;
use getset::*;
use derive_builder::Builder;
use save_load_derive::*;
use save_load_traits::*;

// Both nested and outer must be PartialEq
#[derive(Default,Getters,Setters,Debug, Clone, PartialEq, Serialize, Deserialize, Builder, SaveLoad)]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct InnerPart {
    detail: String,
}

#[derive(Default,Getters,Setters,Debug, Clone, PartialEq, Serialize, Deserialize, Builder, SaveLoad)]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct OuterPart {
    notes: String,
    inner: InnerPart,
}

fn main() {
    let _my_outer = OuterPart {
        notes: "outer notes".to_string(),
        inner: InnerPart { detail: "details...".to_string() },
    };
    println!("Compiled pass_justification_nested_struct.rs successfully!");
}
