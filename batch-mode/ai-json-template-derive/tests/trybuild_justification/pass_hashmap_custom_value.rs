// ---------------- [ File: ai-json-template-derive/tests/trybuild_justification/pass_hashmap_custom_value.rs ]
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

/// A nested struct that can be the value in the HashMap
#[derive(
    SaveLoad,
    Debug,
    Default,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    AiJsonTemplate,
    AiJsonTemplateWithJustification,
)]
struct NestedWeight {
    ratio: f32,
}

/// A top-level struct containing a HashMap<u8, NestedWeight>
#[derive(
    SaveLoad,
    Debug,
    Default,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    AiJsonTemplate,
    AiJsonTemplateWithJustification,
)]
struct HashMapWithNested {
    /// The map from some level number => nested weight
    levels: HashMap<u8, NestedWeight>,
}

fn main() {
    let _example = HashMapWithNested {
        levels: {
            let mut hm = HashMap::new();
            hm.insert(1, NestedWeight { ratio: 0.75 });
            hm
        },
    };
    println!("Compiled pass_hashmap_custom_value.rs successfully!");
}
