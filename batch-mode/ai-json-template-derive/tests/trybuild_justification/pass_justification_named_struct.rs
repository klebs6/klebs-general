// ---------------- [ File: ai-json-template-derive/tests/trybuild_justification/pass_justification_named_struct.rs ]
// tests/trybuild_justification/pass_justification_named_struct.rs
//
// 1) We import getset::* so the attribute macros are in scope.
// 2) We add PartialEq to the user struct.

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

#[derive(
    Default,
    Debug,
    Clone,
    PartialEq,  // needed so the expansions can do PartialEq on the “Justified” struct
    Serialize,
    Deserialize,
    Builder,
    SaveLoad,
    Getters,
    Setters,
    AiJsonTemplate,
    AiJsonTemplateWithJustification,
)]
#[getset(get="pub", set="pub", get_mut="pub", set_with="pub")]
#[builder(setter(into))]
struct BasicStruct {
    /// A numeric field
    count: u8,
    /// A string field
    label: String,
}

fn main() {
    let _item = BasicStruct {
        count: 7,
        label: "Hello".into(),
    };
    println!("Compiled pass_justification_named_struct.rs successfully!");
}
