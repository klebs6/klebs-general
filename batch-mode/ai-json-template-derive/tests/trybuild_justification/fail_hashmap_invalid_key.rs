// ---------------- [ File: ai-json-template-derive/tests/trybuild_justification/fail_hashmap_invalid_key.rs ]
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

/// Attempting a HashMap<bool, String>, which your macro presumably rejects as an invalid key type.
#[derive(
    SaveLoad,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    AiJsonTemplate,
    AiJsonTemplateWithJustification,
    Eq,
    PartialEq,
)]
struct BadMapKeyStruct {
    kv: HashMap<bool, String>,
}

fn main() {
    println!("Should not compile: HashMap<bool, String> is invalid key type");
}
