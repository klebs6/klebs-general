// ---------------- [ File: ai-json-template-derive/tests/trybuild_justification/pass_enum_tuple_with_hashmap.rs ]
#![allow(dead_code)]
#![allow(unused_imports)]

use ai_json_template::*;
use ai_json_template_derive::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use getset::{Getters, Setters};
use derive_builder::Builder;
use save_load_derive::*;
use save_load_traits::*;

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
enum TupleEnumWithHashMap {
    /// A single-field tuple variant that is a HashMap
    MapOnly(HashMap<String, u32>),

    /// Two fields: a string and a map
    Mixed(String, HashMap<u8, f32>),
}

impl Default for TupleEnumWithHashMap {
    fn default() -> Self {
        TupleEnumWithHashMap::MapOnly(HashMap::default())
    }
}

fn main() {
    println!("Compiled pass_enum_tuple_with_hashmap.rs successfully!");
}
