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
use tracing::*;

#[derive(
    SaveLoad,
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    AiJsonTemplate,
    AiJsonTemplateWithJustification,
)]
enum NamedEnumWithMap {
    EmptyVariant,

    NumericStuff {
        count: u32,
        label: String,
    },

    MapVariant {
        items: HashMap<u8, String>,
    }
}

impl Default for NamedEnumWithMap {
    fn default() -> Self {
        Self::EmptyVariant
    }

}

fn main() {
    println!("Compiled pass_enum_named_with_hashmap.rs successfully!");
}
