// ---------------- [ File: ai-json-template-derive/tests/trybuild_justification/fail_hashmap_badtype_value.rs ]
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
use tracing::*;

/// Fake "BadType" that your macro's logic doesn't support => triggers compile_error
struct BadType;

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
struct MapWithBadValue {
    my_map: HashMap<u8, BadType>,
}

fn main() {
    println!("Should fail with compile_error about 'BadType'");
}
