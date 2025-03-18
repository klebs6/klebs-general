// ---------------- [ File: tests/trybuild/pass_named_struct.rs ]
#![allow(unused_imports)]
// ======================= File: tests/trybuild/pass_named_struct.rs =======================
// A minimal named struct that should pass compilation with AiJsonTemplate derive.
// (We show it compiling successfully.)

#![allow(dead_code)]

use ai_json_template_derive::*;
use ai_json_template::*;
use serde::{Serialize, Deserialize};
use getset::{Getters, Setters};
use derive_builder::Builder;
use save_load_derive::*;
use save_load_traits::*;

#[derive(SaveLoad,Clone, Debug, Serialize, Deserialize, Getters, Setters, Builder)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]  // Demonstrating usage; no fields are public
#[builder(setter(into))]
struct PassingNamedStruct {
    /// Just a string field
    some_field: String,
}
fn main() {}
