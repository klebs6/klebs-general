// ---------------- [ File: ai-json-template-derive/tests/trybuild_justification/pass_enum_named_with_hashmap.rs ]
#![allow(dead_code)]
#![allow(unused_imports)]

use ai_json_template::*;
use ai_json_template_derive::*;
use serde::{Serialize, Deserialize};
use getset::{Getters, Setters};
use std::collections::HashMap;
use derive_builder::Builder;
use save_load_derive::*;
use save_load_traits::*;

#[derive(
    SaveLoad,
    Default,
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    AiJsonTemplate,
    AiJsonTemplateWithJustification,
)]
enum NamedEnumWithMap {
    /// No fields here
    #[default]
    EmptyVariant,

    /// A normal variant with some numeric field
    NumericStuff {
        count: u32,
        label: String,
    },

    /// A variant that includes a HashMap
    MapVariant {
        items: HashMap<u8, String>,
    },
}

fn main() {
    println!("Compiled pass_enum_named_with_hashmap.rs successfully!");
}
