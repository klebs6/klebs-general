// ---------------- [ File: ai-json-template-derive/tests/trybuild_justification/fail_unnamed_struct_hashmap.rs ]
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

/// A tuple struct => should fail
#[derive(
    SaveLoad,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    AiJsonTemplate,
    AiJsonTemplateWithJustification,
)]
struct TupleStructMap(HashMap<String, bool>);

fn main() {}
